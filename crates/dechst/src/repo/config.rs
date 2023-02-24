use crate::backend::ext::ReadToEnd;
use crate::backend::BackendWrite;
use crate::id::Id;
use crate::obj::config::Config;
use crate::obj::lock::sealed::{AccessExclusive, AccessShared};
use crate::obj::ObjectKind;
use crate::process::format::Formatter;
use crate::process::pipeline::unprocess;
use crate::repo::{LockedRepo, Result};

const OBJ: ObjectKind = ObjectKind::Config;

pub trait ConfigRead {
	fn config_exists(&self) -> Result<()>;
	fn config_read(&self) -> Result<Config>;
}

impl<B: BackendWrite, CONFIG, INDEX, KEY, SNAPSHOT, PACK> ConfigRead
	for LockedRepo<B, CONFIG, INDEX, KEY, SNAPSHOT, PACK>
where
	CONFIG: AccessShared,
{
	fn config_exists(&self) -> Result<()> {
		self.backend.exists(OBJ, &Id::ZERO)
	}

	fn config_read(&self) -> Result<Config> {
		let bytes = self.backend.read_to_end(OBJ, &Id::ZERO).unwrap();

		Ok(unprocess(Formatter::Cbor, &self.key, &bytes).unwrap())
	}
}

pub trait ConfigUpdate {}

impl<B: BackendWrite, CONFIG, INDEX, KEY, SNAPSHOT, PACK> ConfigUpdate
	for LockedRepo<B, CONFIG, INDEX, KEY, SNAPSHOT, PACK>
where
	CONFIG: AccessExclusive,
{
}
