use std::fmt;

use serde::{Deserialize, Serialize};

use crate::id::Id;
use crate::obj::key::Key;

#[cfg(not(any(feature = "blake3")))]
compile_error!("At least one identifier feature must be active");

#[derive(Debug, Clone, Copy)]
pub enum IdentifyError {
	Unsupported {
		identifier: Identifier,
		feature: &'static str,
	},
}

impl fmt::Display for IdentifyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unsupported {
				identifier,
				feature,
			} => write!(
				f,
				"Identifier `{}` is not supported (to enable, re-compile with the feature `{}` \
				 enabled)",
				identifier, feature
			),
		}
	}
}

impl ::std::error::Error for IdentifyError {}

type Result<T> = ::std::result::Result<T, IdentifyError>;

pub trait Identify {
	fn identify(&self, key: &Key, data: &[u8]) -> Result<Id>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Identifier {
	Blake3,
}

impl Identifier {
	fn _identify(&self, key: &[u8], bytes: &[u8]) -> Result<Id> {
		match self {
			Self::Blake3 => {
				#[cfg(feature = "blake3")]
				{
					let hash = blake3::hash(&bytes);
					Ok(Id::from_bytes(hash.as_bytes()))
				}
				#[cfg(not(feature = "blake3"))]
				{
					Err(IdentifyError::Unsupported {
						identifier: *self,
						feature: "identifier-blake3",
					})
				}
			}
		}
	}
}

impl fmt::Display for Identifier {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Blake3 => f.write_str("Blake3"),
		}
	}
}

impl Identify for Identifier {
	fn identify(&self, key: &Key, bytes: &[u8]) -> Result<Id> {
		self._identify(key.bytes().identify_key(), bytes)
	}
}
