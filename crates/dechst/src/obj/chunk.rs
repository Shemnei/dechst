use serde::{Deserialize, Serialize};

use crate::obj::key::Key;
use crate::process::compress::{Compress as _, CompressError, Compression};
use crate::process::encrypt::{Encrypt as _, EncryptError, Encryption};
use crate::process::format::{Format, Formatter};
use crate::process::verify::{Verifier, Verify as _, VerifyError};

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Serialize, Deserialize)]
pub struct CompressedChunk {
	#[serde(with = "serde_bytes")]
	pub bytes: Vec<u8>,
	pub compression: Compression,
}

impl CompressedChunk {
	pub fn compress(compression: Compression, bytes: &[u8]) -> Result<Self, CompressError> {
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
		encryption: Encryption,
	) -> Result<EncryptedChunk, EncryptError> {
		let bytes = Formatter::Cbor.format(&self).unwrap();
		EncryptedChunk::encrypt(key, encryption, &bytes)
	}
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedChunk {
	#[serde(with = "serde_bytes")]
	pub bytes: Vec<u8>,
	pub encryption: Encryption,
}

impl EncryptedChunk {
	fn encrypt(
		key: &Key,
		encryption: Encryption,
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

		Ok(Formatter::Cbor.parse(&decrypted).unwrap())
	}

	pub fn tag(self, key: &Key, verifier: Verifier) -> Result<TaggedChunk, VerifyError> {
		let bytes = Formatter::Cbor.format(&self).unwrap();
		TaggedChunk::tag(key, verifier, bytes)
	}
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, Serialize, Deserialize)]
pub struct TaggedChunk {
	#[serde(with = "serde_bytes")]
	pub bytes: Vec<u8>,
	#[serde(with = "serde_bytes")]
	pub tag: Vec<u8>,
	pub verifier: Verifier,
}

impl TaggedChunk {
	fn tag(key: &Key, verifier: Verifier, bytes: Vec<u8>) -> Result<Self, VerifyError> {
		let tag = verifier.tag(key, &bytes)?;

		Ok(Self {
			bytes,
			tag,
			verifier,
		})
	}

	pub fn verify(self, key: &Key) -> Result<EncryptedChunk, VerifyError> {
		self.verifier.verify(key, &self.tag, &self.bytes)?;

		Ok(Formatter::Cbor.parse(&self.bytes).unwrap())
	}
}
