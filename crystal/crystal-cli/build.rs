//! Generate a Rust module that embeds the bundled Python SDK package.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(io::Error::other)?);
    let sdk_dir = manifest_dir.join("static").join("scryr-sdk");
    let source_sdk_dir = manifest_dir
        .join("..")
        .join("..")
        .join("manifest")
        .join("scryr");
    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(io::Error::other)?);
    let output_path = out_dir.join("embedded_scryr_sdk.rs");

    println!("cargo:rerun-if-changed={}", sdk_dir.display());
    println!("cargo:rerun-if-changed={}", source_sdk_dir.display());
    verify_copied_sdk_matches_source(&source_sdk_dir, &sdk_dir)?;

    let mut files = Vec::new();
    collect_files(&sdk_dir, &sdk_dir, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));

    let mut output = fs::File::create(output_path)?;
    writeln!(
        output,
        "/// Python SDK package files needed to run `python -m scryr.cli`."
    )?;
    writeln!(output, "static EMBEDDED_SDK_FILES: &[EmbeddedSdkFile] = &[")?;
    for (path, absolute_path) in files {
        writeln!(
            output,
            "    EmbeddedSdkFile {{ path: {path:?}, content: include_str!({absolute_path:?}) }},"
        )?;
    }
    writeln!(output, "];")?;

    Ok(())
}

/// Verify the embedded package copy has not drifted from the Python SDK source.
fn verify_copied_sdk_matches_source(
    source_sdk_dir: &Path,
    copied_sdk_dir: &Path,
) -> io::Result<()> {
    let mut source_files = Vec::new();
    let mut copied_files = Vec::new();
    collect_files(source_sdk_dir, source_sdk_dir, &mut source_files)?;
    collect_files(copied_sdk_dir, copied_sdk_dir, &mut copied_files)?;
    source_files.sort_by(|left, right| left.0.cmp(&right.0));
    copied_files.sort_by(|left, right| left.0.cmp(&right.0));

    let source_paths = source_files
        .iter()
        .map(|(path, _)| path.as_str())
        .collect::<Vec<_>>();
    let copied_paths = copied_files
        .iter()
        .map(|(path, _)| path.as_str())
        .collect::<Vec<_>>();
    if source_paths != copied_paths {
        return Err(io::Error::other(format!(
            "embedded Scryr SDK file list differs from manifest/scryr; refresh {} from {}",
            copied_sdk_dir.display(),
            source_sdk_dir.display()
        )));
    }

    for ((relative, source_path), (_, copied_path)) in source_files.iter().zip(copied_files.iter())
    {
        let source = fs::read(source_path)?;
        let copied = fs::read(copied_path)?;
        if source != copied {
            return Err(io::Error::other(format!(
                "embedded Scryr SDK file {relative} differs from manifest/scryr; refresh {} from {}",
                copied_sdk_dir.display(),
                source_sdk_dir.display()
            )));
        }
    }

    Ok(())
}

/// Recursively collect source files in the crate-local Python SDK package.
fn collect_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, String)>,
) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_text = file_name.to_string_lossy();
        if file_name_text.starts_with('.')
            || file_name_text == "__pycache__"
            || file_name_text == "build"
            || file_name_text.ends_with(".egg-info")
        {
            continue;
        }

        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_files(root, &path, files)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) == Some("pyc") {
            continue;
        }

        let relative = path.strip_prefix(root).map_err(io::Error::other)?;
        let sdk_path = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        files.push((sdk_path, path.to_string_lossy().into_owned()));
    }

    Ok(())
}
