#![allow(clippy::redundant_pub_crate)]

pub(crate) mod adapter;
mod embedded_sdk;
pub(crate) mod environment;
pub(crate) mod output;
mod uv_runner;

pub(crate) use adapter::{prepare_manifest_python_environment, run_manifest_python_command};
pub(crate) use environment::ManifestPythonEnvironment;
pub(crate) use output::{ManifestPythonMode, PythonAdapterOutput, format_manifest_adapter_output};

#[cfg(test)]
mod tests {
    use super::adapter::{
        MANIFEST_PYTHON_MODULE, prepare_manifest_python_environment, run_manifest_python_command,
    };
    use super::environment::{ManifestPythonEnvironment, isolated_environment_root};
    use super::output::ManifestPythonMode;
    use super::uv_runner::{project_python_executable, venv_python_executable};
    use crate::uv::ensure_managed_uv;
    use serde_json::Value;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    fn test_python_environment() -> Result<ManifestPythonEnvironment, String> {
        let scryr_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        fs::create_dir_all(scryr_dir).map_err(|error| {
            format!(
                "failed to create test Scryr dir {}: {error}",
                scryr_dir.display()
            )
        })?;
        let managed_uv = ensure_managed_uv(scryr_dir)?;

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifest");
        Ok(ManifestPythonEnvironment::from_manifest_dir(
            scryr_dir,
            &manifest_dir,
            managed_uv.executable(),
        ))
    }

    #[test]
    fn manifest_python_module_points_at_manifest_library_entrypoint() {
        assert_eq!(MANIFEST_PYTHON_MODULE, "scryr.cli");
    }

    #[test]
    fn manifest_python_mode_flags_match_python_entrypoint() {
        assert_eq!(ManifestPythonMode::Json.as_flag(), "--json");
        assert_eq!(ManifestPythonMode::Types.as_flag(), "--types");
        assert_eq!(ManifestPythonMode::Schema.as_flag(), "--schema");
    }

    #[test]
    fn isolated_environment_root_is_stable_per_manifest_project() {
        let scryr_dir = Path::new("/tmp/.scryr");
        let first = isolated_environment_root(scryr_dir, Path::new("/workspace/manifest-a"));
        let first_again = isolated_environment_root(scryr_dir, Path::new("/workspace/manifest-a"));
        let second = isolated_environment_root(scryr_dir, Path::new("/workspace/manifest-b"));

        assert_eq!(first, first_again);
        assert_ne!(first, second);
        assert!(first.starts_with(scryr_dir.join("python-envs")));
    }

