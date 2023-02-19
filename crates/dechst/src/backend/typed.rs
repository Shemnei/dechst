pub mod read {
	use crate::backend::{BackendRead, ObjectKind, ObjectMetadata, Result};
	use crate::id::Id;

	pub trait ConfigRead {
		fn meta(&self) -> Result<ObjectMetadata>;
		fn read_all(&self, buf: &mut Vec<u8>) -> Result<usize>;
	}

	impl<T> ConfigRead for T
	where
		T: BackendRead,
	{
		fn meta(&self) -> Result<ObjectMetadata> {
			<Self as BackendRead>::meta(self, ObjectKind::Config, &Id::ZERO)
		}

		fn read_all(&self, buf: &mut Vec<u8>) -> Result<usize> {
			<Self as BackendRead>::read_all(self, ObjectKind::Config, &Id::ZERO, buf)
		}
	}

	pub trait ConfigReadExt {
		fn read(&self) -> Result<Vec<u8>>;
	}

	impl<T> ConfigReadExt for T
	where
		T: ConfigRead,
	{
		fn read(&self) -> Result<Vec<u8>> {
			let meta = <Self as ConfigRead>::meta(self)?;
			let mut buf = Vec::with_capacity(meta.len as usize);

			let len = <Self as ConfigRead>::read_all(self, &mut buf)?;
			buf.truncate(len);

			Ok(buf)
		}
	}

	macro_rules! obj_read {
		( $obj:expr => $ty:ident, $ext:ident ) => {
			pub trait $ty {
				type Iter: Iterator<Item = Result<Id>>;

				fn iter(&self) -> Result<Self::Iter>;

				fn meta(&self, id: &Id) -> Result<ObjectMetadata>;

				fn read_all(&self, id: &Id, buf: &mut Vec<u8>) -> Result<usize>;
			}

			impl<T> $ty for T
			where
				T: BackendRead,
			{
				type Iter = T::Iter;

				fn iter(&self) -> Result<Self::Iter> {
					<T as BackendRead>::iter(self, $obj)
				}

				fn meta(&self, id: &Id) -> Result<ObjectMetadata> {
					<T as BackendRead>::meta(self, $obj, id)
				}

				fn read_all(&self, id: &Id, buf: &mut Vec<u8>) -> Result<usize> {
					<T as BackendRead>::read_all(self, $obj, id, buf)
				}
			}

			pub trait $ext {
				fn read(&self, id: &Id) -> Result<Vec<u8>>;
			}

			impl<T> $ext for T
			where
				T: $ty,
			{
				fn read(&self, id: &Id) -> Result<Vec<u8>> {
					let meta = <Self as $ty>::meta(self, id)?;

					let mut buf = Vec::with_capacity(meta.len as usize);

					let len = <Self as $ty>::read_all(self, id, &mut buf)?;
					buf.truncate(len);

					Ok(buf)
				}
			}
		};
	}

	obj_read!(ObjectKind::Index => IndexRead, IndexReadExt);
	obj_read!(ObjectKind::Key => KeyRead, KeyReadExt);
	obj_read!(ObjectKind::Snapshot => SnapshotRead, SnapshotReadExt);
	obj_read!(ObjectKind::Pack => PackRead, PackReadExt);

	pub trait TypedBackendRead: BackendRead {
		type Config: ConfigRead;
		type Index: IndexRead;
		type Key: KeyRead;
		type Snapshot: SnapshotRead;
		type Pack: PackRead;

		fn config(&self) -> &Self::Config;
		fn indices(&self) -> &Self::Index;
		fn keys(&self) -> &Self::Key;
		fn snapshots(&self) -> &Self::Snapshot;
		fn packs(&self) -> &Self::Pack;
	}

	impl<T> TypedBackendRead for T
	where
		T: BackendRead,
	{
		type Config = Self;
		type Index = Self;
		type Key = Self;
		type Pack = Self;
		type Snapshot = Self;

		fn config(&self) -> &Self::Config {
			self
		}

		fn indices(&self) -> &Self::Index {
			self
		}

		fn keys(&self) -> &Self::Key {
			self
		}

		fn snapshots(&self) -> &Self::Snapshot {
			self
		}

		fn packs(&self) -> &Self::Pack {
			self
		}
	}
}

pub mod write {
	use crate::backend::{BackendWrite, ObjectKind, Result};
	use crate::id::Id;

	pub trait ConfigWrite {
		fn write_all(&mut self, buf: &[u8]) -> Result<()>;
	}

	impl<T> ConfigWrite for T
	where
		T: BackendWrite,
	{
		fn write_all(&mut self, buf: &[u8]) -> Result<()> {
			<Self as BackendWrite>::write_all(self, ObjectKind::Config, &Id::ZERO, buf)
		}
	}

	macro_rules! obj_write {
		( $obj:expr => $ty:ident) => {
			pub trait $ty {
				fn remove(&mut self, id: &Id) -> Result<()>;
				fn write_all(&mut self, id: &Id, buf: &[u8]) -> Result<()>;
			}

			impl<T> $ty for T
			where
				T: BackendWrite,
			{
				fn remove(&mut self, id: &Id) -> Result<()> {
					<Self as BackendWrite>::remove(self, $obj, id)
				}

				fn write_all(&mut self, id: &Id, buf: &[u8]) -> Result<()> {
					<Self as BackendWrite>::write_all(self, $obj, id, buf)
				}
			}
		};
	}

	obj_write!(ObjectKind::Index => IndexWrite);
	obj_write!(ObjectKind::Key => KeyWrite);
	obj_write!(ObjectKind::Snapshot => SnapshotWrite);
	obj_write!(ObjectKind::Pack => PackWrite);

	pub trait TypedBackendWrite: BackendWrite {
		type Config: ConfigWrite;
		type Index: IndexWrite;
		type Key: KeyWrite;
		type Snapshot: SnapshotWrite;
		type Pack: PackWrite;

		fn config_mut(&mut self) -> &mut Self::Config;
		fn indices_mut(&mut self) -> &mut Self::Index;
		fn keys_mut(&mut self) -> &mut Self::Key;
		fn snapshots_mut(&mut self) -> &mut Self::Snapshot;
		fn packs_mut(&mut self) -> &mut Self::Pack;
	}

	impl<T> TypedBackendWrite for T
	where
		T: BackendWrite,
	{
		type Config = Self;
		type Index = Self;
		type Key = Self;
		type Pack = Self;
		type Snapshot = Self;

		fn config_mut(&mut self) -> &mut Self::Config {
			self
		}

		fn indices_mut(&mut self) -> &mut Self::Index {
			self
		}

		fn keys_mut(&mut self) -> &mut Self::Key {
			self
		}

		fn snapshots_mut(&mut self) -> &mut Self::Snapshot {
			self
		}

		fn packs_mut(&mut self) -> &mut Self::Pack {
			self
		}
	}
}

#[cfg(test)]
mod test {
	use super::read::*;
	use super::write::*;
	use crate::backend::test::A;
	use crate::backend::{BackendRead, BackendWrite, Result};
	use crate::id::Id;

	#[test]
	fn a() {
		let mut a = A;

		process(&mut a);

		fn process<W: TypedBackendWrite>(r: &mut W) {
			r.config_mut().write_all(&mut vec![]).unwrap();
			r.indices_mut().write_all(&Id::ZERO, &mut vec![]).unwrap();
		}
	}
}
