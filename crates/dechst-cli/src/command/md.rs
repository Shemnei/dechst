use std::path::PathBuf;

use clap::Args;

use crate::BINARY_NAME;

#[derive(Debug, Args)]
pub struct Opts {
	#[arg(long, conflicts_with = "output_file")]
	output_directory: Option<PathBuf>,

	#[arg(long, conflicts_with = "output_directory")]
	output_file: Option<PathBuf>,
}

pub fn execute(cmd: Opts) -> anyhow::Result<()> {
	let md = clap_markdown::help_markdown::<crate::Opts>();

	if let Some(dir) = cmd.output_directory {
		let output = dir.join(format!("{BINARY_NAME}.md"));
		std::fs::write(output, md)?;
	} else if let Some(file) = cmd.output_file {
		std::fs::write(file, md)?;
	} else {
		println!("{md}");
	}

	Ok(())
}
