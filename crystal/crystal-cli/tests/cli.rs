//! Integration tests for the `scryr` binary.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::error::Error;
use std::fs;
#[cfg(unix)]
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
#[cfg(unix)]
use tempfile::TempDir;

/// Resolve the sibling Python manifest workspace used by CLI integration tests.
fn manifest_dir() -> Result<PathBuf, Box<dyn Error>> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../manifest")
        .canonicalize()
        .map_err(|error| format!("manifest workspace should exist: {error}"))?)
}

/// Return a stable sample manifest path that exercises the real Python adapter.
const fn sample_manifest() -> &'static str {
    "tests/samples/open_saas/index.scry"
}

/// Resolve the repo-local Scryr state directory used by CLI tests.
fn scryr_dir() -> Result<PathBuf, Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::create_dir_all(path).map_err(|error| {
        format!(
            "failed to create Scryr test dir {}: {error}",
            path.display()
        )
    })?;
    Ok(path.to_path_buf())
}

/// Build a `scryr` process isolated from ambient Scryr configuration.
fn cli_command() -> Result<Command, Box<dyn Error>> {
    let mut command =
        Command::cargo_bin("scryr").map_err(|error| format!("binary should build: {error}"))?;
    command.env_remove("SCRYR_GRAPHQL_URL");
    command.env_remove("SCRYR_SECRETS_FILE");
    command.env_remove("SCRYR_METRICS_CONNECTIONS_FILE");
    command.env_remove("SCRYR_CLERK_ORG_ID");
    command.env_remove("SCRYR_DIR");
    command.env_remove("SCRYR_ORGANIZATION");
    command.env_remove("SCRYR_GIT_COMMIT_SHA");
    command.env_remove("HOST");
    command.env_remove("PORT");
    command.env_remove("AUTH_MODE");
    Ok(command)
}

#[cfg(unix)]
/// Return a loopback port that is not bound at selection time.
fn unused_loopback_port() -> Result<u16, Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

#[test]
/// Verify the binary help includes the supported top-level commands.
fn help_lists_supported_commands() -> Result<(), Box<dyn Error>> {
    cli_command()?
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("serve"))
        .stdout(predicate::str::contains("generate"))
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("scryr generate"));
    Ok(())
}

#[test]
/// Verify generate help shows the default index.scry invocation.
fn generate_help_shows_index_scry_examples() -> Result<(), Box<dyn Error>> {
    cli_command()?
        .args(["generate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: scryr generate"))
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains(
            "scryr generate upload --path index.scry",
        ))
        .stdout(predicate::str::contains(
            "scryr generate devcontainer --path index.scry",
        ));
    Ok(())
}

#[test]
/// Verify invoking the binary with no args prints help instead of an error.
fn no_args_prints_help() -> Result<(), Box<dyn Error>> {
    cli_command()?
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: scryr"))
        .stdout(predicate::str::contains("serve"))
        .stdout(predicate::str::contains("generate"))
        .stdout(predicate::str::contains("auth"));
    Ok(())
}

