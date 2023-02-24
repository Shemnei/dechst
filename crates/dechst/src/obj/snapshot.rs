use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::Id;
use crate::obj::{ObjectKind, RepoObject};
use crate::os::User;

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
	pub time: DateTime<Utc>,
	pub parent: Option<Id>,
	pub tree: Id,
	pub user: User,
}

impl RepoObject for Snapshot {
	const KIND: ObjectKind = ObjectKind::Snapshot;
}
