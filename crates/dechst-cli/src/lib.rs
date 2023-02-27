#![feature(exit_status_error)]

pub mod command;
pub mod format;
pub mod opts;
pub mod password;
pub mod util;

/// Password used if the repository does not use encryption.
pub const DEFAULT_PASSWORD: &str = "";

// `CARGO_BIN_NAME` does not work, as this is the library part ...
pub const BINARY_NAME: &str = "dechst";
