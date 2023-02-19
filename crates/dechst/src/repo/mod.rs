use crate::backend::ext::{Find, FindIdExt, ReadToEnd};
use crate::backend::BackendWrite;
use crate::id::Id;
use crate::obj::config::Config;
use crate::obj::key::{EncryptedKey, Key};
use crate::obj::lock::{Lock, LockMeta, LockState, Shared};
use crate::obj::ObjectKind;
use crate::process::format::{FormatterParams, Format};
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

		Ok(FormatterParams::Cbor.parse(&key).unwrap())
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
		})
	}

	pub fn decrypt(self, key_id: Id, password: &[u8]) -> Result<DecryptedRepo<B>, (Self, Error)> {
		let key = self.get_key(key_id).unwrap();
		let key = key.decrypt(password);

		Ok(DecryptedRepo {
			backend: self.backend,
			key,
		})
	}
}

#[derive(Debug)]
pub struct DecryptedRepo<B> {
	backend: B,
	key: Key,
}

impl<B: BackendWrite> DecryptedRepo<B> {
	pub fn lock<CONFIG, INDEX, KEY, SNAPSHOT, PACK>(
		self,
		marker: LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, PACK>,
	) -> Result<LockedRepo<B, CONFIG, INDEX, KEY, SNAPSHOT, PACK>, (Self, Error)>
	where
		LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, PACK>: Into<LockState> + Copy,
	{
		let state: LockState = marker.into();
		let meta: LockMeta = LockMeta::new();

		let lock = Lock { state, meta };
		let lock = RepoLock {
			lock,
			_marker: marker,
		};

		Ok(LockedRepo {
			backend: self.backend,
			key: self.key,
			_lock: lock,
		})
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
pub struct LockedRepo<B, CONFIG, INDEX, KEY, SNAPSHOT, PACK> {
	backend: B,
	key: Key,
	_lock: RepoLock<CONFIG, INDEX, KEY, SNAPSHOT, PACK>,
}

impl<B, CONFIG, INDEX, KEY, SNAPSHOT, PACK> LockedRepo<B, CONFIG, INDEX, KEY, SNAPSHOT, PACK> {
	pub fn unlock(self) -> DecryptedRepo<B> {
		DecryptedRepo {
			backend: self.backend,
			key: self.key,
		}
	}
}

impl<B, INDEX, KEY, SNAPSHOT, PACK> LockedRepo<B, Shared, INDEX, KEY, SNAPSHOT, PACK> {
	pub fn read_config(&self) -> Result<Config> {
		todo!()
	}
}
