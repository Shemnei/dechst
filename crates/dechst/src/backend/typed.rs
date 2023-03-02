use super::{BackendRead, BackendWrite};

#[derive(Debug)]
pub struct TypedBackend<B>(B);

impl<B: BackendWrite> TypedBackend<B> {
	pub fn new(backend: B) -> Self {
		Self(backend)
	}
}

impl<B: BackendRead> TypedBackend<B> {
	pub fn read_only(backend: B) -> Self {
		Self(backend)
	}
}

macro_rules! cgt {
	(
		$(
			pub trait $name:ident $body:tt
		)+
	) => {
		$(
			pub trait $name<const OBJECT: ObjectKind> $body
		)+
	};
}

macro_rules! cgti {
	// did = default id
	(
		$(
			$name:ident [ $obj:expr $( => $did:expr )? ] =>
			{
				$( type $ty_name:ident = $ty_ty:ty ; )*
				$( fn $fn:ident (&self $(, $key:ident : $val:ty )* ) $( -> $ret:ty )? ; )*
			}
		)+
	) => {
		$(
			impl<B> $name<{ $obj }> for TypedBackend<B>
			where
				B: BackendRead,
			{
				$( type $ty_name = $ty_ty ; )*
				cgti!(_fn_impl $obj => $( $did => )? $( fn $fn ( &self $(, $key : $val )* ) $( -> $ret )? ; )* );
			}
		)+
	};

	// Function with default id instead of parameter
	(
		_fn_impl $obj:expr => $did:expr =>
		$( fn $fn:ident (&self $(, $key:ident : $val:ty )* ) $( -> $ret:ty )? ; )*
	) => {
		$(
			fn $fn(&self $( , $key: $val)* ) $(-> $ret)? {
				self.0.$fn( $obj , $did $( , $key )* )
			}
		)*
	};

	// Function with only parameters
	(
		_fn_impl $obj:expr =>
		$( fn $fn:ident (&self $(, $key:ident : $val:ty )* ) $( -> $ret:ty )? ; )*
	) => {
		$(
			fn $fn(&self $( , $key: $val)* ) $(-> $ret)? {
				self.0.$fn( $obj $( , $key )* )
			}
		)*
	};
}

pub mod read {
	use super::TypedBackend;
	use crate::backend::{BackendRead, Result};
	use crate::id::Id;
	use crate::obj::{ObjectKind, ObjectMetadata};

	// CONFIG (Special)
	// Did = Default Id

	cgt! {
		pub trait ExistsDid {
			fn exists(&self) -> Result<()>;
		}
		pub trait MetaDid {
			fn meta(&self) -> Result<ObjectMetadata>;
		}
		pub trait ReadAllDid {
			fn read_all(&self, buf: &mut Vec<u8>) -> Result<usize>;
		}
	}

	cgti! {
		ExistsDid[ObjectKind::Config => &Id::ZERO] => {
			fn exists(&self) -> Result<()>;
		}
		MetaDid[ObjectKind::Config => &Id::ZERO] => {
			fn meta(&self) -> Result<ObjectMetadata>;
		}
		ReadAllDid[ObjectKind::Config => &Id::ZERO] => {
			fn read_all(&self, buf: &mut Vec<u8>) -> Result<usize>;
		}
	}

	pub trait GenericReadDid<const OBJECT: ObjectKind>:
		ExistsDid<OBJECT> + MetaDid<OBJECT> + ReadAllDid<OBJECT>
	{
	}

	impl<B, const OBJECT: ObjectKind> GenericReadDid<OBJECT> for TypedBackend<B>
	where
		B: BackendRead,
		TypedBackend<B>: ExistsDid<OBJECT>,
		TypedBackend<B>: MetaDid<OBJECT>,
		TypedBackend<B>: ReadAllDid<OBJECT>,
	{
	}

	// EXISTS
	cgt! {
		pub trait Exists {
			fn exists(&self, id: &Id) -> Result<()>;
		}
	}

	macro_rules! cgti_exists {
		(
			$( $obj:expr ),+
		) => {
			$(
				cgti! {
					Exists[$obj] => {
						fn exists(&self, id: &Id) -> Result<()>;
					}
				}
			)+
		};
	}

	cgti_exists![
		ObjectKind::Index,
		ObjectKind::Key,
		ObjectKind::Snapshot,
		ObjectKind::Pack,
		ObjectKind::Lock
	];

	// ITER

	cgt! {
		pub trait Iter {
			type Iter;
			fn iter(&self) -> Result<Self::Iter>;
		}
	}

	macro_rules! cgti_iter {
		(
			$( $obj:expr ),+
		) => {
			$(
				cgti! {
					Iter[$obj] => {
						type Iter = B::Iter;
						fn iter(&self) -> Result<Self::Iter>;
					}
				}
			)+
		};
	}

