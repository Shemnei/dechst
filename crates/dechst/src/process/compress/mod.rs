use std::fmt;
use std::io::Cursor;

use serde::{Deserialize, Serialize};

use super::Instanciate;

#[cfg(feature = "brotli")]
const BUFFER_SIZE: usize = 4_096;
#[cfg(feature = "brotli")]
const QUALITY: u32 = 5;
#[cfg(feature = "brotli")]
const WINDOW_SIZE: u32 = 20;

#[derive(Debug)]
pub enum CompressError {
	Unsupported {
		compression: String,
		feature: &'static str,
	},
	IoError {
		source: ::std::io::Error,
		context: &'static str,
	},
}

impl fmt::Display for CompressError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unsupported {
				compression,
				feature,
			} => write!(
				f,
				"Compression `{}` is not supported (to enable, re-compile with the feature `{}` \
				 enabled)",
				compression, feature
			),
			Self::IoError { source, context } => {
				write!(f, "{}: {}", context, source)
			}
		}
	}
}

impl ::std::error::Error for CompressError {
	fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
		match self {
			Self::IoError { source, .. } => Some(source),
			_ => None,
		}
	}
}

pub type Result<T, E = CompressError> = ::std::result::Result<T, E>;

pub trait Compress {
	fn compress(&self, bytes: &[u8]) -> Result<Vec<u8>>;
	fn decompress(&self, bytes: &[u8]) -> Result<Vec<u8>>;
}

#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompressionParams {
	None,
	Brotli,
}

impl Instanciate for CompressionParams {
	type Instance = Compression;

	fn create(&self) -> Self::Instance {
		match self {
			Self::None => Compression::None,
			Self::Brotli => Compression::Brotli,
		}
	}
}

#[allow(missing_copy_implementations)]
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Compression {
	None,
	Brotli,
}

impl Compress for Compression {
	fn compress(&self, bytes: &[u8]) -> Result<Vec<u8>> {
		match self {
			Self::None => Ok(bytes.into()),
			Self::Brotli => {
				#[cfg(feature = "brotli")]
				{
					use brotli::CompressorWriter;

					// In most cases the output should have less bytes than the input
					let out = Vec::with_capacity(bytes.len());
					let write = Cursor::new(out);
					let mut encoder =
						CompressorWriter::new(write, BUFFER_SIZE, QUALITY, WINDOW_SIZE);

					let mut read = Cursor::new(bytes);

					if let Err(err) = ::std::io::copy(&mut read, &mut encoder) {
						return Err(CompressError::IoError {
							source: err,
							context: "Compression(brotli) failed to compress bytes",
						});
					}

					Ok(encoder.into_inner().into_inner())
				}
				#[cfg(not(feature = "brotli"))]
				{
					Err(CompressError::Unsupported {
						compression: format!("{self}"),
						feature: "compression-brotli",
					})
				}
			}
		}
	}

	fn decompress(&self, bytes: &[u8]) -> Result<Vec<u8>> {
		match self {
			Self::None => Ok(bytes.into()),
			Self::Brotli => {
				#[cfg(feature = "brotli")]
				{
					use brotli::Decompressor;

					let mut write = Vec::with_capacity(bytes.len());

					let mut decoder = Decompressor::new(bytes, BUFFER_SIZE);

					if let Err(err) = ::std::io::copy(&mut decoder, &mut write) {
						return Err(CompressError::IoError {
							source: err,
							context: "Decompression(brotli) failed to decompress bytes",
						});
					}

					Ok(write)
				}
				#[cfg(not(feature = "brotli"))]
				{
					Err(CompressError::Unsupported {
						compression: format!("{self}"),
						feature: "compression-brotli",
					})
				}
			}
		}
	}
}

impl fmt::Display for Compression {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::None => f.write_str("None"),
			Self::Brotli => f.write_str("Brotli"),
		}
	}
}
