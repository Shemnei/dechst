use clap::Parser;
use dechst::backend::local::Local;
use dechst::backend::local2::Local2;
use dechst::backend::BackendWrite;
use dechst::repo::Repo;
use dechst_cli::command::{self, *};
use dechst_cli::Opts;
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

	if let Some(repo) = &opts.repo.repo {
		if let Some(path) = repo.strip_prefix("v2://") {
			let backend = Local2::new(path);
			execute(opts, backend)
		} else {
			log::debug!("Repo is no url; Falling back to local");
			let backend = Local::new(repo);
			execute(opts, backend)
		}
	} else {
		anyhow::bail!("No repository given");
	}
}

fn execute<B: BackendWrite>(opts: Opts, backend: B) -> anyhow::Result<()> {
	log::debug!("Using {:?} backend", backend);

	let Opts {
		global: global_opts,
		repo: repo_opts,
		command,
	} = opts;

	log::debug!("Executing command {:?}", command);

	// Raw backend access
	if let Command::Init(cmd) = command {
		return command::init::execute(global_opts, repo_opts, cmd, backend);
	}

	let repo = Repo::open(backend).unwrap();

	// Options:
	// - Interactive key chooser
	// - Try to decrypt each key with the password

	match command {
		_ => anyhow::bail!("Unknown command: {command:?}"),
	}

	Ok(())
}
