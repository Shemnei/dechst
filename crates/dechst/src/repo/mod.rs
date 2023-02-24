pub mod config;
pub mod index;
pub mod key;
pub mod lock;

use crate::backend::ext::{Find, FindIdExt, ReadToEnd};
use crate::backend::BackendWrite;
use crate::id::Id;
use crate::obj::config::Config;
use crate::obj::key::{EncryptedKey, Key};
use crate::obj::lock::{Lock, LockMeta, LockState};
use crate::obj::ObjectKind;
use crate::process::format::{Format, Formatter};
use crate::process::identify::Identify;
use crate::process::pipeline::{unprocess, ChunkPipeline};
use crate::process::Instanciate;
use crate::repo::marker::LockMarker;

pub type Error = ();
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Repo<B> {
	backend: B,
}

impl<B: BackendWrite> Repo<B> {
	pub fn open(backend: B) -> Result<Self> {
		backend.verify().unwrap();

		Ok(Self { backend })
	}

	fn get_key(&self, key_id: Id) -> Result<EncryptedKey> {
		let key = self.backend.read_to_end(ObjectKind::Key, &key_id).unwrap();

		Ok(Formatter::Cbor.parse(&key).unwrap())
	}

	pub fn keys(&self) -> Result<B::Iter> {
		self.backend.iter(ObjectKind::Key)
	}

	pub fn find_key_id(&self, hex: &str) -> Result<Option<Find>> {
		Ok(self.backend.find_id(ObjectKind::Key, hex).unwrap())
	}

	pub fn try_unencrypted(self, key_id: Id) -> Result<DecryptedRepo<B>, (Self, Error)> {
		let key = self.get_key(key_id).unwrap();
		let key = key.try_unencrypted();

		Ok(DecryptedRepo {
			backend: self.backend,
			key,
			key_id,
		})
	}

	pub fn decrypt(self, key_id: Id, password: &[u8]) -> Result<DecryptedRepo<B>, (Self, Error)> {
		let key = self.get_key(key_id).unwrap();
		let key = key.decrypt(password);

		Ok(DecryptedRepo {
			backend: self.backend,
			key,
			key_id,
		})
	}

	/// # Safety
	/// This is marked as unsafe as the backend should in most cases only be
	/// used by the `Repo` wrapper.
	/// Direct access should be kept to a minimum as there are no safety guarantees
	/// are enforced.
	/// Extreme care must be taken to ensure the repository stays in a valid state.
	pub unsafe fn backend(&self) -> &B {
		&self.backend
	}
}

#[derive(Debug)]
pub struct DecryptedRepo<B> {
	backend: B,
	key: Key,
	key_id: Id,
}

impl<B: BackendWrite> DecryptedRepo<B> {
	pub fn lock<CONFIG, INDEX, KEY, SNAPSHOT, PACK>(
		mut self,
		marker: LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, PACK>,
	) -> Result<LockedRepo<B, CONFIG, INDEX, KEY, SNAPSHOT, PACK>, (Self, Error)>
	where
		LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, PACK>: Into<LockState> + Copy,
	{
		// TODO: Check existing locks

		let state: LockState = marker.into();
		let meta: LockMeta = LockMeta::new();

		let lock = Lock { state, meta };
		let lock = RepoLock {
			lock,
			_marker: marker,
		};

		// Fetch Config
		let config: Config = {
			let bytes = self
				.backend
				.read_to_end(ObjectKind::Config, &Id::ZERO)
				.unwrap();
			unprocess(Formatter::Cbor, &self.key, &bytes).unwrap()
		};

		// Write lock
		let lock_id = {
			let identifier = config.process.identifier.create();
			let pipeline = ChunkPipeline::new(config.process, self.key.clone());

			let bytes = Formatter::Cbor.format(&lock.lock).unwrap();
			let id = identifier.identify(&self.key, &bytes).unwrap();

			let bytes = pipeline.process(&bytes).unwrap();

			self.backend
				.write_all(ObjectKind::Lock, &id, &bytes)
				.unwrap();

			id
		};

		Ok(LockedRepo {
			backend: self.backend,
			key: self.key,
			key_id: self.key_id,
			lock,
			lock_id,
			config,
		})
	}

	pub fn key(&self) -> &Key {
		&self.key
	}

	pub fn key_id(&self) -> &Id {
		&self.key_id
	}

