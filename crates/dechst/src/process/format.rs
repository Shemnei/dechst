use std::fmt;
use std::io::Cursor;

use super::Instanciate;

#[derive(Debug)]
pub enum FormatError {
	Unsupported {
		identifier: String,
		feature: &'static str,
	},
	Failed(Box<dyn std::error::Error + 'static + Send + Sync>),
}

impl FormatError {
	pub fn from_err<E: std::error::Error + 'static + Send + Sync>(err: E) -> Self {
		Self::Failed(Box::new(err))
	}
}

impl fmt::Display for FormatError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unsupported {
				identifier,
				feature,
			} => write!(
				f,
				"Formatter `{}` is not supported (to enable, re-compile with the feature `{}` \
				 enabled)",
				identifier, feature
			),

			Self::Failed(_) => f.write_str("Failed"),
		}
	}
}

impl ::std::error::Error for FormatError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::Failed(err) => Some(err.as_ref()),
			_ => None,
		}
	}
}

pub type Result<T, E = FormatError> = ::std::result::Result<T, E>;

pub trait Format {
	fn format<V: ?Sized + serde::ser::Serialize>(&self, value: &V) -> Result<Vec<u8>>;
	fn parse<'de, V: serde::de::Deserialize<'de>>(&self, bytes: &[u8]) -> Result<V>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FormatterParams {
	Cbor,
}

impl Instanciate for FormatterParams {
	type Instance = Formatter;

	fn create(&self) -> Self::Instance {
		match self {
			Self::Cbor => Formatter::Cbor,
		}
	}
}

#[allow(missing_copy_implementations)]
#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Formatter {
	Cbor,
}

impl fmt::Display for Formatter {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Cbor => f.write_str("Cbor"),
		}
	}
}

impl Format for Formatter {
	fn format<V: ?Sized + serde::ser::Serialize>(&self, value: &V) -> Result<Vec<u8>> {
		match self {
			Self::Cbor => {
				#[cfg(feature = "ciborium")]
				{
					let mut cur = Cursor::new(Vec::new());

					ciborium::ser::into_writer(value, &mut cur).map_err(FormatError::from_err)?;

					Ok(cur.into_inner())
				}
				#[cfg(not(feature = "ciborium"))]
				{
					Err(FormatError::Unsupported {
						identifier: format!("{self}"),
						feature: "formatter-ciborium",
					})
				}
			}
		}
	}

	fn parse<'de, V: serde::de::Deserialize<'de>>(&self, bytes: &[u8]) -> Result<V> {
		match self {
			Self::Cbor => {
				#[cfg(feature = "ciborium")]
				{
					match ciborium::de::from_reader(bytes) {
						Ok(v) => Ok(v),
						Err(e) => Err(FormatError::from_err(e)),
					}
				}
				#[cfg(not(feature = "ciborium"))]
				{
					Err(FormatError::Unsupported {
						identifier: format!("{self}"),
						feature: "formatter-ciborium",
					})
				}
			}
		}
	}
}
