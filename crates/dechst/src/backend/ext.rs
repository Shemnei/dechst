use super::BackendRead;
use crate::backend::{ObjectKind, Result};
use crate::id::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Find {
	None,
	Unique(Id),
	NonUnique,
}

pub trait FindIdExt {
	fn find_ids(&self, kind: ObjectKind, ids: &[&str]) -> Result<Vec<Find>>;

	fn find_id(&self, kind: ObjectKind, id: &str) -> Result<Option<Find>> {
		Ok(self.find_ids(kind, &[id])?.into_iter().next())
	}
}

impl<T> FindIdExt for T
where
	T: BackendRead,
{
	// TODO: Cleanup potential
	fn find_ids(&self, kind: ObjectKind, ids: &[&str]) -> Result<Vec<Find>> {
		if ids.is_empty() {
			return Ok(Vec::new());
		}

		let mut result = vec![Find::None; ids.len()];

		let backend_ids = self.iter(kind)?;

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
					}
				}
			}
		}

		return Ok(result);
	}
}

pub trait ReadToEnd {
	fn read_to_end(&self, kind: ObjectKind, id: &Id) -> Result<Vec<u8>>;
}

impl<T> ReadToEnd for T
where
	T: BackendRead,
{
	fn read_to_end(&self, kind: ObjectKind, id: &Id) -> Result<Vec<u8>> {
		// TODO: Maybe use const size to reduce call overhead
		let meta = self.meta(kind, id)?;

		let mut buf = Vec::with_capacity(meta.len as usize);

		let _bread = self.read_all(kind, id, &mut buf);

		Ok(buf)
	}
}