	cgti_iter![
		ObjectKind::Index,
		ObjectKind::Key,
		ObjectKind::Snapshot,
		ObjectKind::Pack,
		ObjectKind::Lock
	];

	// META

	cgt! {
		pub trait Meta {
			fn meta(&self, id: &Id) -> Result<ObjectMetadata>;
		}
	}

	macro_rules! cgti_meta {
		(
			$( $obj:expr ),+
		) => {
			$(
				cgti! {
					Meta[$obj] => {
						fn meta(&self, id: &Id) -> Result<ObjectMetadata>;
					}
				}
			)+
		};
	}

	cgti_meta![
		ObjectKind::Index,
		ObjectKind::Key,
		ObjectKind::Snapshot,
		ObjectKind::Pack,
		ObjectKind::Lock
	];

	// READ_ALL

	cgt! {
		pub trait ReadAll {
			fn read_all(&self, id: &Id, buf: &mut Vec<u8>) -> Result<usize>;
		}
	}

	macro_rules! cgti_readall {
		(
			$( $obj:expr ),+
		) => {
			$(
				cgti! {
					ReadAll[$obj] => {
						fn read_all(&self, id: &Id, buf: &mut Vec<u8>) -> Result<usize>;
					}
				}
			)+
		};
	}

	cgti_readall![
		ObjectKind::Index,
		ObjectKind::Key,
		ObjectKind::Snapshot,
		ObjectKind::Pack,
		ObjectKind::Lock
	];

	// READ_AT

	cgt! {
		pub trait ReadAt {
			fn read_at(&self, id: &Id, offset: u32, buf: &mut [u8]) -> Result<usize>;
		}
	}

	macro_rules! cgti_readat {
		(
			$( $obj:expr ),+
		) => {
			$(
				cgti! {
					ReadAt[$obj] => {
						fn read_at(&self, id: &Id, offset: u32, buf: &mut [u8]) -> Result<usize>;
					}
				}
			)+
		};
	}

	cgti_readat![ObjectKind::Pack];

	// GENERIC_READ

	pub trait GenericRead<const OBJECT: ObjectKind>:
		Exists<OBJECT> + Iter<OBJECT> + Meta<OBJECT> + ReadAll<OBJECT>
	{
	}

	impl<B, const OBJECT: ObjectKind> GenericRead<OBJECT> for TypedBackend<B>
	where
		B: BackendRead,
		TypedBackend<B>: Exists<OBJECT>,
		TypedBackend<B>: Iter<OBJECT>,
		TypedBackend<B>: Meta<OBJECT>,
		TypedBackend<B>: ReadAll<OBJECT>,
	{
	}

	pub trait GenericReadExt<const OBJECT: ObjectKind>:
		GenericRead<{ OBJECT }> + ReadAt<{ OBJECT }>
	{
	}

	impl<B> GenericReadExt<{ ObjectKind::Pack }> for TypedBackend<B> where B: BackendRead {}

	// TYPED_READ

	pub trait TypedRead {
		type Iter;

		fn config(&self) -> &dyn GenericReadDid<{ ObjectKind::Config }>;
		fn index(&self) -> &dyn GenericRead<{ ObjectKind::Index }, Iter = Self::Iter>;
		fn key(&self) -> &dyn GenericRead<{ ObjectKind::Key }, Iter = Self::Iter>;
		fn snapshot(&self) -> &dyn GenericRead<{ ObjectKind::Snapshot }, Iter = Self::Iter>;
		fn pack(&self) -> &dyn GenericReadExt<{ ObjectKind::Pack }, Iter = Self::Iter>;
		fn lock(&self) -> &dyn GenericRead<{ ObjectKind::Lock }, Iter = Self::Iter>;
	}

	impl<B> TypedRead for TypedBackend<B>
	where
		B: BackendRead,
	{
		type Iter = B::Iter;

		fn config(&self) -> &dyn GenericReadDid<{ ObjectKind::Config }> {
			self
		}

		fn index(&self) -> &dyn GenericRead<{ ObjectKind::Index }, Iter = Self::Iter> {
			self
		}

		fn key(&self) -> &dyn GenericRead<{ ObjectKind::Key }, Iter = Self::Iter> {
			self
		}

		fn snapshot(&self) -> &dyn GenericRead<{ ObjectKind::Snapshot }, Iter = Self::Iter> {
			self
		}

		fn pack(&self) -> &dyn GenericReadExt<{ ObjectKind::Pack }, Iter = Self::Iter> {
			self
		}

		fn lock(&self) -> &dyn GenericRead<{ ObjectKind::Lock }, Iter = Self::Iter> {
			self
		}
	}

	pub mod ext {
		use super::*;
		use crate::backend::Result;
		use crate::obj::ObjectKind;

