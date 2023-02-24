use std::fs::File;
use std::io::{BufRead as _, BufReader};
use std::process::{Command, Stdio};

use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::RepoOpts;

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Password(String);

impl Password {
	pub fn from_str<S: Into<String>>(s: S) -> Self {
		Self(s.into())
	}

	pub fn get_init(opts: &RepoOpts) -> anyhow::Result<Option<Self>> {
		if opts.no_password {
			Ok(None)
		} else {
			match Self::from_opts(opts)? {
				Some(pw) => Ok(Some(pw)),
				None => Ok(Some(Self::ask_create()?)),
			}
		}
	}

	pub fn get(opts: &RepoOpts) -> anyhow::Result<Option<Self>> {
		if opts.no_password {
			Ok(None)
		} else {
			match Self::from_opts(opts)? {
				Some(pw) => Ok(Some(pw)),
				None => Ok(Some(Self::ask(None)?)),
			}
		}
	}

	pub fn from_opts(opts: &RepoOpts) -> anyhow::Result<Option<Self>> {
		let pw = if let Some(pw) = &opts.password {
			Some(pw.to_string())
		} else if let Some(path) = &opts.password_file {
			let mut reader = BufReader::new(File::open(path)?);
			Some(rpassword::read_password_from_bufread(&mut reader)?)
		} else if let Some(cmd) = &opts.password_command {
			let cmd = prepare_command(cmd)?;
			let pw = run(cmd)?;
			Some(pw)
		} else {
			None
		};

		if let Some(pw) = pw {
			Ok(Some(Self(pw)))
		} else {
			Ok(None)
		}
	}

	pub fn ask_create() -> anyhow::Result<Self> {
		loop {
			let pw = rpassword::prompt_password("Enter new passphrase: ")?;
			let rpw = rpassword::prompt_password("Enter passphrase again: ")?;

			if pw == rpw {
				return Ok(Self(pw));
			}

			println!("Passphrases did not match!");
		}
	}

	pub fn ask<'a, I: Into<Option<&'a str>>>(prompt: I) -> anyhow::Result<Self> {
		let prompt = prompt.into().unwrap_or("Enter passphrase: ");

		Ok(Self(rpassword::prompt_password(prompt)?))
	}

	pub fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}
}

fn prepare_command(c: &str) -> anyhow::Result<Command> {
	#[cfg(target_family = "windows")]
	{
		let mut cmd = Command::new("cmd");
		cmd.args(["/C", c]);
		return Ok(cmd);
	}

	#[cfg(target_family = "unix")]
	{
		let mut cmd = Command::new("sh");
		cmd.args(["-c", c]);
		return Ok(cmd);
	}

	#[cfg(not(any(target_family = "unix", target_family = "windows")))]
	anyhow::bail!("Command execution is only supported for windows/unix systems");
}

fn run(mut cmd: Command) -> anyhow::Result<String> {
	let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::null()).spawn()?;

	// No need to call kill here as the program will immediately exit
	// and thereby kill all spawned children
	let stdout = child.stdout.take().expect("Failed to get stdout from hook");

	let stdout = BufReader::new(stdout)
		.lines()
		.collect::<Result<Vec<_>, std::io::Error>>();

	let stdout = match stdout {
		Ok(lines) => lines.join("\n"),
		Err(err) => {
			let _ = child.kill();
			return Err(err.into());
		}
	};

	child
		.wait_with_output()?
		.status
		.exit_ok()
		.map_err(Into::into)
		.map(|_| stdout)
}
