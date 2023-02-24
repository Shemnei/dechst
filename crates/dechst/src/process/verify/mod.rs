use std::{cmp, fmt};

use serde::{Deserialize, Serialize};

use super::Instanciate;
use crate::obj::key::Key;

#[derive(Debug)]
pub enum VerifyError {
	Unsupported {
		verifier: String,
		feature: &'static str,
	},
	VerficationFailed,
}

impl fmt::Display for VerifyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unsupported {
				verifier: identifier,
				feature,
			} => write!(
				f,
				"Verifier `{}` is not supported (to enable, re-compile with the feature `{}` \
				 enabled)",
				identifier, feature
			),
			Self::VerficationFailed => f.write_str("Failed to verify data"),
		}
	}
}

impl ::std::error::Error for VerifyError {}

pub type Result<T, E = VerifyError> = ::std::result::Result<T, E>;

pub trait Verify {
	fn tag(&self, key: &Key, bytes: &[u8]) -> Result<Vec<u8>>;
	fn verify(&self, key: &Key, tag: &[u8], bytes: &[u8]) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerifierParams {
	None,
	Blake3,
}

impl Instanciate for VerifierParams {
	type Instance = Verifier;

	fn create(&self) -> Self::Instance {
		match self {
			Self::None => Verifier::None,
			Self::Blake3 => Verifier::Blake3,
		}
	}
}

#[allow(missing_copy_implementations)]
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Verifier {
	None,
	Blake3,
}

impl Verifier {
	fn _tag(&self, key: &[u8], bytes: &[u8]) -> Result<Vec<u8>> {
		match self {
			Self::None => Ok(vec![]),
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
						verifier: format!("{self}"),
						feature: "verifier-blake3",
					})
				}
			}
		}
	}

	fn _verify(&self, key: &[u8], tag: &[u8], bytes: &[u8]) -> Result<()> {
		match self {
			Self::None => Ok(()),
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

					if output_hash == input_hash {
						Ok(())
					} else {
						Err(VerifyError::VerficationFailed)
					}
				}
				#[cfg(not(feature = "blake3"))]
				{
					Err(VerifyError::Unsupported {
						verifier: format!("{self}"),
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

	fn verify(&self, key: &Key, tag: &[u8], bytes: &[u8]) -> Result<()> {
		self._verify(key.bytes().verify_key(), tag, bytes)
	}
}

impl fmt::Display for Verifier {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::None => f.write_str("None"),
			Self::Blake3 => f.write_str("Blake3"),
		}
	}
}