		// READ_TO_END
		cgt! {
			pub trait ReadToEndDid {
				fn read_to_end(&self) -> Result<Vec<u8>>;
			}
		}

		impl<T, const OBJECT: ObjectKind> ReadToEndDid<OBJECT> for T
		where
			T: ?Sized + ReadAllDid<OBJECT>,
		{
			fn read_to_end(&self) -> Result<Vec<u8>> {
				let mut buf = Vec::new();
				let bread = self.read_all(&mut buf)?;
				buf.truncate(bread);
				Ok(buf)
			}
		}

		cgt! {
			pub trait ReadToEnd {
				fn read_to_end(&self, id: &Id) -> Result<Vec<u8>>;
			}
		}

		impl<T, const OBJECT: ObjectKind> ReadToEnd<OBJECT> for T
		where
			T: ?Sized + ReadAll<OBJECT>,
		{
			fn read_to_end(&self, id: &Id) -> Result<Vec<u8>> {
				let mut buf = Vec::new();
				let bread = self.read_all(id, &mut buf)?;
				buf.truncate(bread);
				Ok(buf)
			}
		}

		// FIND_IDS
		#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
		pub enum Find {
			None,
			Unique(Id),
			NonUnique,
		}

		cgt! {
			pub trait FindIds {
				fn find_ids(&self, ids: &[&str]) -> Result<Vec<Find>>;
			}
		}

		impl<T, const OBJECT: ObjectKind> FindIds<OBJECT> for T
		where
			T: ?Sized + Iter<OBJECT>,
			T::Iter: Iterator<Item = Result<Id>>,
		{
			fn find_ids(&self, ids: &[&str]) -> Result<Vec<Find>> {
				if ids.is_empty() {
					return Ok(Vec::new());
				}

				let mut result = vec![Find::None; ids.len()];

				let mut non_unique_count = 0;

				let backend_ids = self.iter()?;

				for id in backend_ids {
					let id = id?;
					let hex = id.to_hex();

					for (i, sid) in ids.iter().enumerate() {
						if hex.starts_with(sid) {
							let find = result[i];

							if find == Find::None {
								result[i] = Find::Unique(id);
							} else if matches!(find, Find::Unique(_)) {
								result[i] = Find::NonUnique;

								// Early exit if everything was non unique
								// This can be done as there is no other state
								// beyond `NonUnique`.
								non_unique_count += 1;
								if non_unique_count >= result.len() {
									return Ok(result);
								}
							}
						}
					}
				}

				return Ok(result);
			}
		}

		// FIND_ID
		cgt! {
			pub trait FindId {
				fn find_id(&self, id: &str) -> Result<Find>;
			}
		}

		impl<T, const OBJECT: ObjectKind> FindId<OBJECT> for T
		where
			T: ?Sized + FindIds<OBJECT>,
		{
			fn find_id(&self, id: &str) -> Result<Find> {
				let res = self.find_ids(&[id])?;
				let Some(first) = res.into_iter().next() else {
					return Ok(Find::None);
				};

				Ok(first)
			}
		}
	}

	#[cfg(test)]
	mod tests {
		use super::ext::*;
		use super::*;
		use crate::backend::local::Local;

		#[test]
		fn asdf() {
			let backend = Local::new("/home/mtx/tmp/dechst/crates/dechst-cli/test_repo");
			let typd = TypedBackend::new(backend);

			for x in typd.key().find_ids(&["x"]).unwrap() {
				println!("{x:?}");
			}
		}
	}
}
/*
	pub mod ext {
		use super::ConfigRead;
		use crate::backend::Result;
		use crate::id::Id;

		pub trait ConfigReadToEnd {
			fn read_to_end(&self) -> Result<Vec<u8>>;
		}

		impl<T> ConfigReadToEnd for T
		where
			T: ConfigRead,
		{
			fn read_to_end(&self) -> Result<Vec<u8>> {
				let mut buf = Vec::new();
				let bread = self.read_all(&mut buf)?;
				buf.truncate(bread);
				Ok(buf)
			}
		}

		macro_rules! impl_read_to_end {
			( $name:ident : $read:path ) => {
				pub trait $name {
					fn read_to_end(&self, id: &Id) -> Result<Vec<u8>>;
				}

				impl<T> $name for T
				where
					T: ?Sized + $read,
				{
					fn read_to_end(&self, id: &Id) -> Result<Vec<u8>> {
						let mut buf = Vec::new();
						let bread = self.read_all(id, &mut buf)?;
						buf.truncate(bread);
						Ok(buf)
					}
				}
			};
		}

		impl_read_to_end!(IndexReadToEnd: crate::backend::typed::read::IndexRead);
		impl_read_to_end!(KeyReadToEnd: crate::backend::typed::read::KeyRead);
		impl_read_to_end!(SnapshotReadToEnd: crate::backend::typed::read::SnapshotRead);
		impl_read_to_end!(PackReadToEnd: crate::backend::typed::read::PackRead);
		impl_read_to_end!(LockReadToEnd: crate::backend::typed::read::LockRead);

		/*
		macro_rules! impl_find_ids {
			( $name:ident : $read:path ) => {
				pub trait $name {
					fn read_to_end(&self, id: &Id) -> Result<Vec<u8>>;
				}

				impl<T> $name for T
				where
					T: ?Sized + $read,
				{
					fn read_to_end(&self, id: &Id) -> Result<Vec<u8>> {
						let mut buf = Vec::new();
						let bread = self.read_all(id, &mut buf)?;
						buf.truncate(bread);
						Ok(buf)
					}
				}
			};
		}

		impl_find_ids!(IndexReadToEnd: crate::backend::typed::read::IndexRead);
		impl_find_ids!(KeyReadToEnd: crate::backend::typed::read::KeyRead);
		impl_find_ids!(SnapshotReadToEnd: crate::backend::typed::read::SnapshotRead);
		impl_find_ids!(PackReadToEnd: crate::backend::typed::read::PackRead);
		impl_find_ids!(LockReadToEnd: crate::backend::typed::read::LockRead);
		*/
	}
}

