use std::ffi::OsString;
use std::fmt::{self, Debug};
use std::ops::Deref;

use serde::de::Visitor;
use serde::{Deserialize, Serialize};

// TODO: Alternativly use String with escaping
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
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
