//! Start the local UI and load validated manifest artifacts.
use super::workflow::{Project, publish};
use crate::args::ServerArgs;
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
    let server = crystal_server::server::start(args.server.clone())
        .await
        .map_err(|e| e.to_string())?;
    tokio::pin!(server);
    tokio::select! {
        result = &mut server => result.map_err(|e| e.to_string()),
        () = load_loop(args) => server.await.map_err(|e| e.to_string()),
    }
}

/// Wait for this server, then refresh only when source contents change.
async fn load_loop(args: &ServerArgs) {
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
                    (publish(&project, checked).await, revision)
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
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// Fingerprint source contents, including creation/deletion, without executing code.
fn fingerprint(args: &crate::args::GenerateCommonArgs) -> String {
    let file = args.manifest_dir.join(&args.manifest_file);
    crate::manifest_source::collect_manifest_source_files(&file)
        .and_then(|files| serde_json::to_string(&files).map_err(|e| e.to_string()))
        .unwrap_or_else(|error| error)
}