pub mod write {
	use super::read::{
		ConfigRead, IndexRead, KeyRead, LockRead, PackRead, SnapshotRead, TypedRead,
	};
	use super::TypedBackend;
	use crate::backend::{BackendWrite, Result};
	use crate::id::Id;
	use crate::obj::ObjectKind;

	pub trait ConfigWrite: ConfigRead {
		fn remove(&mut self) -> Result<()>;
		fn write_all(&mut self, buf: &[u8]) -> Result<()>;
	}

	macro_rules! generic_write {
		( $name:ident : $read:path ) => {
			pub trait $name: $read {
				fn remove(&mut self, id: &Id) -> Result<()>;
				fn write_all(&mut self, id: &Id, buf: &[u8]) -> Result<()>;
			}
		};
	}

	generic_write!(IndexWrite: crate::backend::typed::read::IndexRead);
	generic_write!(KeyWrite: crate::backend::typed::read::KeyRead);
	generic_write!(SnapshotWrite: crate::backend::typed::read::SnapshotRead);
	generic_write!(PackWrite: crate::backend::typed::read::PackRead);
	generic_write!(LockWrite: crate::backend::typed::read::LockRead);

	pub trait TypedWrite:
		TypedRead + ConfigWrite + IndexWrite + KeyWrite + SnapshotWrite + PackWrite + LockWrite
	{
		fn config_mut(&mut self) -> &mut dyn ConfigWrite {
			self
		}

		fn index_mut(&mut self) -> &mut dyn IndexWrite<Iter = <Self as IndexRead>::Iter> {
			self
		}

		fn key_mut(&mut self) -> &mut dyn KeyWrite<Iter = <Self as KeyRead>::Iter> {
			self
		}

		fn snapshot_mut(&mut self) -> &mut dyn SnapshotWrite<Iter = <Self as SnapshotRead>::Iter> {
			self
		}

		fn pack_mut(&mut self) -> &mut dyn PackWrite<Iter = <Self as PackRead>::Iter> {
			self
		}

		fn lock_mut(&mut self) -> &mut dyn LockWrite<Iter = <Self as LockRead>::Iter> {
			self
		}
	}

	impl<T: BackendWrite> ConfigWrite for TypedBackend<T> {
		fn remove(&mut self) -> Result<()> {
			self.0.remove(ObjectKind::Config, &Id::ZERO)
		}

		fn write_all(&mut self, buf: &[u8]) -> Result<()> {
			self.0.write_all(ObjectKind::Config, &Id::ZERO, buf)
		}
	}

	macro_rules! impl_generic_write {
		( $read:ident : $obj:expr ) => {
			impl<T> $read for crate::backend::typed::TypedBackend<T>
			where
				T: crate::backend::BackendWrite,
			{
				fn remove(&mut self, id: &crate::id::Id) -> crate::backend::Result<()> {
					self.0.remove($obj, id)
				}

				fn write_all(
					&mut self,
					id: &crate::id::Id,
					buf: &[u8],
				) -> crate::backend::Result<()> {
					self.0.write_all($obj, id, buf)
				}
			}
		};
	}

	impl_generic_write!(IndexWrite: crate::obj::ObjectKind::Index);
	impl_generic_write!(KeyWrite: crate::obj::ObjectKind::Key);
	impl_generic_write!(SnapshotWrite: crate::obj::ObjectKind::Snapshot);
	impl_generic_write!(PackWrite: crate::obj::ObjectKind::Pack);
	impl_generic_write!(LockWrite: crate::obj::ObjectKind::Lock);

	impl<B: BackendWrite> TypedWrite for TypedBackend<B> {}
}

*/
