/// TODO
/// - Able to chose output format
use clap::{Args, ValueEnum};
use dechst::backend::BackendWrite;
use dechst::obj;
use serde::{Deserialize, Serialize};

use crate::format::OutputFormat;
use crate::opts::{GlobalOpts, RepoOpts};

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

	#[arg(long, value_enum, global = true, default_value_t)]
	format: OutputFormat,
}

pub fn execute<B: BackendWrite>(
	_: GlobalOpts,
	_: RepoOpts,
	cmd: Opts,
	backend: B,
) -> anyhow::Result<()> {
	println!("Listing all {:?}", cmd.object);

	let ids = backend
		.iter(cmd.object.into())
		.unwrap()
		.collect::<Result<Vec<_>, _>>()
		.unwrap();

	cmd.format.print(&ids);

	Ok(())
}
