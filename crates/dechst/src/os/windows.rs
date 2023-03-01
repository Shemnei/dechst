use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::raw;

//https://learn.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants

#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct User {
	pub hostname: Option<raw::RawOsString>,
	pub username: Option<raw::RawOsString>,
}

#[cfg(target_family = "windows")]
impl Default for User {
	fn default() -> Self {
		Self {
			hostname: Some(whoami::hostname_os().into()),
			username: Some(whoami::username_os().into()),
		}
	}
}

#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(target_family = "windows", derive(Default))]
pub struct Permissions {
	pub attributes: u32,
}

// https://learn.microsoft.com/en-us/windows/win32/api/fileapi/ns-fileapi-win32_file_attribute_data

#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(target_family = "windows", derive(Default))]
pub struct Times {
	pub access: Option<DateTime<Utc>>,
	// Content changed
	pub modify: Option<DateTime<Utc>>,
	pub create: Option<DateTime<Utc>>,
}

#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(target_family = "windows", derive(Default))]
pub struct Identifier {
	pub volume_serial_number: Option<u32>,
	pub file_index: Option<u64>,
}

#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(target_family = "windows", derive(Default))]
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
