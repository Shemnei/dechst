use serde::{Deserialize, Serialize};

pub mod raw {
	use std::ffi::OsString;
	use std::fmt::{self, Debug};
	use std::ops::Deref;

	use serde::de::Visitor;
	use serde::{Deserialize, Serialize};

	#[derive(Debug, Clone, PartialEq, Eq, Hash)]
	pub struct RawOsString(os_str_bytes::RawOsString);

	impl fmt::Display for RawOsString {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			let mut buf = Vec::with_capacity(self.0.raw_len());
			print_bytes::write_lossy(&mut buf, &self.0).unwrap();

			f.write_str(unsafe { std::str::from_utf8_unchecked(&buf) })
		}
	}

	impl From<OsString> for RawOsString {
		fn from(value: OsString) -> Self {
			Self(os_str_bytes::RawOsString::new(value))
		}
	}

	impl Serialize for RawOsString {
		fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer,
		{
			serializer.serialize_bytes(self.0.as_raw_bytes())
		}
	}

	impl Deref for RawOsString {
		type Target = os_str_bytes::RawOsString;

		fn deref(&self) -> &Self::Target {
			&self.0
		}
	}

	struct RawOsStringVisitor;

	impl<'de> Visitor<'de> for RawOsStringVisitor {
		type Value = RawOsString;

		fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			formatter.write_str("An byte vector")
		}

		fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			Ok(RawOsString(unsafe {
				os_str_bytes::RawOsString::from_raw_vec_unchecked(v)
			}))
		}
	}

	impl<'de> Deserialize<'de> for RawOsString {
		fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where
			D: serde::Deserializer<'de>,
		{
			deserializer.deserialize_byte_buf(RawOsStringVisitor)
		}
	}
}

pub mod unix {
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
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
	pub struct Permissions {
		mode: u32,
	}

	#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
	pub struct Times {
		access: Option<DateTime<Utc>>,
		// Content changed
		modify: Option<DateTime<Utc>>,
		// Metadata changed
		change: Option<DateTime<Utc>>,
		create: Option<DateTime<Utc>>,
	}

	#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
	pub struct Identifier {
		dev: u64,
		rdev: u64,
		ino: u64,
	}

	#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
	#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
	pub struct Metadata {
		#[serde(flatten)]
		user: User,
		#[serde(flatten)]
		perm: Permissions,
		#[serde(flatten)]
		time: Times,
		#[serde(flatten)]
		ident: Identifier,
		len: u64,
	}
}

pub mod windows {
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
	pub struct Permissions {
		attributes: u32,
	}

	// https://learn.microsoft.com/en-us/windows/win32/api/fileapi/ns-fileapi-win32_file_attribute_data

	#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
	pub struct Times {
		access: Option<DateTime<Utc>>,
		// Content changed
		modify: Option<DateTime<Utc>>,
		create: Option<DateTime<Utc>>,
	}

	#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
	pub struct Identifier {
		volume_serial_number: Option<u32>,
		file_index: Option<u64>,
	}

	#[serde_with::apply(
		Option => #[serde(default, skip_serializing_if = "Option::is_none")],
		Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
	)]
	#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
	pub struct Metadata {
		#[serde(flatten)]
		user: User,
		#[serde(flatten)]
		perm: Permissions,
		#[serde(flatten)]
		time: Times,
		#[serde(flatten)]
		ident: Identifier,
		len: u64,
	}
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
