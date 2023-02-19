use crate::backend::BackendWrite;
use crate::obj::lock::{Lock, Shared};
use crate::repo::marker::LockMarker;

pub type Error = ();
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Repo<B> {
	backend: B,
}

impl<B: BackendWrite> Repo<B> {
	pub fn create(backend: B) -> Result<DecryptedRepo<B>> {
		let mut backend = backend;

		backend.create().unwrap();

		Ok(DecryptedRepo { backend })
	}

	pub fn open(backend: B) -> Result<Self> {
		backend.verify().unwrap();

		Ok(Self { backend })
	}

	pub fn decrypt(self) -> Result<DecryptedRepo<B>, (Self, Error)> {
		todo!()
	}
}

#[derive(Debug)]
pub struct DecryptedRepo<B> {
	backend: B,
}

impl<B: BackendWrite> DecryptedRepo<B> {
	pub fn lock<CONFIG, INDEX, KEY, SNAPSHOT, PACK>(
		self,
		marker: LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, PACK>,
	) -> Result<LockedRepo<B, LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, PACK>>, (Self, Error)> {
		todo!()
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
pub struct LockState<CONFIG, INDEX, KEY, SNAPSHOT, PACK> {
	lock: Lock,
	_marker: LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, PACK>,
}

#[derive(Debug)]
pub struct LockedRepo<B, L> {
	backend: B,
	// TODO: decrypted state (e.g. key)
	_lock: L,
}

impl<B, L> LockedRepo<B, L> {
	pub fn unlock(self) -> DecryptedRepo<B> {
		DecryptedRepo {
			backend: self.backend,
		}
	}
}

impl<INDEX, KEY, SNAPSHOT, PACK, B: BackendWrite>
	LockedRepo<B, LockMarker<Shared, INDEX, KEY, SNAPSHOT, PACK>>
{
}
