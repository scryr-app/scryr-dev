//! Start the local UI and load validated manifest artifacts.
use super::workflow::{Project, publish};
use crate::args::ServerArgs;
use sha2::{Digest, Sha256};
use std::time::Duration;

/// Run the server, optionally formatting, validating, and watching local sources.
pub(super) async fn run(args: &ServerArgs) -> Result<(), String> {
    if args.server_only {
        return crystal_server::server::run(args.server.clone())
            .await
            .map_err(|e| e.to_string());
    }
    if args.port == 0 {
        return Err(
            "serve requires a fixed --port; use --server-only for an ephemeral port".into(),
        );
    }
    // Bind before starting the loader, so an occupied port cannot receive our upload.
    let root = args
        .common
        .manifest_dir
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let file = crate::manifest_paths::resolve_manifest_file(&root, &args.common.manifest_file)?;
    let common = args.common.clone();
    let format = !args.no_format;
    let workspace = crystal_server::editor::LocalWorkspace::new(
        &root,
        &file,
        std::sync::Arc::new(move |files, entrypoint| {
            let staged = tempfile::tempdir().map_err(|e| e.to_string())?;
            // Keep the original runtime and working directory (including opted-in
            // project dependencies), but validate source in a disposable tree.
            let mut project = Project::new(common.clone())?;
            let source_root = project.file.parent().ok_or("Missing source root")?;
            let relative = source_root
                .strip_prefix(&project.root)
                .map_err(|e| e.to_string())?;
            let staged_sources = staged.path().join(relative);
            for directory in source_root
                .ancestors()
                .take_while(|path| path.starts_with(&project.root))
            {
                let relative = directory
                    .strip_prefix(&project.root)
                    .map_err(|e| e.to_string())?;
                let destination = staged.path().join(relative);
                std::fs::create_dir_all(&destination).map_err(|e| e.to_string())?;
                for name in ["pyproject.toml", "ruff.toml", ".ruff.toml", "ty.toml"] {
                    let config = directory.join(name);
                    if config.is_file() {
                        std::fs::copy(config, destination.join(name)).map_err(|e| e.to_string())?;
                    }
                }
            }
            for source in files {
                let path = staged_sources.join(source.path);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                std::fs::write(path, source.content).map_err(|e| e.to_string())?;
            }
            project.file = staged_sources.join(entrypoint);
            if format {
                project.tool("--format")?;
            }
            serde_json::from_str(&project.check()?.json).map_err(|e| e.to_string())
        }),
    )?;
    let server =
        crystal_server::server::start_with_workspace(args.server.clone(), Some(workspace.clone()))
            .await
            .map_err(|e| e.to_string())?;
    tokio::pin!(server);
    tokio::select! {
        result = &mut server => result.map_err(|e| e.to_string()),
        () = load_loop(args, &workspace) => server.await.map_err(|e| e.to_string()),
    }
}

/// Wait for this server, then refresh only when source contents change.
async fn load_loop(args: &ServerArgs, workspace: &crystal_server::editor::LocalWorkspace) {
    let host = match args.host.as_str() {
        "0.0.0.0" => "127.0.0.1".to_owned(),
        "::" => "[::1]".to_owned(),
        host if host.contains(':') && !host.starts_with('[') => format!("[{host}]"),
        host => host.to_owned(),
    };
    let base = format!("http://{host}:{}", args.port);
    let client = reqwest::Client::new();
    for attempt in 0..100 {
        if client
            .get(format!("{base}/ready"))
            .timeout(Duration::from_secs(1))
            .send()
            .await
            .is_ok_and(|r| r.status().is_success())
        {
            break;
        }
        if attempt == 99 {
            eprintln!("Local server did not become ready; manifest was not loaded");
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let mut common = args.common.clone();
    common.graphql_url = Some(format!("{base}/graphql"));
    let mut previous = None;
    let mut opened = false;
    loop {
        let guard = workspace.gate.lock().await;
        let fingerprint = fingerprint(&common);
        if previous.as_ref() != Some(&fingerprint) {
            let load_common = common.clone();
            let format = !args.no_format;
            let prepared = tokio::task::spawn_blocking(move || {
                let project = Project::new(load_common.clone())?;
                if format {
                    project.tool("--format")?;
                }
                let revision = self::fingerprint(&load_common);
                let checked = project.check()?;
                Ok::<_, String>((project, checked, revision))
            })
            .await;
            let (result, revision) = match prepared {
                Ok(Ok((project, checked, revision))) => {
                    let declarations =
                        serde_json::from_str(&checked.json).map_err(|e| e.to_string());
                    let result = match declarations {
                        Ok(declarations) => match publish(&project, checked).await {
                            Ok(url) => workspace.set_integrations(declarations).map(|()| url),
                            Err(error) => Err(error),
                        },
                        Err(error) => Err(error),
                    };
                    (result, revision)
                }
                Ok(Err(error)) => (Err(error), fingerprint.clone()),
                Err(error) => (Err(error.to_string()), fingerprint.clone()),
            };
            match result {
                Ok(url) => {
                    if !opened && !args.no_open {
                        if let Err(error) = crate::auth::browser::open_browser(&url) {
                            eprintln!("{error}");
                        }
                        opened = true;
                    }
                }
                Err(error) => {
                    eprintln!(
                        "Manifest was not loaded; the previous diagram is retained.\n{error}"
                    );
                }
            }
            // Ignore our formatting, but retain edits made while checking or uploading.
            previous = Some(revision);
        }
        if !args.watch {
            return;
        }
        drop(guard);
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// Fingerprint source contents, including creation/deletion, without executing code.
fn fingerprint(args: &crate::args::GenerateCommonArgs) -> String {
    let file = args.manifest_dir.join(&args.manifest_file);
    let sources = crate::manifest_source::collect_manifest_source_files(&file)
        .and_then(|files| serde_json::to_string(&files).map_err(|e| e.to_string()))
        .unwrap_or_else(|error| error);
    let mut digest = Sha256::new();
    digest.update(sources.as_bytes());
    if let Ok(secret_file) = std::fs::read(crystal_core::integration_secrets::secrets_path(&file)) {
        digest.update(secret_file);
    }
    format!("{:?}", digest.finalize())
}
