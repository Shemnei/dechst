use std::fmt;

use serde::{Deserialize, Serialize};

use super::Instanciate;
use crate::id::Id;
use crate::obj::key::Key;

#[cfg(not(any(feature = "identifier-blake3")))]
compile_error!("At least one identifier feature must be active");

#[derive(Debug)]
pub enum IdentifyError {
	Unsupported {
		identifier: String,
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

pub type Result<T, E = IdentifyError> = ::std::result::Result<T, E>;

pub trait Identify {
	fn identify(&self, key: &Key, data: &[u8]) -> Result<Id>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentifierParams {
	Blake3,
}

impl Instanciate for IdentifierParams {
	type Instance = Identifier;

	fn create(&self) -> Self::Instance {
		match self {
			Self::Blake3 => Identifier::Blake3,
		}
	}
}

#[allow(missing_copy_implementations)]
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Identifier {
	Blake3,
}

impl Identifier {
	fn _identify(&self, _key: &[u8], bytes: &[u8]) -> Result<Id> {
		match self {
			Self::Blake3 => {
				#[cfg(feature = "identifier-blake3")]
				{
					let hash = blake3::hash(&bytes);
					Ok(Id::from_bytes(hash.as_bytes()))
				}
				#[cfg(not(feature = "identifier-blake3"))]
				{
					Err(IdentifyError::Unsupported {
						identifier: format!("{self}"),
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
