//! TODO
//! - Move out processing steps into separate crates
//! - Save id within tagged chunk to verify it is correct
//! - Make passphrase derivative function generic
//! - Way to get a locked repo without writing a lock to backend (for append/readonly systems)
//! - Allows stdin as source
//! - Error Correction Algorithm? (Reed-Solomon)
//! - Check chunk size after compression; if its larger do not compress
//! - Allow selection of compression alg depending on mime/filetype, size ...
//! - Sharding config (directory spliting of packs e.g. [2] => 02/123123312.., [2, 2] => 02/12/12312..) (https://kopia.io/docs/advanced/sharding/)
//! - Save attr(5) attributes on unix with `xattr`
//! - Move path into node_path and make separate os/backen path (RawOsString)
#![feature(adt_const_params, generic_const_exprs)]

#![allow(rustdoc::private_intra_doc_links)]
#![deny(
    // Documentation
	// TODO: rustdoc::broken_intra_doc_links,
	// TODO: rustdoc::missing_crate_level_docs,
	// TODO: missing_docs,
	// TODO: clippy::missing_docs_in_private_items,

    // Other
	deprecated_in_future,
	exported_private_dependencies,
	future_incompatible,
	missing_copy_implementations,
	missing_debug_implementations,
	private_in_public,
	rust_2018_compatibility,
	rust_2018_idioms,
	trivial_casts,
	trivial_numeric_casts,
	//unstable_features,
	unused_import_braces,
	//unused_qualifications,

	// clippy attributes
	clippy::missing_const_for_fn,
	clippy::redundant_pub_crate,
	clippy::use_self
)]
#![allow(dead_code)]
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_alias))]
#![feature(associated_type_defaults)]
#![feature(fs_try_exists)]
#![cfg_attr(target_family = "windows", feature(windows_by_handle))]

pub mod os;
pub mod path;

pub mod backend;
pub mod id;
pub mod obj;
pub mod process;
pub mod repo;
pub mod source;

mod ideas {
	use crate::obj::tree::node::Node;

	pub trait Target {
		type Error: std::error::Error;
		type Item;
		type Write;

		type Iter: Iterator<Item = Result<Self::Item, Self::Error>>;

		fn iter(&self, item: Option<&Self::Item>) -> Result<Self::Iter, Self::Error>;
		fn write(&self, item: &Self::Item) -> Result<Self::Write, Self::Error>;
		fn node(&self, item: &Self::Item) -> Result<Node, Self::Error>;
	}

	#[derive(Debug, Clone, Copy)]
	pub enum RestoreMode {
		// Only create new files but dont touch existing ones
		OnlyNew,
		// Update exiting files
		OnlyExisting,
		// Restore + Delete any files not included in the backup
		Clean,
	}
}