#[test]
/// Verify serve help shows server bind options.
fn serve_help_shows_server_options() -> Result<(), Box<dyn Error>> {
    cli_command()?
        .args(["serve", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: scryr serve"))
        .stdout(predicate::str::contains("--host"))
        .stdout(predicate::str::contains("--port"))
        .stdout(predicate::str::contains("--auth-mode"));
    Ok(())
}

#[cfg(unix)]
#[test]
/// Verify `generate` uses index.scry when the manifest file is omitted.
fn generate_defaults_to_index_scry() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let manifest_dir = temp_dir.path().join("manifest");
    let scryr_dir = temp_dir.path().join(".scryr");
    fs::create_dir_all(&manifest_dir)?;
    fs::write(
        manifest_dir.join("index.scry"),
        "sample_manifest = Manifest(name='Sample')\n",
    )?;
    let (uv_installer_path, log_path) = fake_uv_installer(&temp_dir)?;

    cli_command()?
        .env("SCRYR_UV_INSTALLER", &uv_installer_path)
        .args([
            "generate",
            "types",
            "--manifest-dir",
            &manifest_dir.to_string_lossy(),
            "--scryr-dir",
            &scryr_dir.to_string_lossy(),
        ])
        .assert()
        .success();

    let log = fs::read_to_string(&log_path)
        .map_err(|error| format!("fake uv should have been executed: {error}"))?;
    assert!(
        log.contains("index.scry"),
        "default manifest file should be index.scry; log:\n{log}"
    );
    Ok(())
}

#[cfg(unix)]
/// Create a fake uv installer that installs a logging uv executable.
fn fake_uv_installer(temp_dir: &TempDir) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    let installer_path = temp_dir.path().join("install-uv.sh");
    let log_path = temp_dir.path().join("uv.log");
    let script = format!(
        r#"#!/bin/sh
set -eu
mkdir -p "$UV_UNMANAGED_INSTALL"
cat > "$UV_UNMANAGED_INSTALL/uv" <<'UV_EOF'
#!/bin/sh
set -eu
printf '%s|%s|%s|%s\n' "$PWD" "$*" "${{UV_PROJECT_ENVIRONMENT:-}}" "${{UV_CACHE_DIR:-}}" >> "{}"
case "$1" in
  --version)
    printf 'uv 0.12.7\n'
    ;;
  python)
    exit 0
    ;;
  sync)
    exit 0
    ;;
  run)
    flag=""
    for argument in "$@"; do
      flag="$argument"
    done
    if [ "$flag" = "--schema" ]; then
      printf '{{"title":"Manifest"}}\n'
    else
      printf '[{{"variable_name":"sample_manifest","manifest":{{"name":"Sample"}}}},{{"kind":"diagram","variable_name":"diagram","diagram":{{"name":"Sample","manifests":[{{"name":"Sample"}}]}}}}]\n'
    fi
    ;;
  *)
    printf 'unexpected fake uv command: %s\n' "$*" >&2
    exit 1
    ;;
esac
UV_EOF
chmod +x "$UV_UNMANAGED_INSTALL/uv"
"#,
        log_path.display()
    );
    fs::write(&installer_path, script).map_err(|error| {
        format!(
            "failed to write fake uv installer {}: {error}",
            installer_path.display()
        )
    })?;
    let mut permissions = fs::metadata(&installer_path)
        .map_err(|error| {
            format!(
                "failed to stat fake uv installer {}: {error}",
                installer_path.display()
            )
        })?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&installer_path, permissions).map_err(|error| {
        format!(
            "failed to make fake uv installer executable {}: {error}",
            installer_path.display()
        )
    })?;

    Ok((installer_path, log_path))
}

