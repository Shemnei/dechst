pub mod command;

use std::path::PathBuf;

use clap::{Args, Parser};
use log::LevelFilter;
use merge::Merge;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

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
	pub password: Option<String>,

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
	pub password_command: Option<String>,

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
}
