/// TODO
/// - Move len from Metadata into generic struct
use serde::{Deserialize, Serialize};

pub mod raw;
pub mod unix;
pub mod windows;

pub mod generic {
	// TODO: Implement
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum User {
	Unix(unix::User),
	Windows(windows::User),
}

impl From<unix::User> for User {
	fn from(value: unix::User) -> Self {
		Self::Unix(value)
	}
}

impl From<windows::User> for User {
	fn from(value: windows::User) -> Self {
		Self::Windows(value)
	}
}

#[cfg(any(target_family = "unix", target_family = "windows"))]
impl Default for User {
	#[cfg(target_family = "unix")]
	fn default() -> Self {
		Self::Unix(unix::User::default())
	}

	#[cfg(target_family = "windows")]
	fn default() -> Self {
		Self::Windows(windows::User::default())
	}
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Metadata {
	Unix(unix::Metadata),
	Windows(windows::Metadata),
}

#[cfg(target_family = "unix")]
impl Default for Metadata {
	fn default() -> Self {
		Self::Unix(Default::default())
	}
}

#[cfg(target_family = "windows")]
impl Default for Metadata {
	fn default() -> Self {
		Self::Windows(Default::default())
	}
}

impl From<unix::Metadata> for Metadata {
	fn from(value: unix::Metadata) -> Self {
		Self::Unix(value)
	}
}

impl From<windows::Metadata> for Metadata {
	fn from(value: windows::Metadata) -> Self {
		Self::Windows(value)
	}
}
