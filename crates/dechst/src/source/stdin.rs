use std::ffi::OsString;
use std::io::{BufReader, Stdin};
use std::iter::Once;

use super::{Item, Source};
use crate::obj::tree::node::{Node, NodeKind};
use crate::path::Segment;

#[derive(Debug, Clone, Copy)]
pub struct StdinItem;

impl Item for StdinItem {
	fn can_descend(&self) -> bool {
		false
	}
}

#[derive(Debug, Clone, Copy)]
pub struct StdinSource;

impl Source for StdinSource {
	type Error = std::io::Error;
	type Item = StdinItem;
	type Iter = Once<Result<Self::Item, Self::Error>>;
	type Read = BufReader<Stdin>;

	fn iter(&self, item: Option<&Self::Item>) -> Result<Self::Iter, Self::Error> {
		Ok(std::iter::once(Ok(StdinItem)))
	}

	fn read(&self, item: &Self::Item) -> Result<Self::Read, Self::Error> {
		let stdin = std::io::stdin();
		Ok(BufReader::new(stdin))
	}

	fn node(&self, item: &Self::Item) -> Result<Node, Self::Error> {
		let meta = crate::os::Metadata::default();
		let kind = NodeKind::file();
		let name = Segment::from(OsString::from("stdin"));

		let node = Node { name, kind, meta };

		Ok(node)
	}
}
