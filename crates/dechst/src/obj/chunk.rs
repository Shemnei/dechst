use std::io::Cursor;

use serde::{Deserialize, Serialize};

use crate::obj::key::Key;
use crate::process::compress::{Compress as _, CompressError, CompressionParams};
use crate::process::encrypt::{Encrypt as _, EncryptError, EncryptionParams};
use crate::process::format::{Format, FormatterParams};
use crate::process::verify::{VerifierParams, Verify as _, VerifyError};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompressedChunk {
	#[serde(with = "serde_bytes")]
	pub bytes: Vec<u8>,
	pub compression: CompressionParams,
}

impl CompressedChunk {
	pub fn compress(compression: CompressionParams, bytes: &[u8]) -> Result<Self, CompressError> {
		let compressed = compression.compress(&bytes)?;

		Ok(Self {
			bytes: compressed,
			compression,
		})
	}

	pub fn decompress(self) -> Result<Vec<u8>, CompressError> {
		self.compression.decompress(&self.bytes)
	}

	pub fn encrypt(
		self,
		key: &Key,
		encryption: EncryptionParams,
	) -> Result<EncryptedChunk, EncryptError> {
		let bytes = FormatterParams::Cbor.format(&self).unwrap();
		EncryptedChunk::encrypt(key, encryption, &bytes)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedChunk {
	#[serde(with = "serde_bytes")]
	pub bytes: Vec<u8>,
	pub encryption: EncryptionParams,
}

impl EncryptedChunk {
	fn encrypt(
		key: &Key,
		encryption: EncryptionParams,
		bytes: &[u8],
	) -> Result<Self, EncryptError> {
		let encrypted = encryption.encrypt(key, bytes)?;

		Ok(Self {
			bytes: encrypted,
			encryption,
		})
	}

	pub fn decrypt(self, key: &Key) -> Result<CompressedChunk, EncryptError> {
		let decrypted = self.encryption.decrypt(key, &self.bytes)?;

		Ok(FormatterParams::Cbor.parse(&decrypted).unwrap())
	}

	pub fn tag(self, key: &Key, verifier: VerifierParams) -> Result<TaggedChunk, VerifyError> {
		let bytes = FormatterParams::Cbor.format(&self).unwrap();
		TaggedChunk::tag(key, verifier, bytes)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaggedChunk {
	#[serde(with = "serde_bytes")]
	pub bytes: Vec<u8>,
	#[serde(with = "serde_bytes")]
	pub tag: Vec<u8>,
	pub verifier: VerifierParams,
}

impl TaggedChunk {
	fn tag(key: &Key, verifier: VerifierParams, bytes: Vec<u8>) -> Result<Self, VerifyError> {
		let tag = verifier.tag(key, &bytes)?;

		Ok(Self {
			bytes,
			tag,
			verifier,
		})
	}

	pub fn verify(self, key: &Key) -> Result<EncryptedChunk, VerifyError> {
		self.verifier.verify(key, &self.tag, &self.bytes)?;

		Ok(FormatterParams::Cbor.parse(&self.bytes).unwrap())
	}
}
