// TODO: move glue code into dechst_core
// - Chunker
// - BoxedChunkRead

/// An implementation of the `FastCdc` algorithm describes in the paper [`FastCDC: a Fast and
/// Efficient Content-Defined Chunking Approach for Data
/// Deduplication`](https://www.usenix.org/system/files/conference/atc16/atc16-paper-xia.pdf).
pub mod fastcdc;

pub mod prelude {
	pub use crate::fastcdc::{
		FastCdc, FastCdcChunker, AVG_SIZE as FASTCDC_AVG_SIZE, MAX_SIZE as FASTCDC_MAX_SIZE,
		MIN_SIZE as FASTCDC_MIN_SIZE,
	};
	pub use crate::{Algorithm, ChunkIter, ChunkRead};
}

use std::io::Read;

/// A algorithm decides when and where to split the contents from a source into
/// a chunk.
///
/// When run with the same settings, a `Algorithm` should always reproduce the
/// same chunks for a source.
pub trait Algorithm {
	type ChunkRead: ChunkRead;

	/// Constructs a new [`ChunkRead`] for this algorithm.
	fn chunk<R: Read + 'static>(&self, read: R) -> Self::ChunkRead;
}

/// The `ChunkRead` trait allows for reading chunks of bytes from a source.
///
/// When run with the same settings, a `ChunkRead` should always reproduce the
/// same chunks for a source.
pub trait ChunkRead {
	/// Constructs a buffer with preferred capacity for use in [`Self::read_chunk`].
	///
	/// If the [`Algorithm`] has a maximal chunk size, it should be used as the
	/// capacity for the buffer.
	fn prefered_buffer(&self) -> Vec<u8>;

	/// Reads a single chunk.
	/// It's function is similar to [`Read::read`](::std::io::Read::read).
	fn read_chunk(&mut self, buf: &mut Vec<u8>) -> Result<usize, ::std::io::Error>;
}

impl<C: ChunkRead + ?Sized> ChunkRead for Box<C> {
	fn prefered_buffer(&self) -> Vec<u8> {
		self.as_ref().prefered_buffer()
	}

	fn read_chunk(&mut self, buf: &mut Vec<u8>) -> Result<usize, std::io::Error> {
		self.as_mut().read_chunk(buf)
	}
}

/// An [`Iterator`](::std::iter::Iterator) over all chunks from a source.
pub struct ChunkIter<C> {
	chunk_read: C,
	buf: Vec<u8>,
}

impl<C: ChunkRead> ChunkIter<C> {
	pub fn new(chunk_read: C) -> Self {
		let buf = chunk_read.prefered_buffer();

		Self { chunk_read, buf }
	}
}

impl<C: ChunkRead> Iterator for ChunkIter<C> {
	type Item = Result<Vec<u8>, ::std::io::Error>;

	fn next(&mut self) -> Option<Self::Item> {
		let bytes_read = match self.chunk_read.read_chunk(&mut self.buf) {
			Ok(bytes_read) => bytes_read,
			Err(err) => return Some(Err(err)),
		};

		if bytes_read == 0 {
			None
		} else {
			Some(Ok(self.buf[..bytes_read].to_vec()))
		}
	}
}
