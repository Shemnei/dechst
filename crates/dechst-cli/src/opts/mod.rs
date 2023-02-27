pub mod global;
pub mod process;
pub mod repo;

use clap::Parser;
pub use global::GlobalOpts;
pub use process::{ChunkProcessOpts, ProcessOpts, RepoProcessOpts};
pub use repo::RepoOpts;

use crate::command::Command;

#[derive(Debug, Parser)]
#[command(author, about, name = "dechst", version)]
pub struct Opts {
	#[command(flatten, next_help_heading = "GLOBAL OPTIONS")]
	pub global: global::GlobalOpts,

	#[command(flatten, next_help_heading = "REPOSITORY OPTIONS")]
	pub repo: repo::RepoOpts,

	#[command(subcommand)]
	pub command: Command,
}
