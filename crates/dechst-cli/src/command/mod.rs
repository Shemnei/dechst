pub mod init;

use clap::Subcommand;

use self::init::Init;

#[non_exhaustive]
#[derive(Debug, Subcommand)]
pub enum Command {
	Init(Init),
}
