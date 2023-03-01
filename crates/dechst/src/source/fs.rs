use std::fs::{self, File, Metadata, ReadDir};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeZone, Utc};

use super::{Item, Source};
use crate::obj::tree::node::{Node, NodeKind};
use crate::path::Segment;

impl Item for PathBuf {
	fn can_descend(&self) -> bool {
		let Ok(meta) = std::fs::symlink_metadata(self) else {
			return false;
		};

		meta.is_dir()
	}
}

#[derive(Debug)]
pub struct FsSource(PathBuf);

impl FsSource {
	pub fn new<P: Into<PathBuf>>(path: P) -> Self {
		Self(path.into())
	}

	fn resolve_item(&self, item: &PathBuf) -> PathBuf {
		self.0.join(item)
	}

	fn fs_meta(&self, path: &Path) -> Result<Metadata, std::io::Error> {
		std::fs::symlink_metadata(path)
	}

	fn extract_meta(&self, meta: &Metadata) -> Result<crate::os::Metadata, std::io::Error> {
		#[cfg(target_family = "unix")]
		{
			return Ok(crate::os::unix::Metadata::from(meta).into());
		}

		#[cfg(target_family = "windows")]
		{
			return Ok(crate::os::windows::Metadata::from(meta).into());
		}

		todo!("Implement generic/wasm")
	}
}

impl Source for FsSource {
	type Error = std::io::Error;
	type Item = PathBuf;
	type Iter = Iter;
	type Read = BufReader<File>;

	fn iter(&self, item: Option<&Self::Item>) -> Result<Self::Iter, Self::Error> {
		// TODO: Check "jail" escape (e.g. test/../../..)

		let path = if let Some(item) = item {
			self.resolve_item(item)
		} else {
			self.0.to_path_buf()
		};

		Ok(Iter(path.read_dir()?))
	}

	fn read(&self, item: &Self::Item) -> Result<Self::Read, Self::Error> {
		let file = File::open(item)?;
		Ok(BufReader::new(file))
	}

	fn node(&self, item: &Self::Item) -> Result<Node, Self::Error> {
		let path = self.resolve_item(item);
		let fsmeta = self.fs_meta(&path)?;

		let meta = self.extract_meta(&fsmeta)?;
		let kind = (path.as_ref(), &fsmeta).try_into()?;
		let name = Segment::from(path.file_name().unwrap().to_os_string());

		let node = Node { name, kind, meta };

		Ok(node)
	}
}

#[derive(Debug)]
pub struct Iter(ReadDir);

impl Iterator for Iter {
	type Item = Result<PathBuf, std::io::Error>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|o| o.map(|d| d.path()))
	}
}

#[cfg(target_family = "unix")]
impl From<&fs::Metadata> for crate::os::unix::Permissions {
	fn from(value: &fs::Metadata) -> Self {
		use std::os::unix::fs::MetadataExt;

		let mode = value.mode();

		Self { mode }
	}
}

#[cfg(target_family = "unix")]
impl From<&fs::Metadata> for crate::os::unix::Times {
	fn from(value: &fs::Metadata) -> Self {
		use std::os::unix::fs::MetadataExt;

		let access = Some(epoch_to_utc(
			value.atime(),
			value.atime_nsec().try_into().unwrap(),
		));
		let modify = Some(epoch_to_utc(
			value.mtime(),
			value.mtime_nsec().try_into().unwrap(),
		));
		let change = Some(epoch_to_utc(
			value.ctime(),
			value.ctime_nsec().try_into().unwrap(),
		));
		let create = value.created().ok().map(|st| DateTime::<Utc>::from(st));

		Self {
			access,
			modify,
			change,
			create,
		}
	}
}

#[cfg(target_family = "unix")]
impl From<&fs::Metadata> for crate::os::unix::Identifier {
	fn from(value: &fs::Metadata) -> Self {
		use std::os::unix::fs::MetadataExt;

		let dev = value.dev();
		let ino = value.ino();

		Self { dev, ino }
	}
}

