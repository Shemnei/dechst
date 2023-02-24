use clap::Parser;
use dechst_cli::{command, Opts};
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

fn main() -> anyhow::Result<()> {
	let opts = Opts::parse();

	// TODO: Read config file and merge

	// Setup logging
	{
		// TODO:
		let level = opts.global.log_level;
		TermLogger::init(
			level,
			ConfigBuilder::new()
				.set_time_level(LevelFilter::Off)
				.build(),
			TerminalMode::Stderr,
			ColorChoice::Auto,
		)
		.unwrap();
	}

	command::execute(opts)
}
