use std::num::NonZeroU32;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::Id;
use crate::obj::{ObjectKind, RepoObject};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub supersedes: Option<Vec<Id>>,
	pub packs: Vec<PackEntry>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub delete: Vec<Id>,
}

impl RepoObject for Index {
	const KIND: ObjectKind = ObjectKind::Index;
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackEntry {
	pub id: Id,
	pub blobs: Vec<BlobEntry>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub time: Option<DateTime<Utc>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub size: Option<NonZeroU32>,
}

impl PackEntry {}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobEntry {
	pub id: Id,
	pub kind: (),
	pub offset: u32,
	pub processed_len: u32,
	pub unprocessed_len: u32,
}
