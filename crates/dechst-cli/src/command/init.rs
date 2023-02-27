use clap::Args;
use dechst::backend::BackendWrite;
use dechst::id::Id;
use dechst::obj::config::Config;
use dechst::obj::key::{EncryptOptions, Key};
use dechst::obj::ObjectKind;
use dechst::process::encrypt::EncryptionParams;
use dechst::process::format::{Format, Formatter};
use dechst::process::identify::Identify;
use dechst::process::pipeline::ChunkPipeline;
use dechst::process::{Instanciate, ProcessOptions};
use merge::Merge;

use crate::opts::{GlobalOpts, ProcessOpts, RepoOpts};
use crate::password::Password;
use crate::DEFAULT_PASSWORD;

#[derive(Debug, Args)]
pub struct Opts {
	#[command(flatten, next_help_heading = "PROCESS OPTIONS")]
	process: ProcessOpts,
}

// TODO: Maybe move creation process into lib
pub fn execute<B: BackendWrite>(
	_: GlobalOpts,
	repo_opts: RepoOpts,
	cmd: Opts,
	mut backend: B,
) -> anyhow::Result<()> {
	println!("Initializing repository");

	if backend.verify().is_ok() {
		println!("Repository already exists");
		return Ok(());
	}

	// Prepare
	let Opts { mut process } = cmd;
	process.merge(ProcessOpts::recommended());

	let encryption = process.chunk.encryption.unwrap();
	let encryption: EncryptionParams = encryption.into();

	// Create key
	let pw = match Password::get_init(&repo_opts)? {
		Some(pw) => pw,
		None => {
			log::info!("No password will be used");
			Password::from_str(DEFAULT_PASSWORD)
		}
	};

	let key = Key::random();

	// Create config file
	let opts = ProcessOptions {
		chunker: process.repo.chunker.unwrap().into(),
		identifier: process.repo.identifier.unwrap().into(),
		compression: process.chunk.compression.unwrap().into(),
		encryption,
		verifier: process.chunk.verifier.unwrap().into(),
	};

	let config = Config::new(opts);

	// Write files

	backend.create().unwrap();

	// Write key
	{
		let identifier = opts.identifier.create();

		let enc_key = key.encrypt(
			EncryptOptions::default(),
			encryption.create(),
			pw.as_bytes(),
		);

		let bytes = Formatter::Cbor.format(&enc_key)?;

		let id = identifier.identify(&key, &bytes)?;

		backend.write_all(ObjectKind::Key, &id, &bytes).unwrap();
	}

	// Write config
	{
		let pipeline = ChunkPipeline::new(opts, key);

		let bytes = Formatter::Cbor.format(&config)?;

		let bytes = pipeline.process(&bytes)?;

		backend
			.write_all(ObjectKind::Config, &Id::ZERO, &bytes)
			.unwrap();
	}

	Ok(())
}
