//! TODO
//! - Move out processing steps into separate crates
//! - Save id within tagged chunk to verify it is correct
//! - Make passphrase derivative function generic

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
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_alias))]
#![feature(associated_type_defaults)]
#![feature(fs_try_exists)]

pub mod backend;
pub mod id;
pub mod obj;
pub mod os;
pub mod process;
pub mod repo;

mod ideas {
	pub trait Target {
		fn restore();
		fn meta();
	}

	pub trait Source {
		fn iter();
		fn read();
		fn meta();
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
