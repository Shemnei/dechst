use serde::{Deserialize, Serialize};

use self::chunk::ChunkerParams;
use self::compress::CompressionParams;
use self::encrypt::EncryptionParams;
use self::identify::IdentifierParams;
use self::verify::VerifierParams;

pub mod chunk;
pub mod compress;
pub mod encrypt;
pub mod format;
pub mod identify;
pub mod pipeline;
pub mod verify;

pub trait Instanciate: Copy {
	type Instance;

	fn create(&self) -> Self::Instance;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessOptions {
	pub chunker: ChunkerParams,
	pub identifier: IdentifierParams,
	pub compression: CompressionParams,
	pub encryption: EncryptionParams,
	pub verifier: VerifierParams,
}

/// TODO:
/// - TreeBuilder
/// - PackBuilder
/// - SnapshotBuilder
///
/// Source -> TreeBuilder -> ImTree -> Tree
mod build {
	mod tree {
		/// TODO:
		/// - How do we handle hardlinks (completly different file stem)
		use crate::obj::tree::node::{Node as ObjNode, NodeKind as ObjNodeKind};
		use crate::os::raw::RawOsString;

		pub type NodePathSegment = RawOsString;
		pub type NodePath = Vec<NodePathSegment>;
		pub type NodePathRef = [NodePathSegment];

		pub enum Node<I> {
			Leaf {
				node: ObjNode,
				item: I,
			},
			Branch {
				node: ObjNode,
				item: I,
				tree: Box<TreeBuilder<I>>,
			},
			UnresolvedBranch {
				sgmt: NodePathSegment,
				tree: Box<TreeBuilder<I>>,
			},
		}

		impl<I> Node<I> {
			pub fn unresolved_branch(sgmt: NodePathSegment) -> Self {
				Self::UnresolvedBranch {
					sgmt,
					tree: Box::new(Default::default()),
				}
			}

			pub fn branch(node: ObjNode, item: I) -> Self {
				Self::Branch {
					node,
					item,
					tree: Default::default(),
				}
			}

			pub fn leaf(node: ObjNode, item: I) -> Self {
				Self::Leaf { node, item }
			}

			// TODO: Replace with try_resolve -> Result
			fn resolve(&mut self, node: ObjNode, item: I) {
				let Self::UnresolvedBranch { tree, .. } = self else {
					unreachable!("Function must only be called with a unresolved branch");
				};

				let tree = std::mem::take(tree);

				let _ = std::mem::replace(self, Self::Branch { node, item, tree });
			}

			pub fn segment(&self) -> &NodePathSegment {
				match self {
					Node::Leaf { node, .. } | Node::Branch { node, .. } => &node.name,
					Node::UnresolvedBranch { sgmt, .. } => &sgmt,
				}
			}

			pub fn is_leaf(&self) -> bool {
				matches!(self, Self::Leaf { .. })
			}

			pub fn is_branch(&self) -> bool {
				matches!(self, Self::Branch { .. })
			}

			pub fn is_unresolved_branch(&self) -> bool {
				matches!(self, Self::UnresolvedBranch { .. })
			}

			pub fn is_tree(&self) -> bool {
				matches!(self, Self::Branch { .. } | Self::UnresolvedBranch { .. })
			}

			pub fn subtree(&self) -> Option<&TreeBuilder<I>> {
				match self {
					Node::Branch { tree, .. } | Node::UnresolvedBranch { tree, .. } => Some(tree),
					_ => None,
				}
			}

			pub fn subtree_mut(&mut self) -> Option<&mut TreeBuilder<I>> {
				match self {
					Node::Branch { tree, .. } | Node::UnresolvedBranch { tree, .. } => Some(tree),
					_ => None,
				}
			}
		}

		pub struct TreeBuilder<I> {
			nodes: Vec<Node<I>>,
		}

		impl<I> TreeBuilder<I> {
			// Non-recursive
			pub fn add(&mut self, path: &NodePathRef, node: ObjNode, item: I) {
				if matches!(node.kind, ObjNodeKind::Directory { .. }) {
					self.add_branch(path, node, item)
				} else {
					self.add_leaf(path, node, item)
				}
			}

			pub fn build(self) -> im::Result<im::Tree<I>> {
				todo!()
			}

			// Non-recursive
			fn add_leaf(&mut self, path: &NodePathRef, node: ObjNode, item: I) {
				let parent = self.get_or_create_tree(path);

				parent.nodes.push(Node::leaf(node, item))
			}

			// Non-recursive
			fn add_branch(&mut self, path: &NodePathRef, node: ObjNode, item: I) {
				let parent = self.get_or_create_tree(path);

				// If there is already an unresolved branch, replace it.
				let unresolved = parent
					.nodes
					.iter_mut()
					.find(|n| n.segment() == &node.name && n.is_branch());

				if let Some(unresolved) = unresolved {
					unresolved.resolve(node, item);
				} else {
					parent.nodes.push(Node::branch(node, item))
				}
			}

			// Recursive
			fn get_or_create_tree(&mut self, path: &NodePathRef) -> &mut TreeBuilder<I> {
				let [head, tail @ .. ] = path else  {
					return self;
				};

				// Search for existing subtree
				/* Does currently not work with the borrow checker
				let tree = self
					.nodes
					.iter_mut()
					.filter(|n| n.segment() == head)
					.find_map(|n| n.subtree_mut());

				let tree = if let Some(tree) = tree {
					tree
				} else {
					// Create unresolved subtree
					let branch = Node::new_unresolved_branch(head.to_owned());
					self.nodes.push(branch);
					self.nodes
						.last_mut()
						.expect("Empty nodes even though one was pushed")
				};
				*/

				// Search for existing subtree
				let idx = self
					.nodes
					.iter()
					.position(|n| n.segment() == head && n.is_tree());

				let idx = if let Some(idx) = idx {
					idx
				} else {
					// Create unresolved subtree
					let branch = Node::unresolved_branch(head.to_owned());
					self.nodes.push(branch);
					self.nodes.len() - 1
				};

				let tree = self.nodes[idx].subtree_mut().expect("Node to be a subtree");

				tree.get_or_create_tree(tail)
			}
		}

		impl<I> Default for TreeBuilder<I> {
			fn default() -> Self {
				Self {
					nodes: Default::default(),
				}
			}
		}

		pub mod im {
			use std::fmt;

			/// TODO:
			/// - How do we handle hardlinks (completly different file stem)
			use crate::obj::tree::node::{Node as ObjNode, NodeKind as ObjNodeKind};
			use crate::os::raw::RawOsString;

			pub type NodePathSegment = RawOsString;

			#[derive(Debug)]
			pub enum TreeError {}

			impl fmt::Display for TreeError {
				fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
					fmt::Debug::fmt(&self, f)
				}
			}

			impl std::error::Error for TreeError {}

			pub type Result<T, E = TreeError> = std::result::Result<T, E>;

			pub enum Node<I> {
				Leaf {
					node: ObjNode,
					item: I,
				},
				Branch {
					node: ObjNode,
					item: I,
					tree: Box<Tree<I>>,
				},
			}

			pub struct Tree<I> {
				nodes: Vec<Node<I>>,
			}

			impl<I> Tree<I> {}
		}
	}
}
