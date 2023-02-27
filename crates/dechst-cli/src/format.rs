use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
	#[default]
	Debug,
	#[cfg(feature = "format-yaml")]
	Yaml,
	#[cfg(feature = "format-json")]
	Json,
}

impl OutputFormat {
	pub fn print<V: serde::ser::Serialize + fmt::Debug>(&self, value: &V) {
		match self {
			Self::Debug => {
				println!("{:#?}", value)
			}

			#[cfg(feature = "format-yaml")]
			Self::Yaml => {
				println!("{}", serde_yaml::to_string(value).unwrap());
			}

			#[cfg(feature = "format-json")]
			Self::Json => {
				println!("{}", serde_json::to_string_pretty(value).unwrap());
			}

			_ => unimplemented!(),
		}
	}
}
