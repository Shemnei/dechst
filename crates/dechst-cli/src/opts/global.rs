use clap::Args;
use log::LevelFilter;
use merge::Merge;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Debug, Args, Serialize, Deserialize, Merge)]
#[serde_as]
#[serde(default, rename_all = "kebab-case")]
pub struct GlobalOpts {
	#[arg(
		short,
		long,
		global = true,
		env = "DECHST_LOG_LEVEL",
		default_value_t = LevelFilter::Info,

	)]
	#[serde_as(as = "DisplayFromStr")]
	#[merge(strategy = merge::ord::max)]
	pub log_level: LevelFilter,
}

impl Default for GlobalOpts {
	fn default() -> Self {
		Self {
			log_level: LevelFilter::Info,
		}
	}
}
