use std::fmt;

use chrono::{DateTime, Local, Utc};
use clap::ValueEnum;
use dechst::id::Id;
use dechst::obj::config::Config;
use dechst::obj::key::Key;
use dechst::os::User;
use dechst::process::ProcessOptions;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
	#[default]
	Debug,
	Plain,
	#[cfg(feature = "serde_yaml")]
	Yaml,
	#[cfg(feature = "serde_json")]
	Json,
}

impl OutputFormat {
	pub fn print<V: serde::ser::Serialize + fmt::Debug + FormatPlain>(&self, value: &V) {
		match self {
			Self::Debug => {
				println!("{:#?}", value)
			}

			Self::Plain => {
				println!("{}", FormatPlain::to_plain(value));
			}

			#[cfg(feature = "serde_yaml")]
			Self::Yaml => {
				println!("{}", serde_yaml::to_string(value).unwrap());
			}

			#[cfg(feature = "serde_json")]
			Self::Json => {
				println!("{}", serde_json::to_string_pretty(value).unwrap());
			}

			_ => unimplemented!(),
		}
	}
}

// TODO: Replace with custom serde serializer
pub trait FormatPlain {
	fn to_plain(&self) -> String;
}

impl FormatPlain for [u8] {
	fn to_plain(&self) -> String {
		format!("{}", hex::encode(self))
	}
}

impl FormatPlain for Id {
	fn to_plain(&self) -> String {
		format!("{self:x}")
	}
}

impl FormatPlain for DateTime<Utc> {
	fn to_plain(&self) -> String {
		let local = self.with_timezone(&Local);
		local.format("%Y-%m-%d %H:%M:%S").to_string()
	}
}

impl FormatPlain for User {
	#[rustfmt::skip]
	fn to_plain(&self) -> String {
		match self {
			User::Unix(usr) => {
				format!(
"Username: {}
Hostname: {}
UID:      {}
GID:      {}",
					usr.username.as_ref().map(|u| format!("{}", u)).as_deref().unwrap_or("-"),
					usr.hostname.as_ref().map(|u| format!("{}", u)).as_deref().unwrap_or("-"),
					usr.uid.as_ref().map(|u| u.to_string()).as_deref().unwrap_or("-"),
					usr.gid.as_ref().map(|u| u.to_string()).as_deref().unwrap_or("-"),
				)
			}
			User::Windows(usr) => {
				format!(
"Username: {}
Hostname: {}",
					usr.username.as_ref().map(|u| format!("{}", u)).as_deref().unwrap_or("-"),
					usr.hostname.as_ref().map(|u| format!("{}", u)).as_deref().unwrap_or("-"),
				)
			}
		}
	}
}

impl FormatPlain for ProcessOptions {
	#[rustfmt::skip]
	fn to_plain(&self) -> String {
		format!(
"# Process Options
Identifier:  {:?}
Compression: {:?}
Encryption:  {:?}
Verifier:    {:?}",
			self.identifier,
			self.compression,
			self.encryption,
			self.verifier
		)
	}
}

impl FormatPlain for Config {
	#[rustfmt::skip]
	fn to_plain(&self) -> String {
		format!(
"# Config
Version: {}
Id:      {}

{}",
			self.version,
			self.id.to_plain(),
			self.process.to_plain()
		)
	}
}

impl FormatPlain for Key {
	#[rustfmt::skip]
	fn to_plain(&self) -> String {
		format!(
"# Metadata
{}
Created:  {}

# Keys
Encrypt:  0x{}
Verify:   0x{}
Identify: 0x{}
Chunk:    0x{}",
			self.meta().user.to_plain(),
			self.meta().created.to_plain(),
			self.bytes().encrypt_key().to_plain(),
			self.bytes().verify_key().to_plain(),
			self.bytes().identify_key().to_plain(),
			self.bytes().chunk_key().to_plain(),
		)
	}
}
