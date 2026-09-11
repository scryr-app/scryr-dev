//! Manifest generation command workflow.

pub(crate) mod executor;
mod outputs;
pub(crate) mod upload;

use crate::args::GenerateRequest;
use crate::manifest_paths::resolve_manifest_file;

use executor::prepare_manifest_executor;
use outputs::render_stdout_target;

/// Execute manifest generation.
pub(super) async fn run(args: &GenerateRequest) -> Result<(), String> {
    if args.output == crate::args::GenerateOutput::Upload {
        let project = super::workflow::Project::from_request(args.clone())?;
        let checked = project.check()?;
        super::workflow::publish(&project, checked).await?;
        return Ok(());
    }
    let manifest_dir = args.manifest_dir.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve manifest directory {}: {error}",
            args.manifest_dir.display()
        )
    })?;
    let manifest_file = resolve_manifest_file(&manifest_dir, &args.manifest_file)?;
    let manifest_executor = prepare_manifest_executor(args, &manifest_dir)?;

    if render_stdout_target(
        args.output,
        args,
        &manifest_executor,
        &manifest_dir,
        &manifest_file,
    )? {
        return Ok(());
    }

    Ok(())
}
