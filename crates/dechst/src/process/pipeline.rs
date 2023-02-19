use std::fmt;

use serde::Serialize;

use super::compress::CompressError;
use super::encrypt::EncryptError;
use super::format::{Format, FormatError, FormatterParams};
use super::verify::VerifyError;
use super::{Instanciate, ProcessOptions};
use crate::obj::chunk::{CompressedChunk, TaggedChunk};
use crate::obj::key::Key;

#[derive(Debug)]
pub enum PipelineError {
	Compress(CompressError),
	Encrypt(EncryptError),
	Verify(VerifyError),
	Format(FormatError),
}

impl From<CompressError> for PipelineError {
	fn from(value: CompressError) -> Self {
		Self::Compress(value)
	}
}

impl From<EncryptError> for PipelineError {
	fn from(value: EncryptError) -> Self {
		Self::Encrypt(value)
	}
}

impl From<VerifyError> for PipelineError {
	fn from(value: VerifyError) -> Self {
		Self::Verify(value)
	}
}

impl From<FormatError> for PipelineError {
	fn from(value: FormatError) -> Self {
		Self::Format(value)
	}
}

impl fmt::Display for PipelineError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Compress(inner) => write!(f, "Compress: {inner}"),
			Self::Encrypt(inner) => write!(f, "Encrypt: {inner}"),
			Self::Verify(inner) => write!(f, "Verify: {inner}"),
			Self::Format(inner) => write!(f, "Format: {inner}"),
		}
	}
}

impl std::error::Error for PipelineError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::Compress(s) => Some(s),
			Self::Encrypt(s) => Some(s),
			Self::Verify(s) => Some(s),
			Self::Format(s) => Some(s),
		}
	}
}

pub type Result<T, E = PipelineError> = std::result::Result<T, E>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkPipeline {
	pub key: Key,
	pub opts: ProcessOptions,
}

impl ChunkPipeline {
	pub fn new(opts: ProcessOptions, key: Key) -> Self {
		Self { key, opts }
	}

	pub fn process_forward(&self, bytes: &[u8]) -> Result<Vec<u8>> {
		let tagged = CompressedChunk::compress(self.opts.compression.create(), bytes)?
			.encrypt(&self.key, self.opts.encryption.create())?
			.tag(&self.key, self.opts.verifier.create())?;

		Ok(FormatterParams::Cbor.format(&tagged)?)
	}

	pub fn process_backward(&self, bytes: &[u8]) -> Result<Vec<u8>> {
		let tagged: TaggedChunk = FormatterParams::Cbor.parse(bytes)?;

		let bytes = tagged.verify(&self.key)?.decrypt(&self.key)?.decompress()?;

		Ok(bytes)
	}
}
