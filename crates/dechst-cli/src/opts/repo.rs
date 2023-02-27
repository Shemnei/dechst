use std::path::PathBuf;

use clap::Args;
use merge::Merge;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

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
