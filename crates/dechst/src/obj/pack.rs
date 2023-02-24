use serde::{Deserialize, Serialize};

use crate::id::Id;
use crate::obj::{ObjectKind, RepoObject};

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeaderEntry {
	Data {
		processed_len: u32,
		unprocessed_len: u32,
		id: Id,
	},
	Tree {
		processed_len: u32,
		unprocessed_len: u32,
		id: Id,
	},
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackHeader {
	entries: Vec<HeaderEntry>,
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pack {
	#[serde(with = "serde_bytes")]
	blobs: Vec<u8>,
	#[serde(with = "serde_bytes")]
	header: Vec<u8>,
	header_len: u32,
}

impl RepoObject for Pack {
	const KIND: ObjectKind = ObjectKind::Pack;
}
