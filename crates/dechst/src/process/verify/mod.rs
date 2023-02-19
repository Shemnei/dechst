use std::{cmp, fmt};

use serde::{Deserialize, Serialize};

use crate::obj::key::Key;

#[derive(Debug, Clone, Copy)]
pub enum VerifyError {
	Unsupported {
		identifier: Verifier,
		feature: &'static str,
	},
}

impl fmt::Display for VerifyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unsupported {
				identifier,
				feature,
			} => write!(
				f,
				"Verifier `{}` is not supported (to enable, re-compile with the feature `{}` \
				 enabled)",
				identifier, feature
			),
		}
	}
}

impl ::std::error::Error for VerifyError {}

type Result<T> = ::std::result::Result<T, VerifyError>;

pub trait Verify {
	fn tag(&self, key: &Key, bytes: &[u8]) -> Result<Vec<u8>>;
	fn verify(&self, key: &Key, tag: &[u8], bytes: &[u8]) -> Result<bool>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Verifier {
	No,
	Blake3,
}

impl Verifier {
	fn _tag(&self, key: &[u8], bytes: &[u8]) -> Result<Vec<u8>> {
		match self {
			Self::No => Ok(vec![]),
			Self::Blake3 => {
				#[cfg(feature = "blake3")]
				{
					let mut tmp_key = [0_u8; 32];
					let len = cmp::min(32, key.len());
					tmp_key[..len].copy_from_slice(&key[..len]);

					Ok(blake3::keyed_hash(&tmp_key, bytes).as_bytes().to_vec())
				}
				#[cfg(not(feature = "blake3"))]
				{
					Err(VerifyError::Unsupported {
						identifier: *self,
						feature: "verifier-blake3",
					})
				}
			}
		}
	}

	fn _verify(&self, key: &[u8], tag: &[u8], bytes: &[u8]) -> Result<bool> {
		match self {
			Self::No => Ok(tag.is_empty()),
			Self::Blake3 => {
				#[cfg(feature = "blake3")]
				{
					let mut tmp_hash = [0_u8; 32];
					tmp_hash.copy_from_slice(&tag[..32]);

					let mut tmp_key = [0_u8; 32];
					let len = cmp::min(32, key.len());
					tmp_key[..len].copy_from_slice(&key[..len]);

					let input_hash = blake3::Hash::from(tmp_hash);
					let output_hash = blake3::keyed_hash(&tmp_key, bytes);

					Ok(output_hash.eq(&input_hash))
				}
				#[cfg(not(feature = "blake3"))]
				{
					Err(VerifyError::Unsupported {
						identifier: *self,
						feature: "verifier-blake3",
					})
				}
			}
		}
	}
}

impl Verify for Verifier {
	fn tag(&self, key: &Key, bytes: &[u8]) -> Result<Vec<u8>> {
		self._tag(key.bytes().verify_key(), bytes)
	}

	fn verify(&self, key: &Key, tag: &[u8], bytes: &[u8]) -> Result<bool> {
		self._verify(key.bytes().verify_key(), tag, bytes)
	}
}

impl fmt::Display for Verifier {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::No => f.write_str("None"),
			Self::Blake3 => f.write_str("Blake3"),
		}
	}
}
