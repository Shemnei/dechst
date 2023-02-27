use std::io::{BufReader, Read};

pub use dechst_chunker::prelude::*;
use serde::{Deserialize, Serialize};

use super::Instanciate;

pub type Result<T, E = ::std::io::Error> = ::std::result::Result<T, E>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkerParams {
	FastCdc(FastCdc),
}

impl Instanciate for ChunkerParams {
	type Instance = Chunker;

	fn create(&self) -> Self::Instance {
		match self {
			Self::FastCdc(inner) => Chunker::FastCdc(*inner),
		}
	}
}

/// A collection of different chunking algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Chunker {
	/// Settings for the `FastCdc` algorithm.
	FastCdc(FastCdc),
}

impl Chunker {
	pub fn fastcdc_default() -> Self {
		Self::FastCdc(Default::default())
	}

	pub fn chunk<R: Read + 'static>(&self, read: R) -> BoxedChunkRead {
		match self {
			Self::FastCdc(fastcdc) => BoxedChunkRead::new(fastcdc.chunk(read)),
		}
	}

	pub fn chunk_buffered<R: Read + 'static>(&self, read: R) -> BoxedChunkRead {
		match self {
			Self::FastCdc(fastcdc) => BoxedChunkRead::new(fastcdc.chunk(BufReader::new(read))),
		}
	}
}

/// An boxed [`ChunkRead`] instance.
/// This type is primarily used to implement [`IntoIterator`](::std::iter::IntoIterator)
/// on it.
#[allow(missing_debug_implementations)]
pub struct BoxedChunkRead(Box<dyn ChunkRead>);

impl BoxedChunkRead {
	fn new<C: ChunkRead + 'static>(chunk_read: C) -> Self {
		Self(Box::new(chunk_read))
	}
}

impl IntoIterator for BoxedChunkRead {
	type IntoIter = ChunkIter<Box<dyn ChunkRead>>;
	type Item = Result<Vec<u8>>;

	fn into_iter(self) -> Self::IntoIter {
		ChunkIter::new(self.0)
	}
}

impl AsRef<Box<dyn ChunkRead>> for BoxedChunkRead {
	fn as_ref(&self) -> &Box<dyn ChunkRead> {
		&self.0
	}
}

impl From<BoxedChunkRead> for Box<dyn ChunkRead> {
	fn from(value: BoxedChunkRead) -> Self {
		value.0
	}
}
