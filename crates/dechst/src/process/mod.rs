use serde::{Deserialize, Serialize};

use self::compress::CompressionParams;
use self::encrypt::EncryptionParams;
use self::identify::IdentifierParams;
use self::verify::VerifierParams;

pub mod chunk;
pub mod compress;
pub mod encrypt;
pub mod format;
pub mod identify;
pub mod pipeline;
pub mod verify;

pub trait Instanciate: Copy {
	type Instance;

	fn create(&self) -> Self::Instance;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessOptions {
	pub identifier: IdentifierParams,
	pub compression: CompressionParams,
	pub encryption: EncryptionParams,
	pub verifier: VerifierParams,
}
