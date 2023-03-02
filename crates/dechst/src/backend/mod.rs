pub mod ext;
pub mod local;
pub mod typed;

use std::borrow::Cow;
use std::fmt;

use crate::id::Id;
use crate::obj::{ObjectKind, ObjectMetadata};

pub type Result<T, E = ()> = std::result::Result<T, E>;

pub trait BackendRead: fmt::Debug + Clone + Send + Sync + 'static {
	type Iter: Iterator<Item = Result<Id>>;

	/// The location this backend is mounted to (e.g. file path, url, ...).
	fn mount_point(&self) -> Cow<'_, str>;

	fn verify(&self) -> Result<()>;

	fn iter(&self, kind: ObjectKind) -> Result<Self::Iter>;

	fn exists(&self, kind: ObjectKind, id: &Id) -> Result<()>;

	fn meta(&self, kind: ObjectKind, id: &Id) -> Result<ObjectMetadata>;

	fn read_at(&self, kind: ObjectKind, id: &Id, offset: u32, buf: &mut [u8]) -> Result<usize>;
	fn read_all(&self, kind: ObjectKind, id: &Id, buf: &mut Vec<u8>) -> Result<usize>;
}

pub trait BackendWrite: BackendRead {
	/// Creates a new repository structure.
	fn create(&mut self) -> Result<()>;

	fn remove(&mut self, kind: ObjectKind, id: &Id) -> Result<()>;

	fn write_all(&mut self, kind: ObjectKind, id: &Id, buf: &[u8]) -> Result<()>;
}
