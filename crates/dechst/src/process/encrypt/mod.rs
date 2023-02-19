use std::{cmp, fmt};

use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::obj::key::Key;

#[derive(Debug)]
pub enum EncryptError {
	Unsupported {
		encryption: Encryption,
		feature: &'static str,
	},
	IoError {
		source: ::std::io::Error,
		context: &'static str,
	},
}

impl fmt::Display for EncryptError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unsupported {
				encryption,
				feature,
			} => write!(
				f,
				"Encryption `{}` is not supported (to enable, re-compile with the feature `{}` \
				 enabled)",
				encryption, feature
			),
			Self::IoError { source, context } => {
				write!(f, "{}: {}", context, source)
			}
		}
	}
}

impl ::std::error::Error for EncryptError {
	fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
		match self {
			Self::IoError { source, .. } => Some(source),
			_ => None,
		}
	}
}

type Result<T> = ::std::result::Result<T, EncryptError>;

pub trait Encrypt {
	fn encrypt(&self, key: &Key, bytes: &[u8]) -> Result<Vec<u8>>;
	fn decrypt(&self, key: &Key, bytes: &[u8]) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Encryption {
	No,
	ChaCha20 { iv: [u8; 12] },
}

impl Encryption {
	#[cfg(feature = "chacha20")]
	pub fn new_chacha20() -> Self {
		let mut iv: [u8; 12] = [0; 12];
		rand::thread_rng().fill_bytes(&mut iv);
		Self::ChaCha20 { iv }
	}

	pub fn key_length(&self) -> u32 {
		match self {
			Self::No => 16,
			Self::ChaCha20 { .. } => 32,
		}
	}

	pub fn encrypt_bytes(&self, key: &[u8], bytes: &[u8]) -> Result<Vec<u8>> {
		match self {
			Self::No => Ok(bytes.into()),
			Self::ChaCha20 { iv } => {
				#[cfg(feature = "chacha20")]
				{
					use chacha20::cipher::generic_array::GenericArray;
					use chacha20::cipher::{KeyIvInit, StreamCipher};
					use chacha20::ChaCha20;

					let mut proper_key: [u8; 32] = [0; 32];
					proper_key[..cmp::min(key.len(), 32)]
						.clone_from_slice(&key[..cmp::min(key.len(), 32)]);

					let key = GenericArray::from_slice(key);
					let iv = GenericArray::from_slice(&iv[..]);
					let mut encryptor = ChaCha20::new(key, iv);

					let mut final_result = bytes.to_vec();
					encryptor.apply_keystream(&mut final_result);

					proper_key.zeroize();

					Ok(final_result)
				}
				#[cfg(not(feature = "chacha20"))]
				{
					Err(IdentifyError::Unsupported {
						encryption: *self,
						feature: "encryption-chacha20",
					})
				}
			}
		}
	}

	pub fn decrypt_bytes(&self, key: &[u8], bytes: &[u8]) -> Result<Vec<u8>> {
		match self {
			Self::No => Ok(bytes.into()),
			Self::ChaCha20 { iv } => {
				#[cfg(feature = "chacha20")]
				{
					use chacha20::cipher::generic_array::GenericArray;
					use chacha20::cipher::{KeyIvInit, StreamCipher};
					use chacha20::ChaCha20;

					let mut proper_key: [u8; 32] = [0; 32];
					proper_key[..cmp::min(key.len(), 32)]
						.clone_from_slice(&key[..cmp::min(key.len(), 32)]);

					let key = GenericArray::from_slice(key);
					let iv = GenericArray::from_slice(&iv[..]);
					let mut decryptor = ChaCha20::new(key, iv);

					let mut final_result = bytes.to_vec();
					decryptor.apply_keystream(&mut final_result);

					proper_key.zeroize();

					Ok(final_result)
				}
				#[cfg(not(feature = "chacha20"))]
				{
					Err(IdentifyError::Unsupported {
						encryption: *self,
						feature: "encryption-chacha20",
					})
				}
			}
		}
	}
}

impl Encrypt for Encryption {
	fn encrypt(&self, key: &Key, bytes: &[u8]) -> Result<Vec<u8>> {
		self.encrypt_bytes(key.bytes().encrypt_key(), bytes)
	}

	fn decrypt(&self, key: &Key, bytes: &[u8]) -> Result<Vec<u8>> {
		self.decrypt_bytes(key.bytes().encrypt_key(), bytes)
	}
}

impl fmt::Display for Encryption {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::No => f.write_str("None"),
			Self::ChaCha20 { .. } => f.write_str("ChaCha20"),
		}
	}
}
