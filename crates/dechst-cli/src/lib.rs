#![feature(exit_status_error)]

pub mod command;
pub mod format;
pub mod opts;
pub mod password;
pub mod util;

pub use opts::global::GlobalOpts;
pub use opts::process::{ChunkProcessOpts, RepoProcessOpts};
pub use opts::repo::RepoOpts;
pub use opts::Opts;

/// Password used if the repository does not use encryption.
pub const DEFAULT_PASSWORD: &str = "";

// `CARGO_BIN_NAME` does not work, as this is the library part ...
pub const BINARY_NAME: &str = "dechst";
