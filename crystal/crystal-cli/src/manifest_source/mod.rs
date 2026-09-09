//! Manifest source discovery and source metadata helpers.
#![allow(clippy::redundant_pub_crate)]

mod files;
mod line_numbers;
mod root;

pub(crate) use files::collect_manifest_source_files;
pub(crate) use line_numbers::manifest_line_numbers;