    #[test]
    fn project_python_executable_prefers_project_virtual_environment() -> Result<(), String> {
        let temp_dir = tempfile::TempDir::new()
            .map_err(|error| format!("failed to create temp dir: {error}"))?;
        let venv_python = venv_python_executable(&temp_dir.path().join(".venv"));
        let parent = venv_python.parent().ok_or_else(|| {
            format!(
                "venv Python path has no parent directory: {}",
                venv_python.display()
            )
        })?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        fs::write(&venv_python, "")
            .map_err(|error| format!("failed to write {}: {error}", venv_python.display()))?;

        assert_eq!(
            project_python_executable(temp_dir.path()),
            Some(venv_python)
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn inline_adapter_script_declares_local_scryr_dependency_with_uv_metadata() -> Result<(), String>
    {
        let temp_dir = tempfile::TempDir::new()
            .map_err(|error| format!("failed to create temp dir: {error}"))?;
        let scryr_dir = temp_dir.path().join(".scryr");
        let manifest_dir = temp_dir.path().join("project");
        fs::create_dir_all(&scryr_dir)
            .map_err(|error| format!("failed to create Scryr dir: {error}"))?;
        fs::create_dir_all(&manifest_dir)
            .map_err(|error| format!("failed to create manifest dir: {error}"))?;
        let uv_path = temp_dir.path().join("uv");
        fs::write(
            &uv_path,
            "#!/bin/sh\nif [ \"$1\" = \"python\" ]; then exit 0; fi\nexit 1\n",
        )
        .map_err(|error| format!("failed to write fake uv: {error}"))?;
        let mut permissions = fs::metadata(&uv_path)
            .map_err(|error| format!("failed to stat fake uv: {error}"))?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&uv_path, permissions)
            .map_err(|error| format!("failed to chmod fake uv: {error}"))?;
        let environment =
            ManifestPythonEnvironment::from_manifest_dir(&scryr_dir, &manifest_dir, &uv_path);

        prepare_manifest_python_environment(&manifest_dir, &environment)?;

        let adapter = fs::read_to_string(environment.inline_adapter_path())
            .map_err(|error| format!("failed to read inline adapter: {error}"))?;
        assert!(adapter.contains("# /// script"));
        assert!(adapter.contains("\"scryr @ file://"));
        assert!(adapter.contains("from scryr.cli import main"));
        let sdk_package_dir = fs::read_dir(&environment.root_path)
            .map_err(|error| format!("failed to read environment root: {error}"))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("scryr-sdk-"))
            })
            .ok_or_else(|| "expected content-addressed embedded SDK package".to_string())?;
        assert!(sdk_package_dir.join("pyproject.toml").is_file());
        assert!(sdk_package_dir.join("src/scryr/cli.py").is_file());
        Ok(())
    }

    #[test]
    fn sample_manifest_emits_expected_blocks() -> Result<(), String> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifest");
        let manifest_file = manifest_dir.join("tests/samples/open_saas/index.scry");
        let environment = test_python_environment()?;
        prepare_manifest_python_environment(&manifest_dir, &environment)?;

        let manifest_json = run_manifest_python_command(
            &manifest_dir,
            &manifest_file,
            ManifestPythonMode::Json,
            &environment,
        )?;
        let parsed = serde_json::from_str::<Value>(&manifest_json)
            .map_err(|error| format!("expected manifest artifact to be valid JSON: {error}"))?;
        let manifests = parsed
            .get("manifests")
            .and_then(Value::as_array)
            .ok_or_else(|| "expected manifest array".to_string())?;

        assert!(manifest_json.contains("\"name\": \"PostgreSQL\""));
        assert!(manifest_json.contains("\"name\": \"Wasp Server\""));
        assert!(
            manifests
                .iter()
                .all(|manifest| manifest.get("variable_name").is_some())
        );
        assert!(
            manifests
                .iter()
                .all(|manifest| manifest.get("line_number").is_some())
        );
        Ok(())
    }

    #[test]
    fn sample_manifest_emits_forges() -> Result<(), String> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifest");
        let manifest_file = manifest_dir.join("tests/samples/mern/index.scry");
        let environment = test_python_environment()?;
        prepare_manifest_python_environment(&manifest_dir, &environment)?;

        let manifest_json = run_manifest_python_command(
            &manifest_dir,
            &manifest_file,
            ManifestPythonMode::Json,
            &environment,
        )?;
        let parsed = serde_json::from_str::<Value>(&manifest_json)
            .map_err(|error| format!("expected manifest artifact to be valid JSON: {error}"))?;
        let forges = parsed
            .get("forges")
            .and_then(Value::as_array)
            .ok_or_else(|| "expected forge array".to_string())?;

        assert_eq!(forges.len(), 1);
        assert!(manifest_json.contains("\"name\": \"MERN Forge\""));
        assert!(
            forges
                .iter()
                .all(|forge| forge.get("variable_name").is_some())
        );
        assert!(
            forges
                .iter()
                .all(|forge| forge.get("line_number").is_some())
        );
        Ok(())
    }

    #[test]
    fn sample_manifest_emits_diagrams() -> Result<(), String> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifest");
        let manifest_file = manifest_dir.join("tests/samples/mern/index.scry");
        let environment = test_python_environment()?;
        prepare_manifest_python_environment(&manifest_dir, &environment)?;

        let manifest_json = run_manifest_python_command(
            &manifest_dir,
            &manifest_file,
            ManifestPythonMode::Json,
            &environment,
        )?;
        let parsed = serde_json::from_str::<Value>(&manifest_json)
            .map_err(|error| format!("expected manifest artifact to be valid JSON: {error}"))?;
        let diagrams = parsed
            .get("diagrams")
            .and_then(Value::as_array)
            .ok_or_else(|| "expected diagram array".to_string())?;

        assert_eq!(diagrams.len(), 4);
        assert!(manifest_json.contains("\"name\": \"MERN\""));
        assert!(manifest_json.contains("\"name\": \"MERN Customer Path\""));
        assert!(manifest_json.contains("\"name\": \"MERN Terraform Managed\""));
        assert!(manifest_json.contains("\"name\": \"MERN Operability\""));
        assert!(
            diagrams
                .iter()
                .all(|diagram| diagram.get("variable_name").is_some())
        );
        assert!(
            diagrams
                .iter()
                .all(|diagram| diagram.get("line_number").is_some())
        );
        Ok(())
    }
}
