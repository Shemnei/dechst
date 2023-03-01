use crate::obj::tree::node::Node;

pub mod fs;
pub mod stdin;

pub trait Item: Clone {
	fn can_descend(&self) -> bool;
}

pub trait Source {
	type Error: std::error::Error;
	type Item: Item;
	type Read;

	type Iter: Iterator<Item = Result<Self::Item, Self::Error>>;

	fn iter(&self, item: Option<&Self::Item>) -> Result<Self::Iter, Self::Error>;
	fn read(&self, item: &Self::Item) -> Result<Self::Read, Self::Error>;
	fn node(&self, item: &Self::Item) -> Result<Node, Self::Error>;
}
