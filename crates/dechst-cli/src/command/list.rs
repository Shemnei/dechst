/// TODO
/// - Able to chose output format
use std::str::FromStr;

use clap::{Args, Subcommand, ValueEnum};
use dechst::backend::ext::Find;
use dechst::backend::BackendWrite;
use dechst::id::{self, Id};
use dechst::obj;
use dechst::obj::lock::{None, Shared};
use dechst::repo::config::ConfigRead;
use dechst::repo::index::IndexRead;
use dechst::repo::key::KeyRead;
use dechst::repo::lock::LockRead;
use dechst::repo::marker::LockMarker;
use dechst::repo::DecryptedRepo;
use serde::{Deserialize, Serialize};

use crate::password::Password;
use crate::{GlobalOpts, RepoOpts};

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
enum ObjectKind {
	Key,
	Lock,
	Index,
	Snapshot,
	Pack,
}

impl From<ObjectKind> for obj::ObjectKind {
	fn from(value: ObjectKind) -> Self {
		match value {
			ObjectKind::Key => Self::Key,
			ObjectKind::Lock => Self::Lock,
			ObjectKind::Index => Self::Index,
			ObjectKind::Snapshot => Self::Snapshot,
			ObjectKind::Pack => Self::Pack,
		}
	}
}

#[derive(Debug, Args)]
pub struct Opts {
	#[arg(value_enum)]
	object: ObjectKind,
}

pub fn execute<B: BackendWrite>(
	_: GlobalOpts,
	_: RepoOpts,
	cmd: Opts,
	backend: B,
) -> anyhow::Result<()> {
	println!("Listing all {:?}", cmd.object);

	let ids = backend.iter(cmd.object.into()).unwrap();

	for id in ids {
		let Ok(id) = id else {
			continue;
		};

		println!("{id:x}");
	}

	Ok(())
}
