//! Manifest source discovery and source metadata helpers.
#![allow(clippy::redundant_pub_crate)]

mod files;
mod line_numbers;
mod root;

pub(crate) use files::{collect_manifest_source_files, manifest_source_files_in_directory};
pub(crate) use line_numbers::manifest_line_numbers;
pub(crate) use root::manifest_source_root;
