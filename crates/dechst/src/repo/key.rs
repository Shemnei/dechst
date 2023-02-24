use crate::backend::ext::{Find, FindIdExt, ReadToEnd};
use crate::backend::BackendWrite;
use crate::id::Id;
use crate::obj::key::{EncryptedKey, Key};
use crate::obj::lock::sealed::AccessShared;
use crate::obj::ObjectKind;
use crate::process::format::{Format, Formatter};
use crate::repo::{LockedRepo, Result};

const OBJ: ObjectKind = ObjectKind::Key;

pub trait KeyRead {
	type Iter: Iterator<Item = Result<Id>>;

	fn key_exists(&self, id: &Id) -> Result<()>;
	fn keys(&self) -> Result<Self::Iter>;
	fn key_read(&self, id: &Id, user_key: &[u8]) -> Result<Key>;
	fn keys_find(&self, ids: &[&str]) -> Result<Vec<Find>>;
	fn key_find(&self, id: &str) -> Result<Option<Find>>;
}

impl<B: BackendWrite, CONFIG, INDEX, KEY, SNAPSHOT, PACK> KeyRead
	for LockedRepo<B, CONFIG, INDEX, KEY, SNAPSHOT, PACK>
where
	KEY: AccessShared,
{
	type Iter = B::Iter;

	fn key_exists(&self, id: &Id) -> Result<()> {
		self.backend.exists(OBJ, id)
	}

	fn keys(&self) -> Result<B::Iter> {
		self.backend.iter(OBJ)
	}

	fn key_read(&self, id: &Id, user_key: &[u8]) -> Result<Key> {
		let bytes = self.backend.read_to_end(OBJ, id).unwrap();

		let enc_key: EncryptedKey = Formatter::Cbor.parse(&bytes).unwrap();

		Ok(enc_key.decrypt(user_key))
	}

	fn keys_find(&self, ids: &[&str]) -> Result<Vec<Find>> {
		self.backend.find_ids(OBJ, ids)
	}

	fn key_find(&self, id: &str) -> Result<Option<Find>> {
		self.backend.find_id(OBJ, id)
	}
}

pub trait KeyUpdate {}
