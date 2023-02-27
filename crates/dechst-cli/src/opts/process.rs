use clap::{Args, ValueEnum};
use dechst::process::{chunk, compress, encrypt, identify, verify};
use merge::Merge;
use serde::{Deserialize, Serialize};

// TODO: Implement properly as soon as <https://github.com/clap-rs/clap/issues/2621> is done
// Alternative:
//	Have process option strings: --chunker "fastcdc=min-size=44;max-size=10"

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum Chunker {
	#[default]
	FastCdc,
}

impl From<Chunker> for chunk::ChunkerParams {
	fn from(value: Chunker) -> Self {
		match value {
			Chunker::FastCdc {} => chunk::ChunkerParams::FastCdc(chunk::FastCdc::default()),
		}
	}
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum Identifier {
	#[default]
	Blake3,
}

impl From<Identifier> for identify::IdentifierParams {
	fn from(value: Identifier) -> Self {
		match value {
			Identifier::Blake3 => identify::IdentifierParams::Blake3,
		}
	}
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum Compression {
	None,
	#[default]
	Brotli,
}

impl From<Compression> for compress::CompressionParams {
	fn from(value: Compression) -> Self {
		match value {
			Compression::None => compress::CompressionParams::None,
			Compression::Brotli => compress::CompressionParams::Brotli,
		}
	}
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum Encryption {
	None,
	#[default]
	ChaCha20,
}

impl From<Encryption> for encrypt::EncryptionParams {
	fn from(value: Encryption) -> Self {
		match value {
			Encryption::None => encrypt::EncryptionParams::None,
			Encryption::ChaCha20 => encrypt::EncryptionParams::ChaCha20,
		}
	}
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum Verifier {
	None,
	#[default]
	Blake3,
}

impl From<Verifier> for verify::VerifierParams {
	fn from(value: Verifier) -> Self {
		match value {
			Verifier::None => verify::VerifierParams::None,
			Verifier::Blake3 => verify::VerifierParams::Blake3,
		}
	}
}

#[derive(Default, Debug, Args, Serialize, Deserialize, Merge)]
#[serde(default, rename_all = "kebab-case")]
pub struct RepoProcessOpts {
	#[arg(value_enum, long, global = true, env = "DECHST_PROCESS_CHUNKER")]
	pub chunker: Option<Chunker>,

	#[arg(value_enum, long, global = true, env = "DECHST_PROCESS_IDENTIFIER")]
	pub identifier: Option<Identifier>,
}

impl RepoProcessOpts {
	pub fn recommended() -> Self {
		Self {
			chunker: Some(Chunker::default()),
			identifier: Some(Identifier::default()),
		}
	}
}

#[derive(Default, Debug, Args, Serialize, Deserialize, Merge)]
#[serde(default, rename_all = "kebab-case")]
pub struct ChunkProcessOpts {
	#[arg(value_enum, long, global = true, env = "DECHST_PROCESS_COMPRESSION")]
	pub compression: Option<Compression>,

	#[arg(value_enum, long, global = true, env = "DECHST_PROCESS_ENCRYPTION")]
	pub encryption: Option<Encryption>,

	#[arg(value_enum, long, global = true, env = "DECHST_PROCESS_VERIFIER")]
	pub verifier: Option<Verifier>,
}

impl ChunkProcessOpts {
	pub fn recommended() -> Self {
		Self {
			compression: Some(Compression::default()),
			encryption: Some(Encryption::default()),
			verifier: Some(Verifier::default()),
		}
	}
}

#[derive(Default, Debug, Args, Serialize, Deserialize, Merge)]
#[serde(default, rename_all = "kebab-case")]
pub struct ProcessOpts {
	#[serde(flatten)]
	#[command(flatten, next_help_heading = "REPOSITORY PROCESS OPTIONS")]
	pub repo: RepoProcessOpts,

	#[serde(flatten)]
	#[command(flatten, next_help_heading = "CHUNK PROCESS OPTIONS")]
	pub chunk: ChunkProcessOpts,
}

impl ProcessOpts {
	pub fn recommended() -> Self {
		Self {
			repo: RepoProcessOpts::recommended(),
			chunk: ChunkProcessOpts::recommended(),
		}
	}
}
