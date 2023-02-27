use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::obj::{ObjectKind, RepoObject};
use crate::os::User;
use crate::repo::marker::LockMarker;

pub(crate) mod sealed {
	pub trait Access {
		const ACCESS: super::LockAccess;
	}

	pub trait AccessShared: Access {}
	pub trait AccessExclusive: AccessShared {}
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
impl sealed::AccessShared for Shared {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Exclusive {}
impl sealed::Access for Exclusive {
	const ACCESS: self::LockAccess = LockAccess::Exclusive;
}
impl sealed::AccessShared for Exclusive {}
impl sealed::AccessExclusive for Exclusive {}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LockAccess {
	None,
	Shared,
	Exclusive,
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LockState {
	pub config: LockAccess,
	pub index: LockAccess,
	pub key: LockAccess,
	pub snapshot: LockAccess,
	pub pack: LockAccess,
}

impl<CONFIG, INDEX, KEY, SNAPSHOT, PACK> From<LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, PACK>>
	for LockState
where
	CONFIG: sealed::Access,
	INDEX: sealed::Access,
	KEY: sealed::Access,
	SNAPSHOT: sealed::Access,
	PACK: sealed::Access,
{
	fn from(_: LockMarker<CONFIG, INDEX, KEY, SNAPSHOT, PACK>) -> Self {
		Self {
			config: CONFIG::ACCESS,
			index: INDEX::ACCESS,
			key: KEY::ACCESS,
			snapshot: SNAPSHOT::ACCESS,
			pack: PACK::ACCESS,
		}
	}
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LockMeta {
	pub user: User,
	pub created: DateTime<Utc>,
	pub pid: u32,
}

impl LockMeta {
	#[cfg(not(target_family = "unix"))]
	pub fn new() -> Self {
		Self {
			user: Default::default(),
			created: Utc::now(),
			pid: std::process::id(),
		}
	}

	#[cfg(target_family = "unix")]
	pub fn new() -> Self {
		Self {
			user: Default::default(),
			created: Utc::now(),
			pid: std::process::id(),
		}
	}
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
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
