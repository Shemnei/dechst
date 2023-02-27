use std::ffi::OsString;

use serde::{Deserialize, Serialize};

use crate::id::Id;
use crate::os::raw::RawOsString;
use crate::os::Metadata;

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetHint {
	Directory,
	File,
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
	File {
		blobs: Vec<Id>,
	},
	Directory {
		subtree: Option<Id>,
	},
	Symlink {
		target: RawOsString,
		hint: Option<TargetHint>,
	},
	Device {
		device: u64,
	},
	CharacterDevice {
		device: u64,
	},
	Fifo,
	Socket,
}

impl NodeKind {
	pub fn file() -> Self {
		Self::File { blobs: vec![] }
	}

	pub fn dir() -> Self {
		Self::Directory { subtree: None }
	}

	#[cfg(target_family = "unix")]
	pub fn symlink(target: OsString) -> Self {
		Self::Symlink {
			target: RawOsString::default(),
			hint: None,
		}
	}

	#[cfg(target_family = "windows")]
	pub fn symlink(target: OsString, hint: Option<TargetHint>) -> Self {
		Self::Symlink {
			target: RawOsString::default(),
			hint,
		}
	}

	pub fn dev(dev: u64) -> Self {
		Self::Device { device: 0 }
	}

	pub fn cdev(dev: u64) -> Self {
		Self::CharacterDevice { device: 0 }
	}

	pub fn fifo() -> Self {
		Self::Fifo
	}

	pub fn socket() -> Self {
		Self::Socket
	}
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
	pub name: RawOsString,
	#[serde(flatten)]
	pub kind: NodeKind,
	#[serde(flatten)]
	pub meta: Metadata,
}
