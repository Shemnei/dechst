use std::borrow::Cow;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read as _, Seek as _, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use walkdir::WalkDir;

use crate::backend::{BackendRead, BackendWrite, ObjectMetadata, Result};
use crate::id::Id;
use crate::obj::{ObjectKind, DIRECTORY_OBJECTS};

#[derive(Debug, Clone)]
pub struct Local2 {
	path: PathBuf,
}

impl Local2 {
	pub fn new<P: Into<PathBuf>>(path: P) -> Self {
		Self { path: path.into() }
	}

	fn resolve_path(&self, kind: ObjectKind, id: &Id) -> PathBuf {
		let hex = id.to_hex();

		match kind {
			ObjectKind::Config => self.path.join(kind.name()),
			// TODO: Make configurable (0..2)
			ObjectKind::Pack => self.path.join(&hex[0..2]).join(hex),
			_ => self.path.join(kind.name()).join(hex),
		}
	}

	fn check_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
		let path = self.path.join(path);

		println!("{}", path.display());

		if !path.try_exists().unwrap() {
			return Err(());
		}

		if path.metadata().unwrap().permissions().readonly() {
			return Err(());
		}

		Ok(path)
	}
}

impl BackendRead for Local2 {
	type Iter = Iter;

	fn mount_point(&self) -> Cow<'_, str> {
		self.path.to_string_lossy()
	}

	fn verify(&self) -> Result<()> {
		// Config
		{
			let path = self.check_path("config")?;
			if !path.is_file() {
				return Err(());
			}
		}

		for kind in DIRECTORY_OBJECTS {
			if !self.check_path(kind.name())?.is_dir() {
				return Err(());
			}
		}

		Ok(())
	}

	fn iter(&self, kind: ObjectKind) -> Result<Self::Iter> {
		if kind == ObjectKind::Config {
			Ok(Iter::from_config(
				std::fs::try_exists(self.path.join(kind.name())).unwrap(),
			))
		} else {
			Ok(Iter::new(self.path.join(kind.name())))
		}
	}

	fn exists(&self, kind: ObjectKind, id: &Id) -> Result<()> {
		let path = self.resolve_path(kind, id);

		if !path.try_exists().unwrap() {
			return Err(());
		}

		Ok(())
	}

	fn meta(&self, kind: ObjectKind, id: &Id) -> Result<ObjectMetadata> {
		let path = self.resolve_path(kind, id);

		let meta = path.metadata().unwrap();

		Ok(ObjectMetadata {
			accessed: meta.accessed().ok().map(|t| t.into()),
			created: meta.created().ok().map(|t| t.into()),
			modified: meta.modified().ok().map(|t| t.into()),
			len: meta.len(),
		})
	}

	fn read_at(&self, kind: ObjectKind, id: &Id, offset: u32, buf: &mut [u8]) -> Result<usize> {
		let path = self.resolve_path(kind, id);

		let mut r = File::open(path).unwrap();

		r.seek(SeekFrom::Start(offset as u64)).unwrap();

		r.read_exact(buf).unwrap();

		Ok(buf.len())
	}

	fn read_all(&self, kind: ObjectKind, id: &Id, buf: &mut Vec<u8>) -> Result<usize> {
		let path = self.resolve_path(kind, id);

		let mut r = File::open(path).unwrap();

		Ok(r.read_to_end(buf).unwrap())
	}
}

pub struct Iter {
	inner: Box<dyn Iterator<Item = Result<Id>>>,
}

impl fmt::Debug for Iter {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("Iter")
	}
}

impl Iter {
	fn new(path: PathBuf) -> Self {
		let iter = WalkDir::new(path)
			.into_iter()
			.filter_map(walkdir::Result::ok)
			.filter(|e| e.file_type().is_file())
			.map(|e| Id::from_str(&e.file_name().to_string_lossy()))
			.map(|e| if let Ok(e) = e { Ok(e) } else { Err(()) });

		Self {
			inner: Box::new(iter),
		}
	}

	fn from_config(exists: bool) -> Self {
		let iter: Box<dyn Iterator<Item = Result<Id>>> = if exists {
			Box::new(std::iter::once(Ok(Id::ZERO)))
		} else {
			Box::new(std::iter::empty())
		};

		Self {
			inner: Box::new(iter),
		}
	}
}

impl Iterator for Iter {
	type Item = Result<Id>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

impl BackendWrite for Local2 {
	fn create(&mut self) -> Result<()> {
		std::fs::create_dir_all(&self.path).unwrap();

		{
			std::fs::File::create(self.path.join(ObjectKind::Config.name())).unwrap();
		}

		for kind in DIRECTORY_OBJECTS {
			std::fs::create_dir_all(self.path.join(kind.name())).unwrap();
		}

		for byte in u8::MIN..=u8::MAX {
			std::fs::create_dir_all(
				self.path
					.join(ObjectKind::Pack.name())
					.join(format!("{byte:02x}")),
			)
			.unwrap();
		}

		Ok(())
	}

	fn remove(&mut self, kind: ObjectKind, id: &Id) -> Result<()> {
		let path = self.resolve_path(kind, id);
		std::fs::remove_file(path).unwrap();
		Ok(())
	}

	fn write_all(&mut self, kind: ObjectKind, id: &Id, buf: &[u8]) -> Result<()> {
		let path = self.resolve_path(kind, id);

		let mut w = OpenOptions::new()
			.create(true)
			.write(true)
			.open(path)
			.unwrap();

		w.set_len(buf.len().try_into().unwrap()).unwrap();
		w.write_all(buf).unwrap();
		w.sync_all().unwrap();

		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn local() {
		let mut l = Local2::new("test_repo");
		l.create().unwrap();
		l.verify().unwrap();
	}
}
