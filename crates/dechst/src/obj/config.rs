use serde::{Deserialize, Serialize};

use crate::id::Id;
use crate::obj::{ObjectKind, RepoObject};

#[serde_with::apply(Option => #[serde(default, skip_serializing_if = "Option::is_none")])]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
	pub version: u32,
	pub id: Id,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			version: 1,
			id: Id::random(),
		}
	}
}

impl RepoObject for Config {
	const KIND: ObjectKind = ObjectKind::Config;
}

impl Config {
	pub fn new(version: u32, id: Id) -> Self {
		Self {
			version,
			id,
			..Self::default()
		}
	}
}
