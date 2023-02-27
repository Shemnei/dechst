use serde::{Deserialize, Serialize};

use crate::id::Id;
use crate::obj::{ObjectKind, RepoObject};

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlobKind {
	Tree,
	Data,
}
