pub mod config;
pub mod index;
pub mod key;
pub mod lock;

use std::fmt;

use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectKind {
	Config,
	Index,
	Key,
	Snapshot,
	Pack,
	Lock,
}

impl fmt::Display for ObjectKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&self, f)
	}
}

impl ObjectKind {
	pub fn name(&self) -> &'static str {
		use ObjectKind::*;

		match self {
			Config => "config",
			Index => "indices",
			Key => "keys",
			Snapshot => "snapshots",
			Pack => "packs",
			Lock => "locks",
		}
	}

	pub fn is_cacheable(&self) -> bool {
		use ObjectKind::*;

		match self {
			Config | Key | Pack | Lock => false,
			Snapshot | Index => true,
		}
	}
}

pub const DIRECTORY_OBJECTS: &[ObjectKind] = &[
	ObjectKind::Index,
	ObjectKind::Key,
	ObjectKind::Snapshot,
	ObjectKind::Pack,
	ObjectKind::Lock,
];

pub trait RepoObject: Serialize + DeserializeOwned + Sized + Send + Sync + 'static {
	const KIND: ObjectKind;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectMetadata {
	pub accessed: Option<DateTime<Utc>>,
	pub created: Option<DateTime<Utc>>,
	pub modified: Option<DateTime<Utc>>,
	pub len: u64,
}
