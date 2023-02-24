#![feature(exit_status_error)]

pub mod command;
pub mod format;
pub mod password;
pub mod util;

use std::fmt;
use std::path::PathBuf;

use clap::{Args, Parser, ValueEnum};
use dechst::process;
use log::LevelFilter;
use merge::Merge;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use zeroize::Zeroizing;

/// Password used if the repository does not use encryption.
pub const DEFAULT_PASSWORD: &str = "";

// `CARGO_BIN_NAME` does not work, as this is the library part ...
pub const BINARY_NAME: &str = "dechst";

#[derive(Debug, Parser)]
#[command(author, about, name = "dechst", version)]
pub struct Opts {
	#[command(flatten, next_help_heading = "GLOBAL OPTIONS")]
	pub global: GlobalOpts,

	#[command(flatten, next_help_heading = "REPOSITORY OPTIONS")]
	pub repo: RepoOpts,

	#[command(subcommand)]
	pub command: command::Command,
}

#[derive(Debug, Args, Serialize, Deserialize, Merge)]
#[serde_as]
#[serde(default, rename_all = "kebab-case")]
pub struct GlobalOpts {
	#[arg(
		short,
		long,
		global = true,
		env = "DECHST_LOG_LEVEL",
		default_value_t = LevelFilter::Info,

	)]
	#[serde_as(as = "DisplayFromStr")]
	#[merge(strategy = merge::ord::max)]
	pub log_level: LevelFilter,
}

impl Default for GlobalOpts {
	fn default() -> Self {
		Self {
			log_level: LevelFilter::Info,
		}
	}
}

#[derive(Default, Debug, Args, Serialize, Deserialize, Merge)]
#[serde(default, rename_all = "kebab-case")]
pub struct RepoOpts {
	#[arg(short, long, global = true, env = "DECHST_REPO", value_hint = clap::ValueHint::DirPath)]
	pub repo: Option<String>,

	/// WARNING: NOT SECURE
	#[arg(long, global = true, env = "DECHST_PASSWORD", conflicts_with_all = &["password_file", "password_command"])]
	pub password: Option<Zeroizing<String>>,

	#[arg(
		short,
		long,
		global = true,
		conflicts_with_all = &["password", "password_command"],
		env = "DECHST_PASSWORD_FILE",
		value_hint = clap::ValueHint::FilePath
	)]
	pub password_file: Option<PathBuf>,

	#[arg(
		long,
		global = true,
		conflicts_with_all = &["password", "password_file"],
		env = "DECHST_PASSWORD_COMMAND",
		value_hint = clap::ValueHint::CommandString
	)]
	pub password_command: Option<Zeroizing<String>>,

	#[arg(
		long,
		global = true,
		conflicts_with_all = &["password", "password_file", "password_command"],
		env = "DECHST_NO_PASSWORD"
	)]
	#[merge(strategy = merge::bool::overwrite_false)]
	pub no_password: bool,

	#[arg(
		long,
		global = true,
		conflicts_with = "cache_dir",
		env = "DECHST_NO_CACHE"
	)]
	#[merge(strategy = merge::bool::overwrite_false)]
	pub no_cache: bool,

	#[arg(
		long,
		global = true,
		conflicts_with = "no_cache",
		env = "DECHST_CACHE_DIR",
		value_hint = clap::ValueHint::DirPath
	)]
	pub cache_dir: Option<PathBuf>,

	#[arg(short, long, global = true, env = "DECHST_KEY")]
	pub key: Option<Zeroizing<String>>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum Identifier {
	#[default]
	Blake3,
}

impl From<Identifier> for process::identify::IdentifierParams {
	fn from(value: Identifier) -> Self {
		match value {
			Identifier::Blake3 => process::identify::IdentifierParams::Blake3,
		}
	}
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum Compression {
	None,
	#[default]
	Brotli,
}

impl From<Compression> for process::compress::CompressionParams {
	fn from(value: Compression) -> Self {
		match value {
			Compression::None => process::compress::CompressionParams::None,
			Compression::Brotli => process::compress::CompressionParams::Brotli,
		}
	}
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum Encryption {
	None,
	#[default]
	ChaCha20,
}

impl From<Encryption> for process::encrypt::EncryptionParams {
	fn from(value: Encryption) -> Self {
		match value {
			Encryption::None => process::encrypt::EncryptionParams::None,
			Encryption::ChaCha20 => process::encrypt::EncryptionParams::ChaCha20,
		}
	}
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum Verifier {
	None,
	#[default]
	Blake3,
}

impl From<Verifier> for process::verify::VerifierParams {
	fn from(value: Verifier) -> Self {
		match value {
			Verifier::None => process::verify::VerifierParams::None,
			Verifier::Blake3 => process::verify::VerifierParams::Blake3,
		}
	}
}

#[derive(Default, Debug, Args, Serialize, Deserialize, Merge)]
#[serde(default, rename_all = "kebab-case")]
pub struct ProcessOpts {
	#[arg(value_enum, long, global = true, env = "DECHST_PROCESS_IDENTIFIER")]
	identifier: Option<Identifier>,

	#[arg(value_enum, long, global = true, env = "DECHST_PROCESS_COMPRESSION")]
	compression: Option<Compression>,

	#[arg(value_enum, long, global = true, env = "DECHST_PROCESS_ENCRYPTION")]
	encryption: Option<Encryption>,

	#[arg(value_enum, long, global = true, env = "DECHST_PROCESS_VERIFIER")]
	verifier: Option<Verifier>,
}

impl ProcessOpts {
	pub fn recommended() -> Self {
		Self {
			identifier: Some(Identifier::default()),
			compression: Some(Compression::default()),
			encryption: Some(Encryption::default()),
			verifier: Some(Verifier::default()),
		}
	}
}
