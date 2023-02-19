use serde::{Deserialize, Serialize};

use self::compress::Compression;
use self::encrypt::Encryption;
use self::identify::Identifier;
use self::verify::Verifier;

pub mod chunk;
pub mod compress;
pub mod encrypt;
pub mod identify;
pub mod pipeline;
pub mod verify;
pub mod format;

/// TODO: format

pub trait Instanciate: Copy {
	type Instance;

	fn create(&self) -> Self::Instance;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessOptions {
	pub identifier: Identifier,
	pub compression: Compression,
	pub encryption: Encryption,
	pub verifier: Verifier,
}