#[cfg(target_family = "unix")]
impl From<&fs::Metadata> for crate::os::unix::Metadata {
	fn from(value: &fs::Metadata) -> Self {
		let user = crate::os::unix::User::default();

		Self {
			user,
			perm: value.into(),
			time: value.into(),
			ident: value.into(),
			len: value.len(),
		}
	}
}

#[cfg(target_family = "unix")]
impl TryFrom<(&Path, &fs::Metadata)> for NodeKind {
	type Error = std::io::Error;

	fn try_from((path, meta): (&Path, &fs::Metadata)) -> Result<Self, Self::Error> {
		use std::os::unix::fs::{FileTypeExt, MetadataExt};

		let ftype = meta.file_type();

		let kind = if ftype.is_file() {
			NodeKind::file()
		} else if ftype.is_dir() {
			NodeKind::dir()
		} else if ftype.is_symlink() {
			let target = std::fs::read_link(path)?;
			NodeKind::symlink(target.into_os_string())
		} else if ftype.is_char_device() {
			NodeKind::cdev(meta.rdev())
		} else if ftype.is_block_device() {
			NodeKind::dev(meta.rdev())
		} else if ftype.is_fifo() {
			NodeKind::fifo()
		} else if ftype.is_socket() {
			NodeKind::socket()
		} else {
			unreachable!("All file types covered");
		};

		Ok(kind)
	}
}

fn epoch_to_utc(secs: i64, nsecs: u32) -> DateTime<Utc> {
	Utc.timestamp_opt(secs, nsecs).single().unwrap()
}

#[cfg(target_family = "windows")]
impl From<&fs::Metadata> for crate::os::windows::Permissions {
	fn from(value: &fs::Metadata) -> Self {
		use std::os::windows::fs::MetadataExt;

		let attributes = value.file_attributes();

		Self { attributes }
	}
}

#[cfg(target_family = "windows")]
impl From<&fs::Metadata> for crate::os::windows::Times {
	fn from(value: &fs::Metadata) -> Self {
		let access = value.accessed().ok().map(|st| DateTime::<Utc>::from(st));
		let modify = value.modified().ok().map(|st| DateTime::<Utc>::from(st));
		let create = value.created().ok().map(|st| DateTime::<Utc>::from(st));

		Self {
			access,
			modify,
			create,
		}
	}
}

#[cfg(target_family = "windows")]
impl From<&fs::Metadata> for crate::os::windows::Identifier {
	fn from(value: &fs::Metadata) -> Self {
		use std::os::windows::fs::MetadataExt;

		let volume_serial_number = value.volume_serial_number();
		let file_index = value.file_index();

		Self {
			volume_serial_number,
			file_index,
		}
	}
}

#[cfg(target_family = "windows")]
impl From<&fs::Metadata> for crate::os::windows::Metadata {
	fn from(value: &fs::Metadata) -> Self {
		let user = crate::os::windows::User::default();

		Self {
			user,
			perm: value.into(),
			time: value.into(),
			ident: value.into(),
			len: value.len(),
		}
	}
}

#[cfg(target_family = "windows")]
impl From<&fs::FileType> for crate::obj::tree::node::TargetHint {
	fn from(value: &fs::FileType) -> Self {
		use std::os::windows::fs::FileTypeExt;

		use crate::obj::tree::node::TargetHint;

		if value.is_symlink_dir() {
			TargetHint::Directory
		} else if value.is_symlink_file() {
			TargetHint::File
		} else {
			unreachable!("All file types covered");
		}
	}
}

#[cfg(target_family = "windows")]
impl TryFrom<(&Path, &fs::Metadata)> for NodeKind {
	type Error = std::io::Error;

	fn try_from((path, meta): (&Path, &fs::Metadata)) -> Result<Self, Self::Error> {
		use std::os::windows::fs::{FileTypeExt, MetadataExt};

		let ftype = meta.file_type();

		let kind = if ftype.is_file() {
			NodeKind::file()
		} else if ftype.is_dir() {
			NodeKind::dir()
		} else if ftype.is_symlink() {
			let target = std::fs::read_link(path)?;
			NodeKind::symlink(target.into_os_string(), (&ftype).into())
		} else {
			unreachable!("All file types covered");
		};

		Ok(kind)
	}
}
