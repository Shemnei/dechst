pub mod ext;
pub mod local;
pub mod local2;
//pub mod typed;

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

#[cfg(test)]
pub(crate) mod test {
	use super::*;

	#[derive(Debug, Clone)]
	pub struct A;
	impl BackendRead for A {
		type Iter = std::vec::IntoIter<Result<Id>>;

		fn mount_point(&self) -> Cow<'_, str> {
			todo!()
		}

		fn verify(&self) -> Result<()> {
			todo!()
		}

		fn iter(&self, kind: crate::backend::ObjectKind) -> crate::backend::Result<Self::Iter> {
			todo!()
		}

		fn exists(&self, kind: ObjectKind, id: &Id) -> Result<()> {
			todo!()
		}

		fn meta(
			&self,
			kind: crate::backend::ObjectKind,
			id: &crate::id::Id,
		) -> crate::backend::Result<crate::backend::ObjectMetadata> {
			todo!()
		}

		fn read_at(
			&self,
			kind: crate::backend::ObjectKind,
			id: &crate::id::Id,
			offset: u32,
			buf: &mut [u8],
		) -> crate::backend::Result<usize> {
			todo!()
		}

		fn read_all(
			&self,
			kind: crate::backend::ObjectKind,
			id: &crate::id::Id,
			buf: &mut Vec<u8>,
		) -> crate::backend::Result<usize> {
			todo!()
		}
	}

	impl BackendWrite for A {
		fn create(&mut self) -> Result<()> {
			todo!()
		}

		fn remove(&mut self, kind: crate::backend::ObjectKind, id: &Id) -> Result<()> {
			todo!()
		}

		fn write_all(
			&mut self,
			kind: crate::backend::ObjectKind,
			id: &Id,
			buf: &[u8],
		) -> Result<()> {
			todo!()
		}
	}
}
