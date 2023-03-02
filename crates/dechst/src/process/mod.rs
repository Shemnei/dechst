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
///
///
/// # Idea 1
/// workload() -> Vec<Workload>;
/// resolve(node_id, ids);
///
/// Workload {
///		nodes: Vec<Node>,
///		OR
///		tree: Tree,
/// }
///
/// # Idea 2
///
/// imtree.leaves().for_each({..; imtree.resolve(&node, ids)});
/// imtree.branches().for_each({});
mod build {
	mod tree {
		use std::fmt;

		/// TODO:
		/// - How do we handle hardlinks (completly different file stem)
		use crate::{
			obj::tree::node::{Node as ObjNode, NodeKind as ObjNodeKind},
			path::{Path, Segment},
		};

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
				sgmt: Segment,
				tree: Box<TreeBuilder<I>>,
			},
		}

		impl<I> Node<I> {
			pub fn unresolved_branch(sgmt: Segment) -> Self {
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

			pub fn segment(&self) -> &Segment {
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

		impl<I> fmt::Debug for Node<I> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				match self {
					Self::Leaf { node, .. } => f.debug_struct("Leaf").field("node", node).finish(),
					Self::Branch { node, tree, .. } => f
						.debug_struct("Branch")
						.field("node", node)
						.field("tree", tree)
						.finish(),
					Self::UnresolvedBranch { sgmt, tree } => f
						.debug_struct("UnresolvedBranch")
						.field("sgmt", sgmt)
						.field("tree", tree)
						.finish(),
				}
			}
		}

		pub struct TreeBuilder<I> {
			nodes: Vec<Node<I>>,
		}

		impl<I> TreeBuilder<I> {
			// Non-recursive
			pub fn add(&mut self, path: &Path, node: ObjNode, item: I) {
				if matches!(node.kind, ObjNodeKind::Directory { .. }) {
					self.add_branch(path, node, item)
				} else {
					self.add_leaf(path, node, item)
				}
			}

			pub fn build(self) -> im::Result<im::Tree<I>, I> {
				im::Tree::try_from_builder(self)
			}

			// Non-recursive
			fn add_leaf(&mut self, path: &Path, node: ObjNode, item: I) {
				let parent = self.get_or_create_tree(path);

				parent.nodes.push(Node::leaf(node, item))
			}

			// Non-recursive
			fn add_branch(&mut self, path: &Path, node: ObjNode, item: I) {
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
			fn get_or_create_tree(&mut self, path: &Path) -> &mut TreeBuilder<I> {
				let Some((head, tail)) = path.split_head() else {
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

		impl<I> fmt::Debug for TreeBuilder<I> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				f.debug_struct("TreeBuilder")
					.field("nodes", &self.nodes)
					.finish()
			}
		}

		pub mod im {
			/// TODO
			/// - Options:
			///		- Keep/remove empty dirs
			use std::fmt;

			use super::TreeBuilder;
			/// TODO:
			/// - How do we handle hardlinks (completly different file stem)
			use crate::obj::tree::node::{Node as ObjNode, NodeKind as ObjNodeKind};
			use crate::path::{PathBuf, Segment};

			#[derive(Debug)]
			pub enum TreeErrorKind {
				DuplicateNode,
				UnresolvedBranch,
			}

			pub struct TreeError<I> {
				kind: TreeErrorKind,
				node: super::Node<I>,
				path: PathBuf,
			}

			impl<I> TreeError<I> {
				fn duplicate(path: PathBuf, node: super::Node<I>) -> Self {
					Self {
						kind: TreeErrorKind::DuplicateNode,
						node,
						path,
					}
				}

				fn unresolved(path: PathBuf, node: super::Node<I>) -> Self {
					Self {
						kind: TreeErrorKind::UnresolvedBranch,
						node,
						path,
					}
				}
			}

			impl<I> fmt::Debug for TreeError<I> {
				fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
					f.debug_struct("TreeError")
						.field("kind", &self.kind)
						.field("node", &self.node)
						.field("path", &self.path)
						.finish()
				}
			}

			impl<I> fmt::Display for TreeError<I> {
				fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
					fmt::Debug::fmt(&self, f)
				}
			}

			impl<I> std::error::Error for TreeError<I> {}

			pub type Result<T, I> = std::result::Result<T, TreeError<I>>;

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

			impl<I> Node<I> {
				pub fn segment(&self) -> &Segment {
					match self {
						Node::Leaf { node, .. } | Node::Branch { node, .. } => &node.name,
					}
				}

				pub fn is_leaf(&self) -> bool {
					matches!(self, Self::Leaf { .. })
				}

				pub fn is_branch(&self) -> bool {
					matches!(self, Self::Branch { .. })
				}

				pub fn subtree(&self) -> Option<&Tree<I>> {
					match self {
						Node::Branch { tree, .. } => Some(tree),
						_ => None,
					}
				}

				pub fn subtree_mut(&mut self) -> Option<&mut Tree<I>> {
					match self {
						Node::Branch { tree, .. } => Some(tree),
						_ => None,
					}
				}
			}

			impl<I> fmt::Debug for Node<I> {
				fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
					match self {
						Self::Leaf { node, item } => {
							f.debug_struct("Leaf").field("node", node).finish()
						}
						Self::Branch { node, item, tree } => f
							.debug_struct("Branch")
							.field("node", node)
							.field("tree", tree)
							.finish(),
					}
				}
			}

			pub struct Tree<I> {
				nodes: Vec<Node<I>>,
			}

			/// Checks
			/// - Duplicate same nodes?
			/// - Unresolved branches
			impl<I> Tree<I> {
				pub fn try_from_builder(mut builder: TreeBuilder<I>) -> Result<Self, I> {
					let (_, tree) = tree_from_builder(PathBuf::new(), &mut builder)?;
					Ok(tree)
				}

				fn with_capacity(capacity: usize) -> Self {
					Self {
						nodes: Vec::with_capacity(capacity),
					}
				}
			}

			impl<I> Default for Tree<I> {
				fn default() -> Self {
					Self {
						nodes: Default::default(),
					}
				}
			}

			fn node_exists<I>(tree: &Tree<I>, node: &super::Node<I>) -> bool {
				tree.nodes.iter().any(|n| n.segment() == node.segment())
			}

			fn tree_from_builder<I>(
				mut path: PathBuf,
				layer: &mut TreeBuilder<I>,
			) -> Result<(PathBuf, Tree<I>), I> {
				let mut tree = Tree::with_capacity(layer.nodes.len());

				for node in layer.nodes.drain(..) {
					if node_exists(&tree, &node) {
						return Err(TreeError::duplicate(path, node));
					}

					let node = match node {
						super::Node::Leaf { node, item } => Node::Leaf { node, item },
						super::Node::Branch {
							node,
							item,
							mut tree,
						} => {
							let subtree = {
								// Push path segment
								path.push(node.name.clone());
								let (rpath, subtree) = tree_from_builder(path, &mut tree)?;
								// Return path back into variable
								path = rpath;
								// Pop segment
								let _ = path
									.pop()
									.expect("A value was pushed but could not be pop'd");

								Box::new(subtree)
							};

							Node::Branch {
								node,
								item,
								tree: subtree,
							}
						}
						super::Node::UnresolvedBranch { .. } => {
							return Err(TreeError::unresolved(path, node))
						}
					};

					tree.nodes.push(node);
				}

				Ok((path, tree))
			}

			impl<I> fmt::Debug for Tree<I> {
				fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
					f.debug_struct("Tree").field("nodes", &self.nodes).finish()
				}
			}
		}

		#[cfg(test)]
		#[allow(unused_imports)]
		mod test {
			use std::time::Instant;

			use super::*;
			use crate::path::PathBuf;
			use crate::source::fs::FsSource;
			use crate::source::stdin::StdinSource;
			use crate::source::{Item, Source};

			fn count_nodes<I>(tree: &TreeBuilder<I>) -> usize {
				let mut count = 0;

				for node in &tree.nodes {
					count += 1;

					if let Some(subtree) = node.subtree() {
						count += count_nodes(subtree);
					}
				}

				count
			}

			#[test]
			fn tree_build() -> Result<(), Box<dyn std::error::Error>> {
				fn build<S>(
					source: &S,
					path: &mut PathBuf,
					tree: &mut TreeBuilder<S::Item>,
					item: Option<&S::Item>,
				) -> Result<(), Box<dyn std::error::Error>>
				where
					S: Source,
					S::Error: 'static,
				{
					let Ok(iter) = source.iter(item) else {
						eprintln!("Failed to get iter: {}", path);
						return Ok(());
					};

					for item in iter {
						let Ok(item) = item else {
							eprintln!("Failed to get item: {}", path);
							continue;
						};

						let Ok(node) = source.node(&item) else {
							eprintln!("Failed to get node: {}", path);
							continue;
						};
						let desc = item.can_descend();
						let sgmt = node.name.clone();

						tree.add(&path, node, item.clone());

						if desc {
							path.push(sgmt);
							let _ = build(source, path, tree, Some(&item))?;
							let _ = path.pop();
						}
					}

					Ok(())
				}

				let path = std::env::var_os("DECHST_TEST_PATH").unwrap();
				println!("Path: {}", path.to_string_lossy());
				let source = FsSource::new(path);
				//let source = StdinSource;
				let mut tree = TreeBuilder::default();

				let start = Instant::now();
				build(&source, &mut PathBuf::new(), &mut tree, None)?;
				let elapsed = start.elapsed();

				println!("{:#?}", tree);

				let ncount = count_nodes(&tree);
				println!("Found {} nodes", ncount);
				println!("Took {elapsed:?}");
				println!("Was {} ms/node", elapsed.as_millis() as f64 / ncount as f64);
				println!("Was {} nodes/s", ncount as f64 / elapsed.as_secs() as f64);

				let start = Instant::now();
				let tree = tree.build()?;
				let elapsed = start.elapsed();
				println!("Verified tree");
				println!("Took {elapsed:?}");

				Ok(())
			}
		}
	}
}
