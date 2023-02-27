use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::Id;
use crate::obj::{ObjectKind, RepoObject};
use crate::os::raw::RawOsString;
use crate::os::User;

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserData(HashMap<String, String>);

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
	pub root: RawOsString,
	pub tags: Vec<String>,
	pub user_data: UserData,

	pub name: Option<String>,
	pub description: Option<String>,
	pub id: Id,
}

impl Default for Snapshot {
	fn default() -> Self {
		let (parent, tree, user, root, tags, user_data, name, description, id) = <_>::default();

		Self {
			time: Utc::now(),
			parent,
			tree,
			user,
			root,
			tags,
			user_data,
			name,
			description,
			id,
		}
	}
}

impl RepoObject for Snapshot {
	const KIND: ObjectKind = ObjectKind::Snapshot;
}
