use std::path::PathBuf;

use clap::{Args, CommandFactory};

use crate::BINARY_NAME;

#[derive(Debug, Args)]
pub struct Opts {
	#[arg(long, conflicts_with = "output_file")]
	output_directory: Option<PathBuf>,

	#[arg(long, conflicts_with = "output_directory")]
	output_file: Option<PathBuf>,
}

pub fn execute(cmd: Opts) -> anyhow::Result<()> {
	let command = crate::Opts::command();

	let man = clap_mangen::Man::new(command);
	let mut buffer: Vec<u8> = Default::default();
	man.render(&mut buffer)?;

	if let Some(dir) = cmd.output_directory {
		let output = dir.join(format!("{BINARY_NAME}.1"));
		std::fs::write(output, buffer)?;
	} else if let Some(file) = cmd.output_file {
		std::fs::write(file, buffer)?;
	} else {
		let string = std::str::from_utf8(&buffer)?;

		println!("{string}");
	}

	Ok(())
}
