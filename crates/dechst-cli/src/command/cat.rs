/// TODO
/// - Able to chose output format
use std::str::FromStr;

use clap::{Args, Subcommand};
use dechst::backend::ext::{Find, FindIdExt, ReadToEnd};
use dechst::backend::{BackendRead, BackendWrite};
use dechst::id::{self, Id};
use dechst::obj::config::Config;
use dechst::obj::index::Index;
use dechst::obj::key::{EncryptedKey, Key};
use dechst::obj::lock::Lock;
use dechst::obj::{self, RepoObject};
use dechst::process::format::{Format, FormatterParams};
use dechst::process::{pipeline, Instanciate};
use dechst::repo::DecryptedRepo;
use serde::{Deserialize, Serialize};

use crate::format::OutputFormat;
use crate::opts::{GlobalOpts, RepoOpts};
use crate::password::Password;

#[derive(Debug, Clone, PartialEq, Eq, Args, Serialize, Deserialize)]
struct KeyOpts {
	id: Option<String>,
}

impl KeyOpts {
	pub fn into_id(self) -> Option<IdOpt> {
		self.id.map(|id| IdOpt { id })
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Args, Serialize, Deserialize)]
struct IdOpt {
	id: String,
}

impl IdOpt {
	pub fn is_full(&self) -> bool {
		self.id.len() == id::WIDTH / 2
	}

	pub fn to_id(&self) -> Option<Id> {
		if self.is_full() {
			Id::from_str(&self.id).ok()
		} else {
			None
		}
	}
}

impl AsRef<str> for IdOpt {
	fn as_ref(&self) -> &str {
		&self.id
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand, Serialize, Deserialize)]
enum ObjectKind {
	Config,
	Key(KeyOpts),
	Lock(IdOpt),
	Index(IdOpt),
	Snapshot(IdOpt),
	Pack(IdOpt),
}

#[derive(Debug, Args)]
pub struct Opts {
	#[command(subcommand)]
	object: ObjectKind,

	#[arg(long, value_enum, global = true, default_value_t)]
	format: OutputFormat,
}

pub fn execute<B: BackendWrite>(
	_: GlobalOpts,
	_: RepoOpts,
	cmd: Opts,
	repo: DecryptedRepo<B>,
) -> anyhow::Result<()> {
	let (backend, key, key_id) = unsafe { repo.into_inner() };

	let Opts { object, format } = cmd;

	match object {
		ObjectKind::Config => cat_cfg(backend, format, key),
		ObjectKind::Key(id) => cat_key(backend, format, key, key_id, id.into_id()),
		ObjectKind::Lock(id) => cat_obj::<_, Lock>(backend, format, &key, &id),
		ObjectKind::Index(id) => cat_obj::<_, Index>(backend, format, &key, &id),
		_ => unimplemented!(),
	}
}
fn cat_cfg<B: BackendRead>(backend: B, format: OutputFormat, key: Key) -> anyhow::Result<()> {
	let cfg: Config = get_obj(&backend, &key, &Id::ZERO)?;

	format.print(&cfg);

	Ok(())
}

fn cat_key<B>(
	backend: B,
	format: OutputFormat,
	key: Key,
	key_id: Id,
	search_id: Option<IdOpt>,
) -> anyhow::Result<()>
where
	B: BackendRead,
{
	let id = if let Some(id) = search_id {
		let tmp_id = if let Some(i) = id.to_id() {
			i
		} else {
			match backend
				.find_id(obj::ObjectKind::Key, id.as_ref())
				.unwrap()
				.unwrap()
			{
				Find::None => anyhow::bail!("No key found for the given id"),
				Find::Unique(id) => id,
				Find::NonUnique => anyhow::bail!("Multiple keys found for the given id"),
			}
		};

		if tmp_id == key_id {
			None
		} else {
			Some(tmp_id)
		}
	} else {
		None
	};

	let key: Key = if let Some(id) = id {
		let bytes = backend.read_to_end(obj::ObjectKind::Key, &id).unwrap();
		let key: EncryptedKey = FormatterParams::Cbor.create().parse(&bytes)?;

		let password = Password::ask(format!("Enter passphrase for key {id}: ").as_str())?;

		key.decrypt(password.as_bytes())
	} else {
		key
	};

	format.print(&key);

	Ok(())
}

fn cat_obj<'de, B, V>(backend: B, format: OutputFormat, key: &Key, id: &IdOpt) -> anyhow::Result<()>
where
	B: BackendRead,
	V: RepoObject,
{
	let id = resolve_id(&backend, V::KIND, id)?;
	let obj: V = get_obj(&backend, &key, &id)?;

	println!("{}: {id}", V::KIND);
	println!("{obj:#?}");

	Ok(())
}

fn get_obj<'de, B, V>(backend: &B, key: &Key, id: &Id) -> anyhow::Result<V>
where
	B: BackendRead,
	V: RepoObject,
{
	let bytes = backend.read_to_end(V::KIND, id).unwrap();
	Ok(pipeline::unprocess(
		FormatterParams::Cbor.create(),
		key,
		&bytes,
	)?)
}

fn resolve_id<B>(backend: &B, kind: obj::ObjectKind, id: &IdOpt) -> anyhow::Result<Id>
where
	B: BackendRead,
{
	let id = if let Some(id) = id.to_id() {
		// Full length id
		id
	} else {
		// Partial id
		match backend.find_id(kind, id.as_ref()).unwrap().unwrap() {
			Find::None => anyhow::bail!("No {kind} found for the given id"),
			Find::Unique(id) => id,
			Find::NonUnique => anyhow::bail!("Multiple {kind} found for the given id"),
		}
	};

	Ok(id)
}
