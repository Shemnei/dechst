pub mod node;

use serde::{Deserialize, Serialize};

use self::node::Node;

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree {
	nodes: Vec<Node>,
}

impl Tree {
	pub fn iter(&self) -> std::slice::Iter<'_, Node> {
		self.nodes.iter()
	}
}

impl IntoIterator for Tree {
	type IntoIter = std::vec::IntoIter<Node>;
	type Item = Node;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}
