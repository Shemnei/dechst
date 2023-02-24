use super::DecryptedRepo;
use crate::backend::ext::{Find, FindIdExt, ReadToEnd};
use crate::backend::BackendWrite;
use crate::id::Id;
use crate::obj::lock::Lock;
use crate::obj::ObjectKind;
use crate::process::format::Formatter;
use crate::process::pipeline::unprocess;
use crate::repo::Result;

const OBJ: ObjectKind = ObjectKind::Lock;

pub trait LockRead {
	type Iter: Iterator<Item = Result<Id>>;

	fn lock_exists(&self, id: &Id) -> Result<()>;
	fn locks(&self) -> Result<Self::Iter>;
	fn lock_read(&self, id: &Id) -> Result<Lock>;
	fn locks_find(&self, ids: &[&str]) -> Result<Vec<Find>>;
	fn lock_find(&self, id: &str) -> Result<Option<Find>>;
}

impl<B: BackendWrite> LockRead for DecryptedRepo<B> {
	type Iter = B::Iter;

	fn lock_exists(&self, id: &Id) -> Result<()> {
		self.backend.exists(OBJ, id)
	}

	fn locks(&self) -> Result<B::Iter> {
		self.backend.iter(OBJ)
	}

	fn lock_read(&self, id: &Id) -> Result<Lock> {
		let bytes = self.backend.read_to_end(OBJ, id).unwrap();

		let lock = unprocess(Formatter::Cbor, &self.key, &bytes).unwrap();

		Ok(lock)
	}

	fn locks_find(&self, ids: &[&str]) -> Result<Vec<Find>> {
		self.backend.find_ids(OBJ, ids)
	}

	fn lock_find(&self, id: &str) -> Result<Option<Find>> {
		self.backend.find_id(OBJ, id)
	}
}

pub trait LockUpdate {}
