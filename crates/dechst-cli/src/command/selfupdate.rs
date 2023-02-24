use clap::Args;
use self_update::cargo_crate_version;

use crate::BINARY_NAME;

#[derive(Debug, Args)]
pub struct Opts {
	#[arg(long)]
	force: bool,
}

pub fn execute(cmd: Opts) -> anyhow::Result<()> {
	let status = self_update::backends::github::Update::configure()
		.repo_owner("Shemnei")
		.repo_name("dechst")
		.bin_name(BINARY_NAME)
		.show_download_progress(true)
		.current_version(cargo_crate_version!())
		.no_confirm(cmd.force)
		.build()?
		.update()?;

	println!("Update status: `{}`!", status.version());

	Ok(())
}
