use std::path::PathBuf;

use clap::{Args, CommandFactory};

use crate::BINARY_NAME;

#[derive(Debug, Args)]
pub struct Opts {
	shell: clap_complete::Shell,

	#[arg(long)]
	output_directory: Option<PathBuf>,
}

pub fn execute(cmd: Opts) -> anyhow::Result<()> {
	let mut command = crate::opts::Opts::command();

	if let Some(dir) = cmd.output_directory {
		clap_complete::generate_to(cmd.shell, &mut command, BINARY_NAME, dir)?;
	} else {
		clap_complete::generate(cmd.shell, &mut command, BINARY_NAME, &mut std::io::stdout());
	}

	Ok(())
}
