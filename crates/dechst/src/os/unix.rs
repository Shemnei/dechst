use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::raw;

#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct User {
	pub hostname: Option<raw::RawOsString>,
	pub username: Option<raw::RawOsString>,
	pub uid: Option<u32>,
	pub gid: Option<u32>,
}

#[cfg(target_family = "unix")]
impl Default for User {
	fn default() -> Self {
		Self {
			hostname: Some(whoami::hostname_os().into()),
			username: Some(whoami::username_os().into()),
			uid: Some(users::get_effective_uid()),
			gid: Some(users::get_current_gid()),
		}
	}
}

#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(target_family = "unix", derive(Default))]
pub struct Permissions {
	pub mode: u32,
}

impl fmt::Debug for Permissions {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Permissions")
			.field("mode", &format!("{:o}", self.mode))
			.finish()
	}
}

#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(target_family = "unix", derive(Default))]
pub struct Times {
	pub access: Option<DateTime<Utc>>,
	// Content changed
	pub modify: Option<DateTime<Utc>>,
	// Metadata changed
	pub change: Option<DateTime<Utc>>,
	pub create: Option<DateTime<Utc>>,
}

#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(target_family = "unix", derive(Default))]
pub struct Identifier {
	pub dev: u64,
	pub ino: u64,
}

#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(target_family = "unix", derive(Default))]
pub struct Metadata {
	#[serde(flatten)]
	pub user: User,
	#[serde(flatten)]
	pub perm: Permissions,
	#[serde(flatten)]
	pub time: Times,
	#[serde(flatten)]
	pub ident: Identifier,
	pub len: u64,
}
