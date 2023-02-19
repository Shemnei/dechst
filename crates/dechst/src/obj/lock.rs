use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::obj::{ObjectKind, RepoObject};

pub(crate) mod sealed {
	pub trait Access {
		const ACCESS: super::LockAccess;
	}

	pub trait AccessRead: Access {}
	pub trait AccessWrite: AccessRead {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct None {}
impl sealed::Access for None {
	const ACCESS: self::LockAccess = LockAccess::None;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shared {}
impl sealed::Access for Shared {
	const ACCESS: self::LockAccess = LockAccess::Shared;
}
impl sealed::AccessRead for Shared {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Exclusive {}
impl sealed::Access for Exclusive {
	const ACCESS: self::LockAccess = LockAccess::Exclusive;
}
impl sealed::AccessRead for Exclusive {}
impl sealed::AccessWrite for Exclusive {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LockAccess {
	None,
	Shared,
	Exclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LockState {
	pub config: LockAccess,
	pub index: LockAccess,
	pub key: LockAccess,
	pub snapshot: LockAccess,
	pub pack: LockAccess,
}

#[serde_with::apply(Option => #[serde(default, skip_serializing_if = "Option::is_none")])]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LockMeta {
	pub hostname: Option<String>,
	pub username: Option<String>,
	pub created: DateTime<Utc>,
	pub pid: u32,
	pub uid: Option<u32>,
	pub gid: Option<u32>,
}

impl LockMeta {
	#[cfg(not(target_family = "unix"))]
	pub fn new() -> Self {
		Self {
			hostname: Some(whoami::hostname()),
			username: Some(whoami::username()),
			created: Utc::now(),
			pid: std::process::id(),
			uid: None,
			gid: None,
		}
	}

	#[cfg(target_family = "unix")]
	pub fn new() -> Self {
		Self {
			hostname: Some(whoami::hostname()),
			username: Some(whoami::username()),
			created: Utc::now(),
			pid: std::process::id(),
			uid: Some(users::get_effective_uid()),
			gid: Some(users::get_current_gid()),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
	#[serde(flatten)]
	pub meta: LockMeta,

	#[serde(flatten)]
	pub state: LockState,
}

impl RepoObject for Lock {
	const KIND: ObjectKind = ObjectKind::Lock;
}
