use crate::backend::ext::{Find, FindIdExt, ReadToEnd};
use crate::backend::BackendWrite;
use crate::id::Id;
use crate::obj::key::Key;
use crate::obj::lock::sealed::AccessShared;
use crate::obj::ObjectKind;
use crate::process::format::Formatter;
use crate::process::pipeline::unprocess;
use crate::repo::{LockedRepo, Result};

const OBJ: ObjectKind = ObjectKind::Index;

pub trait IndexRead {
	type Iter: Iterator<Item = Result<Id>>;

	fn index_exists(&self, id: &Id) -> Result<()>;
	fn indices(&self) -> Result<Self::Iter>;
	fn index_read(&self, id: &Id) -> Result<Key>;
	fn indices_find(&self, ids: &[&str]) -> Result<Vec<Find>>;
	fn index_find(&self, id: &str) -> Result<Option<Find>>;
}

impl<B: BackendWrite, CONFIG, INDEX, KEY, SNAPSHOT, PACK> IndexRead
	for LockedRepo<B, CONFIG, INDEX, KEY, SNAPSHOT, PACK>
where
	INDEX: AccessShared,
{
	type Iter = B::Iter;

	fn index_exists(&self, id: &Id) -> Result<()> {
		self.backend.exists(OBJ, id)
	}

	fn indices(&self) -> Result<B::Iter> {
		self.backend.iter(OBJ)
	}

	fn index_read(&self, id: &Id) -> Result<Key> {
		let bytes = self.backend.read_to_end(OBJ, id).unwrap();

		let lock = unprocess(Formatter::Cbor, &self.key, &bytes).unwrap();

		Ok(lock)
	}

	fn indices_find(&self, ids: &[&str]) -> Result<Vec<Find>> {
		self.backend.find_ids(OBJ, ids)
	}

	fn index_find(&self, id: &str) -> Result<Option<Find>> {
		self.backend.find_id(OBJ, id)
	}
}

pub trait IndexUpdate {}
