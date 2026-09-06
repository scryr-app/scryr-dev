//! Generate a Rust module that embeds the built map UI assets.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(io::Error::other)?);
    let map_static_dir = manifest_dir.join("static").join("map");
    let sample_static_dir = manifest_dir.join("static").join("samples");
    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(io::Error::other)?);
    let output_path = out_dir.join("embedded_map_assets.rs");

    println!("cargo:rerun-if-changed={}", map_static_dir.display());
    println!("cargo:rerun-if-changed={}", sample_static_dir.display());

    let mut assets = Vec::new();
    if map_static_dir.join("index.html").is_file() {
        collect_assets(&map_static_dir, &map_static_dir, &mut assets)?;
    }
    assets.sort_by(|left, right| left.0.cmp(&right.0));

    let mut samples = Vec::new();
    if sample_static_dir.is_dir() {
        collect_sample_manifests(&sample_static_dir, &mut samples)?;
    }
    samples.sort_by(|left, right| left.0.cmp(&right.0));

    let mut output = fs::File::create(output_path)?;
    writeln!(output, "pub(crate) static ASSETS: &[EmbeddedAsset] = &[")?;
    for (path, absolute_path) in assets {
        let content_type = content_type_for_path(&path);
        writeln!(
            output,
            "    EmbeddedAsset {{ path: {path:?}, content_type: {content_type:?}, bytes: include_bytes!({absolute_path:?}) }},"
        )?;
    }
    writeln!(output, "];")?;
    writeln!(
        output,
        "pub(crate) static SAMPLE_MANIFESTS: &[EmbeddedSampleManifest] = &["
    )?;
    for (name, absolute_path) in samples {
        writeln!(
            output,
            "    EmbeddedSampleManifest {{ name: {name:?}, json: include_str!({absolute_path:?}) }},"
        )?;
    }
    writeln!(output, "];")?;

    Ok(())
}

/// Recursively collect files under the prepared Vite build directory.
fn collect_assets(
    root: &Path,
    directory: &Path,
    assets: &mut Vec<(String, String)>,
) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_name = entry.file_name();
        if file_name.to_string_lossy().starts_with('.') {
            continue;
        }

        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_assets(root, &path, assets)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }

        let relative = path.strip_prefix(root).map_err(io::Error::other)?;
        let asset_path = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        assets.push((asset_path, path.to_string_lossy().into_owned()));
    }

    Ok(())
}

/// Collect generated sample manifest JSON files prepared by the release task.
fn collect_sample_manifests(
    directory: &Path,
    samples: &mut Vec<(String, String)>,
) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_name = entry.file_name();
        if file_name.to_string_lossy().starts_with('.') {
            continue;
        }

        let path = entry.path();
        if !entry.file_type()?.is_file()
            || path.extension().and_then(|extension| extension.to_str()) != Some("json")
        {
            continue;
        }

        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        samples.push((name.to_string(), path.to_string_lossy().into_owned()));
    }

    Ok(())
}

/// Return the HTTP content type for a generated frontend asset path.
fn content_type_for_path(path: &str) -> &'static str {
    match Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
    {
        "css" => "text/css; charset=utf-8",
        "gif" => "image/gif",
        "html" => "text/html; charset=utf-8",
        "ico" => "image/x-icon",
        "jpg" | "jpeg" => "image/jpeg",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" | "map" | "webmanifest" => "application/json; charset=utf-8",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "txt" => "text/plain; charset=utf-8",
        "wasm" => "application/wasm",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}
