//! sprites.dev CLI invocation for manifest execution.

use super::bundle::SpriteExecutionPlan;
use super::environment::SpriteExecutionEnvironment;
use super::script::remote_execution_script;
use crate::manifest_python::ManifestPythonMode;
use std::ffi::OsString;
use std::process::Command;

/// Invoke `sprite exec` for one adapter mode.
pub(super) fn run_sprite_exec(
    execution_plan: &SpriteExecutionPlan,
    mode: ManifestPythonMode,
    environment: &SpriteExecutionEnvironment,
) -> Result<String, String> {
    let script = remote_execution_script(execution_plan, mode);
    let mut args = vec![OsString::from("exec"), OsString::from("--no-port-forward")];
    if let Some(organization) = &environment.organization {
        args.push(OsString::from("-o"));
        args.push(OsString::from(organization));
    }
    args.push(OsString::from("-s"));
    args.push(OsString::from(&environment.sprite_name));
    args.push(OsString::from("--file"));
    args.push(OsString::from(format!(
        "{}:{}",
        execution_plan.bundle_path.display(),
        execution_plan.remote_bundle_path
    )));
    args.push(OsString::from("--"));
    args.push(OsString::from("sh"));
    args.push(OsString::from("-lc"));
    args.push(OsString::from(script));

    let output = Command::new(&environment.sprite_executable)
        .args(args)
        .output()
        .map_err(|error| {
            format!(
                "Failed to execute {}. Install and authenticate the sprites.dev CLI, or set SCRYR_SPRITE_BIN: {error}",
                environment.sprite_executable.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let organization = environment
            .organization
            .as_deref()
            .map_or(String::new(), |org| format!(" in organization {org}"));
        let verification_hint = environment.organization.as_deref().map_or_else(
            || {
                format!(
                    "Verify it exists with `sprite list`, or create it with `sprite create {}`.",
                    environment.sprite_name
                )
            },
            |org| {
                format!(
                    "Verify it exists with `sprite list -o {org}`, or create it with `sprite create -o {org} {}`.",
                    environment.sprite_name
                )
            },
        );
        return Err(format!(
            "sprite exec failed for Sprite {}{organization}: {stderr}{verification_hint}",
            environment.sprite_name,
        ));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| format!("sprite exec produced invalid UTF-8 in stdout: {error}"))
}