	/// # Safety
	/// This is marked as unsafe as the backend should in most cases only be
	/// used by the `Repo` wrapper.
	/// Direct access should be kept to a minimum as there are no safety guarantees
	/// are enforced.
	/// Extreme care must be taken to ensure the repository stays in a valid state.
	pub unsafe fn backend(&self) -> &B {
		&self.backend
	}

	/// # Safety
	/// This is marked as unsafe as the backend should in most cases only be
	/// used by the `Repo` wrapper.
	/// Direct access should be kept to a minimum as there are no safety guarantees
	/// are enforced.
	/// Extreme care must be taken to ensure the repository stays in a valid state.
	pub unsafe fn into_inner(self) -> (B, Key, Id) {
		let DecryptedRepo {
			backend,
			key,
			key_id,
		} = self;

		(backend, key, key_id)
	}
}

pub mod marker {
	use std::marker::PhantomData;

	use crate::obj::lock::sealed::Access;
	use crate::obj::lock::{Exclusive, None, Shared};

	#[derive(Debug, Clone, Copy)]
	pub struct LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, PACK>(
		PhantomData<(CONFIG, INDEX, KEY, SNAPSHOT, PACK)>,
	);

	impl Default for LockMarker<None, None, None, None, None> {
		fn default() -> Self {
			Self::new()
		}
	}

	impl LockMarker<None, None, None, None, None> {
		pub const NO: Self = Self::new();
	}

	impl LockMarker<Shared, Shared, Shared, Shared, Shared> {
		pub const READ: Self = Self::new();
	}

	impl LockMarker<Exclusive, Exclusive, Exclusive, Exclusive, Exclusive> {
		pub const WRITE: Self = Self::new();
	}

	impl<CONFIG, INDEX, KEY, SNAPSHOT, PACK> LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, PACK> {
		pub const fn new() -> Self {
			Self(PhantomData)
		}

		pub fn config<A: Access>(self) -> LockMarker<A, INDEX, KEY, SNAPSHOT, PACK> {
			LockMarker::new()
		}

		pub fn index<A: Access>(self) -> LockMarker<CONFIG, A, KEY, SNAPSHOT, PACK> {
			LockMarker::new()
		}

		pub fn key<A: Access>(self) -> LockMarker<CONFIG, INDEX, A, SNAPSHOT, PACK> {
			LockMarker::new()
		}

		pub fn snapshot<A: Access>(self) -> LockMarker<CONFIG, INDEX, KEY, A, PACK> {
			LockMarker::new()
		}

		pub fn pack<A: Access>(self) -> LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, A> {
			LockMarker::new()
		}
	}
}

#[derive(Debug)]
pub struct RepoLock<CONFIG, INDEX, KEY, SNAPSHOT, PACK> {
	lock: Lock,
	_marker: LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, PACK>,
}

#[derive(Debug)]
pub struct LockedRepo<B: BackendWrite, CONFIG, INDEX, KEY, SNAPSHOT, PACK> {
	backend: B,
	key: Key,
	key_id: Id,
	lock: RepoLock<CONFIG, INDEX, KEY, SNAPSHOT, PACK>,
	lock_id: Id,
	config: Config,
}

impl<B: BackendWrite, CONFIG, INDEX, KEY, SNAPSHOT, PACK>
	LockedRepo<B, CONFIG, INDEX, KEY, SNAPSHOT, PACK>
{
	fn cleanup(&mut self) -> Result<()> {
		log::debug!("Cleaning up lock {:x}", self.lock_id);

		self.backend
			.remove(ObjectKind::Lock, &self.lock_id)
			.unwrap();

		Ok(())
	}

	pub fn key(&self) -> &Key {
		&self.key
	}

	pub fn key_id(&self) -> &Id {
		&self.key_id
	}

	pub fn lock(&self) -> &Lock {
		&self.lock.lock
	}

	pub fn lock_id(&self) -> &Id {
		&self.lock_id
	}

	pub fn config(&self) -> &Config {
		&self.config
	}
}

impl<B: BackendWrite, CONFIG, INDEX, KEY, SNAPSHOT, PACK> Drop
	for LockedRepo<B, CONFIG, INDEX, KEY, SNAPSHOT, PACK>
{
	fn drop(&mut self) {
		self.cleanup().unwrap();
	}
}