#[test]
/// Verify `generate types` emits JSON metadata for a real manifest sample.
fn generate_types_emits_json_metadata_for_sample_manifest() -> Result<(), Box<dyn Error>> {
    let manifest_dir = manifest_dir()?;
    let scryr_dir = scryr_dir()?;
    let output = cli_command()?
        .args([
            "generate",
            "types",
            "--path",
            sample_manifest(),
            "--manifest-dir",
            &manifest_dir.to_string_lossy(),
            "--scryr-dir",
            &scryr_dir.to_string_lossy(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed = serde_json::from_slice::<Value>(&output)
        .map_err(|error| format!("stdout should be valid json: {error}"))?;
    let blocks = parsed
        .as_array()
        .ok_or("types payload should be an array")?;

    assert!(
        !blocks.is_empty(),
        "types payload should include at least one block"
    );
    assert!(blocks.iter().any(|block| {
        block.get("variable") == Some(&Value::String("postgres_block".to_string()))
    }));
    assert!(blocks.iter().any(|block| {
        block.get("manifest_name") == Some(&Value::String("PostgreSQL".to_string()))
    }));
    assert!(
        scryr_dir.join("bin/uv").exists(),
        "scryr should install a managed uv executable"
    );
    assert!(
        scryr_dir.join("python-envs").exists(),
        "uv should create an isolated managed Scryr Python environment"
    );
    assert!(
        scryr_dir.join("python").exists(),
        "scryr should install uv-managed Python under Scryr state"
    );
    Ok(())
}

#[test]
/// Verify `generate mise` emits the selected Forge as mise.toml.
fn generate_mise_toml_emits_sample_forge() -> Result<(), Box<dyn Error>> {
    let manifest_dir = manifest_dir()?;
    let scryr_dir = scryr_dir()?;
    let output = cli_command()?
        .args([
            "generate",
            "mise",
            "--path",
            "tests/samples/mern/index.scry",
            "--manifest-dir",
            &manifest_dir.to_string_lossy(),
            "--scryr-dir",
            &scryr_dir.to_string_lossy(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output = String::from_utf8(output)
        .map_err(|error| format!("mise.toml output should be valid UTF-8: {error}"))?;

    assert!(output.contains("[tools]"));
    assert!(output.contains("node = { postinstall = \"corepack enable\", version = \"22\" }"));
    assert!(output.contains("[env]"));
    assert!(output.contains("MONGO_URL = \"mongodb://localhost:27017/mern\""));
    assert!(output.contains("[vars]"));
    assert!(output.contains("api_port = 3001"));
    assert!(output.contains("[tasks.\"dev:api\"]"));
    assert!(output.contains("run = \"npm run dev --workspace api\""));
    assert!(!output.contains("Generating manifest artifacts"));
    Ok(())
}

#[test]
/// Verify `generate compose` emits Docker Compose services from the selected Forge.
fn generate_compose_emits_sample_forge_services() -> Result<(), Box<dyn Error>> {
    let manifest_dir = manifest_dir()?;
    let scryr_dir = scryr_dir()?;
    let output = cli_command()?
        .args([
            "generate",
            "compose",
            "--path",
            "tests/samples/mern/index.scry",
            "--manifest-dir",
            &manifest_dir.to_string_lossy(),
            "--scryr-dir",
            &scryr_dir.to_string_lossy(),
            "--forge",
            "MERN Forge",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output = String::from_utf8(output)
        .map_err(|error| format!("compose output should be valid UTF-8: {error}"))?;

    assert!(output.contains("services:"));
    assert!(output.contains("mongodb:"));
    assert!(output.contains("image: \"mongo:8\""));
    assert!(output.contains("- \"27017:27017\""));
    assert!(output.contains("redis:"));
    assert!(output.contains("image: \"redis:7\""));
    assert!(output.contains("volumes:"));
    assert!(!output.contains("Generating manifest artifacts"));
    Ok(())
}

#[test]
/// Verify `generate devcontainer` emits valid devcontainer JSON from the selected Forge.
fn generate_devcontainer_emits_sample_forge_json() -> Result<(), Box<dyn Error>> {
    let manifest_dir = manifest_dir()?;
    let scryr_dir = scryr_dir()?;
    let output = cli_command()?
        .args([
            "generate",
            "devcontainer",
            "--path",
            "tests/samples/mern/index.scry",
            "--manifest-dir",
            &manifest_dir.to_string_lossy(),
            "--scryr-dir",
            &scryr_dir.to_string_lossy(),
            "--forge",
            "MERN Forge",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed = serde_json::from_slice::<Value>(&output)
        .map_err(|error| format!("devcontainer output should be valid json: {error}"))?;

    assert_eq!(parsed["name"], Value::String("MERN Forge".to_string()));
    assert_eq!(
        parsed["image"],
        Value::String("mcr.microsoft.com/devcontainers/base:ubuntu".to_string())
    );
    assert_eq!(
        parsed["containerEnv"]["MONGO_URL"],
        Value::String("mongodb://localhost:27017/mern".to_string())
    );
    assert!(
        parsed["postCreateCommand"]
            .as_str()
            .is_some_and(|command| command.contains("mise run install"))
    );
    Ok(())
}

#[test]
/// Verify Forge-backed generators have explicit success/failure behavior across samples.
fn generate_forge_backed_targets_have_expected_sample_behavior() -> Result<(), Box<dyn Error>> {
    let manifest_dir = manifest_dir()?;
    let scryr_dir = scryr_dir()?;
    for entry in fs::read_dir(manifest_dir.join("tests/samples"))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let sample_path = entry.path().join("index.scry");
        if !sample_path.is_file() {
            continue;
        }
        let relative = sample_path.strip_prefix(&manifest_dir)?.to_string_lossy();
        let sample_name = entry.file_name().to_string_lossy().into_owned();

        let mut devcontainer = cli_command()?;
        devcontainer.args([
            "generate",
            "devcontainer",
            "--path",
            &relative,
            "--manifest-dir",
            &manifest_dir.to_string_lossy(),
            "--scryr-dir",
            &scryr_dir.to_string_lossy(),
        ]);

        if sample_name == "mern" {
            devcontainer.assert().success();
            cli_command()?
                .args([
                    "generate",
                    "compose",
                    "--path",
                    &relative,
                    "--manifest-dir",
                    &manifest_dir.to_string_lossy(),
                    "--scryr-dir",
                    &scryr_dir.to_string_lossy(),
                ])
                .assert()
                .success()
                .stdout(predicate::str::contains("mongodb:"))
                .stdout(predicate::str::contains("redis:"));
        } else {
            devcontainer
                .assert()
                .failure()
                .stderr(predicate::str::contains(
                    "Manifest file did not define any public Forge instances",
                ));
        }
    }
    Ok(())
}

#[cfg(unix)]
#[test]
/// Verify generation defaults to the local GraphQL endpoint before persistence.
fn generate_without_graphql_url_uses_default_local_endpoint() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let manifest_dir = temp_dir.path().join("manifest");
    let scryr_dir = temp_dir.path().join(".scryr");
    let port = unused_loopback_port()?;
    let expected_graphql_url = format!("http://127.0.0.1:{port}/graphql");
    fs::create_dir_all(&manifest_dir)?;
    fs::write(
        manifest_dir.join("sample.py"),
        "sample_manifest = Manifest(name='Sample')\n",
    )?;
    let (uv_installer_path, log_path) = fake_uv_installer(&temp_dir)?;

    cli_command()?
        .env("SCRYR_UV_INSTALLER", &uv_installer_path)
        .env("PORT", port.to_string())
        .args([
            "generate",
            "upload",
            "--path",
            "sample.py",
            "--manifest-dir",
            &manifest_dir.to_string_lossy(),
            "--scryr-dir",
            &scryr_dir.to_string_lossy(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "Failed to call GraphQL endpoint {expected_graphql_url}",
        )))
        .stderr(predicate::str::contains("Run `scryr auth login`").not());

    let log = fs::read_to_string(&log_path)
        .map_err(|error| format!("fake uv should have been executed: {error}"))?;
    assert!(
        log.contains("--version"),
        "managed uv verification should execute uv --version; log:\n{log}"
    );
    assert!(
        log.contains("python install 3.14 --managed-python --install-dir"),
        "manifest generation should install Scryr-managed Python before running the adapter; log:\n{log}"
    );
    assert!(
        !log.contains("sync --no-dev --locked"),
        "manifest preparation should not execute uv sync without scryr.toml; log:\n{log}"
    );
    assert!(
        log.contains("run") && log.contains("scryr_adapter.py"),
        "manifest generation should execute the inline uv adapter script; log:\n{log}"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
/// Only scryr.toml opts generation into the manifest directory's Python project.
fn generate_requires_scryr_toml_to_use_project_dependencies() -> Result<(), Box<dyn Error>> {
    for (has_pyproject, has_scryr, expected_project_mode) in [
        (false, false, false),
        (true, false, false),
        (true, true, true),
    ] {
        let temp_dir = TempDir::new()?;
        let manifest_dir = temp_dir.path().join("manifest");
        let state_dir = temp_dir.path().join(".scryr");
        fs::create_dir_all(&manifest_dir)?;
        fs::write(
            manifest_dir.join("index.scry"),
            "sample = Manifest(name='Sample')\n",
        )?;
        // Parent project markers must not opt child directories into project mode.
        fs::write(temp_dir.path().join("scryr.toml"), "")?;
        if has_pyproject {
            fs::write(
                manifest_dir.join("pyproject.toml"),
                "[project]\nname = 'unrelated'\n",
            )?;
        }
        if has_scryr {
            fs::write(manifest_dir.join("scryr.toml"), "")?;
        }
        let (installer, log_path) = fake_uv_installer(&temp_dir)?;
        cli_command()?
            .env("SCRYR_UV_INSTALLER", installer)
            .args(["generate", "types", "--manifest-dir"])
            .arg(&manifest_dir)
            .arg("--scryr-dir")
            .arg(&state_dir)
            .assert()
            .success();
        let log = fs::read_to_string(log_path)?;
        assert_eq!(
            log.contains("sync --no-dev --locked"),
            expected_project_mode,
            "{log}"
        );
        assert_eq!(
            log.contains("run --no-dev --no-sync python -m scryr.cli"),
            expected_project_mode,
            "{log}"
        );
        assert_eq!(
            log.contains("run --no-project"),
            !expected_project_mode,
            "{log}"
        );
        assert_eq!(
            log.contains("scryr_adapter.py"),
            !expected_project_mode,
            "{log}"
        );
    }
    Ok(())
}

#[cfg(unix)]
#[test]
/// A marker without local Python metadata fails before uv can discover a parent project.
fn scryr_project_requires_its_own_python_metadata() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let manifest_dir = temp_dir.path().join("manifest");
    fs::create_dir_all(&manifest_dir)?;
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[project]\nname = 'parent'\n",
    )?;
    fs::write(manifest_dir.join("scryr.toml"), "")?;
    fs::write(
        manifest_dir.join("index.scry"),
        "sample = Manifest(name='Sample')\n",
    )?;
    let (installer, log_path) = fake_uv_installer(&temp_dir)?;
    cli_command()?
        .env("SCRYR_UV_INSTALLER", installer)
        .args(["generate", "types", "--manifest-dir"])
        .arg(&manifest_dir)
        .arg("--scryr-dir")
        .arg(temp_dir.path().join(".scryr"))
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "pyproject.toml is missing from the same directory",
        ));
    let log = fs::read_to_string(log_path)?;
    assert!(!log.contains("sync --no-dev --locked"), "{log}");
    assert!(!log.contains("python install"), "{log}");
    Ok(())
}

/// Migration is a one-shot, repeatable command and does not require an HTTP server.
#[test]
fn migrate_creates_schema_and_is_repeatable() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("migrate.db");
    for _ in 0..2 {
        cli_command()?
            .env_remove("DATABASE_URL")
            .env_remove("TURSO_DATABASE_URL")
            .env_remove("TURSO_AUTH_TOKEN")
            .env_remove("LIBSQL_AUTH_TOKEN")
            .env("SCRYR_SQLITE_PATH", &database)
            .arg("migrate")
            .assert()
            .success()
            .stdout(predicate::str::contains("Database schema is up to date"));
    }
    assert!(fs::metadata(database)?.len() > 0);
    Ok(())
}

#[test]
fn workflow_commands_have_help_and_serve_requires_explicit_server_only()
-> Result<(), Box<dyn Error>> {
    for args in [
        vec!["check"],
        vec!["format"],
        vec!["lint"],
        vec!["push"],
        vec!["export", "json"],
        vec!["inspect", "types"],
        vec!["query"],
        vec!["report", "tests"],
        vec!["serve"],
    ] {
        cli_command()?.args(args).arg("--help").assert().success();
    }
    cli_command()?
        .args(["serve", "--server-only", "--watch"])
        .assert()
        .failure();
    cli_command()?.args(["query"]).assert().failure();
    Ok(())
}

#[test]
/// Typed source IDs, per-card query selection, and redacted TOML checks use the native workflow.
fn typed_cards_and_toml_credentials_are_checked() -> Result<(), Box<dyn Error>> {
    let folder = tempfile::tempdir()?;
    let source = folder.path().join("index.scry");
    fs::copy(
        manifest_dir()?.join("examples/typed_integrations/index.scry"),
        &source,
    )?;
    let secrets = folder.path().join("scryr.secrets.toml");
    fs::write(
        &secrets,
        "[authentication.grafana_authentication]\nusername = \"test-user\"\ntoken = \"sensitive-test-value\"\n[authentication.posthog_authentication]\napi_key = \"test-api-key\"\n",
    )?;
    for command in ["format", "check"] {
        cli_command()?
            .args([command, "--path"])
            .arg(&source)
            .arg("--manifest-dir")
            .arg(manifest_dir()?)
            .arg("--scryr-dir")
            .arg(scryr_dir()?)
            .env("SCRYR_SECRETS_FILE", &secrets)
            .assert()
            .success();
    }
    let listing = cli_command()?
        .args(["query", "--list", "--json", "--path"])
        .arg(&source)
        .arg("--manifest-dir")
        .arg(manifest_dir()?)
        .arg("--scryr-dir")
        .arg(scryr_dir()?)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let rows: Value = serde_json::from_slice(&listing)?;
    assert!(rows.as_array().is_some_and(|rows| {
        rows.iter()
            .any(|r| r["card"] == "grafana_performance" && r["manifestId"] == "api")
    }));
    fs::write(
        &secrets,
        "[authentication.grafana_authentication]\ntoken = \"sensitive-test-value\"\n",
    )?;
    cli_command()?
        .args(["check", "--path"])
        .arg(&source)
        .arg("--manifest-dir")
        .arg(manifest_dir()?)
        .arg("--scryr-dir")
        .arg(scryr_dir()?)
        .env("SCRYR_SECRETS_FILE", &secrets)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "authentication.grafana_authentication",
        ))
        .stderr(predicate::str::contains("sensitive-test-value").not());
    Ok(())
}
