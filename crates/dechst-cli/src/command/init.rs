use clap::Args;
use dechst::backend::BackendWrite;
use dechst::obj::lock::{Exclusive, None, Shared};
use dechst::repo::marker::LockMarker;
use dechst::repo::Repo;

use crate::{GlobalOpts, Opts, RepoOpts};

#[derive(Debug, Args)]
pub struct Init {}

pub fn execute<B: BackendWrite>(
	global_opts: GlobalOpts,
	repo_opts: RepoOpts,
	cmd: Init,
	backend: B,
) -> anyhow::Result<()> {
	log::info!("Initializing repository");

	let repo = Repo::create(backend).unwrap();
	let lock = LockMarker::NO.key::<Exclusive>().config::<Exclusive>();
	let repo = repo.lock(lock).unwrap();

	// Write config with defaults

	Ok(())
}
