//! Top-level CLI command dispatch.

mod auth;
mod collect;
mod generate;
mod migrate;
mod serve;
mod workflow;

use crate::args::ResolvedCommand;

/// Execute a fully resolved top-level command.
pub(crate) async fn run(command: ResolvedCommand) -> Result<(), String> {
    match command {
        ResolvedCommand::Check(args) => workflow::Project::new(args)?.check().map(|_| ()),
        ResolvedCommand::Format(args) => workflow::Project::new(args.common)?.tool(if args.check {
            "--format-check"
        } else {
            "--format"
        }),
        ResolvedCommand::Lint(args) => workflow::Project::new(args.common)?.tool(if args.fix {
            "--lint-fix"
        } else {
            "--lint"
        }),
        ResolvedCommand::Push(args) => workflow::push(args).await,
        ResolvedCommand::Inspect(args) => generate::run(&args.request()).await,
        ResolvedCommand::Export(args) => {
            let request = args.request();
            if request.output == crate::args::GenerateOutput::ArtifactJson {
                let checked = workflow::Project::from_request(request)?.check()?;
                println!("{}", checked.json);
                Ok(())
            } else {
                generate::run(&request).await
            }
        }
        ResolvedCommand::Collect(args) => collect::run(args).await,
        ResolvedCommand::Migrate => migrate::run().await,
        ResolvedCommand::Serve(args) => serve::run(&args).await,
        ResolvedCommand::Auth(args) => auth::run(&args).await,
    }
}
