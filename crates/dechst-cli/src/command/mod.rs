pub mod cat;
#[cfg(feature = "clap_complete")]
pub mod completions;
pub mod init;
pub mod list;
#[cfg(feature = "clap_mangen")]
pub mod man;
#[cfg(feature = "clap-markdown")]
pub mod md;
#[cfg(feature = "self_update")]
pub mod selfupdate;

use clap::Subcommand;
use dechst::backend::local::Local;
use dechst::backend::local2::Local2;
use dechst::backend::BackendWrite;
use dechst::repo::Repo;

use crate::util::unlock_repo;
use crate::Opts;

#[non_exhaustive]
#[derive(Debug, Subcommand)]
pub enum Command {
	// Utility
	#[cfg(feature = "clap_complete")]
	Completions(completions::Opts),
	#[cfg(feature = "clap_mangen")]
	Man(man::Opts),
	#[cfg(feature = "clap-markdown")]
	Md(md::Opts),
	#[cfg(feature = "self_update")]
	SelfUpdate(selfupdate::Opts),

	// Read
	Cat(cat::Opts),
	List(list::Opts),

	// Write
	Init(init::Opts),
}

pub fn execute(opts: Opts) -> anyhow::Result<()> {
	log::debug!("Executing command {:?}", opts.command);

	#[cfg(feature = "clap_complete")]
	if let Command::Completions(cmd) = opts.command {
		return completions::execute(cmd);
	}

	#[cfg(feature = "clap_mangen")]
	if let Command::Man(cmd) = opts.command {
		return man::execute(cmd);
	}

	#[cfg(feature = "clap-markdown")]
	if let Command::Md(cmd) = opts.command {
		return md::execute(cmd);
	}

	#[cfg(feature = "self_update")]
	if let Command::SelfUpdate(cmd) = opts.command {
		return selfupdate::execute(cmd);
	}

	if let Some(repo) = &opts.repo.repo {
		if let Some(path) = repo.strip_prefix("v2://") {
			let backend = Local2::new(path);
			exec_repo_command(opts, backend)
		} else {
			log::debug!("Repo is no url; Falling back to local");
			let backend = Local::new(repo);
			exec_repo_command(opts, backend)
		}
	} else {
		anyhow::bail!("No repository given");
	}
}

fn exec_repo_command<B: BackendWrite>(opts: Opts, backend: B) -> anyhow::Result<()> {
	let Opts {
		global: global_opts,
		repo: repo_opts,
		command,
	} = opts;

	log::debug!("Using {:?} backend", backend);

	// Raw backend access
	if let Command::Init(cmd) = command {
		return init::execute(global_opts, repo_opts, cmd, backend);
	} else if let Command::List(cmd) = command {
		return list::execute(global_opts, repo_opts, cmd, backend);
	}

	let repo = Repo::open(backend).unwrap();

	let repo = unlock_repo(repo, &repo_opts).map_err(|(_, err)| err)?;

	match command {
		Command::Cat(cmd) => cat::execute(global_opts, repo_opts, cmd, repo),
		_ => anyhow::bail!("Unknown command: {command:?}"),
	}
}
