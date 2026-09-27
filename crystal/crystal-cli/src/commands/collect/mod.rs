//! Typed local collector adapters and a single workspace execution owner.
#![allow(clippy::missing_docs_in_private_items)]
mod dependencies;
mod files;
mod integrations;
mod openmetrics;
mod process;
mod reports;
mod runtime;
use crate::args::{CollectArgs, CollectCommand};
use runtime::{Location, Request, Selection};
pub(crate) use runtime::{Plan, supervise};

#[allow(clippy::too_many_lines)] // CLI selection and human/JSON rendering of the six subcommands.
pub(super) async fn run(args: CollectArgs) -> Result<(), String> {
    let options = match &args.command {
        CollectCommand::List(o)
        | CollectCommand::Doctor(o)
        | CollectCommand::Run(o)
        | CollectCommand::Status(o)
        | CollectCommand::Pause(o)
        | CollectCommand::Resume(o) => o,
    };
    let location = Location::new(
        &options.common.manifest_dir,
        options.common.scryr_dir.as_deref(),
    )?;
    let selection = Selection {
        manifest: options.manifest.clone(),
        section: options.section.clone(),
        collector: options.collector.clone(),
    };
    let request = match &args.command {
        CollectCommand::Run(_) => Some(Request::Run {
            selection: selection.clone(),
        }),
        CollectCommand::Status(_) => Some(Request::Status),
        CollectCommand::Pause(_) => Some(Request::Pause { paused: true }),
        CollectCommand::Resume(_) => Some(Request::Pause { paused: false }),
        _ => None,
    };
    if let Some(request) = request {
        if let Some(response) = runtime::contact(&location, &request).await? {
            if options.json {
                println!(
                    "{}",
                    serde_json::to_string(&response).map_err(|e| e.to_string())?
                );
            } else {
                println!("{}", response.message);
                if matches!(args.command, CollectCommand::Status(_)) {
                    for status in response.statuses {
                        println!(
                            "{} / {} / {}: {:?} {}",
                            status.manifest_id,
                            status.section.as_str(),
                            status.collector_id,
                            status.state,
                            status.message.unwrap_or_default()
                        );
                    }
                }
            }
            return if response.error {
                Err("collector request failed".into())
            } else {
                Ok(())
            };
        }
        if matches!(
            args.command,
            CollectCommand::Pause(_) | CollectCommand::Resume(_)
        ) {
            return Err("no active collector owner; start scryr serve".into());
        }
        if matches!(args.command, CollectCommand::Status(_)) {
            let file = location.state.join("status.json");
            if file.exists() {
                println!("{}", files::read(&file)?);
            } else {
                println!("No local collector attempts yet; run scryr serve or scryr collect run");
            }
            return Ok(());
        }
    }
    let common = options.common.clone();
    let plan = tokio::task::spawn_blocking(move || {
        let project = super::workflow::Project::new(common)?;
        let checked = project.check()?;
        Plan::parse(&checked.json, &project.root, &project.file)
    })
    .await
    .map_err(|e| e.to_string())??;
    if matches!(args.command, CollectCommand::Run(_)) {
        return runtime::once(location, plan, selection).await;
    }
    let mut rows = Vec::new();
    let mut failed = false;
    for d in &plan.declarations {
        if selection
            .manifest
            .as_ref()
            .is_some_and(|s| *s != d.manifest_id)
            || selection
                .section
                .as_ref()
                .is_some_and(|s| *s != d.section.as_str())
            || selection
                .collector
                .as_ref()
                .is_some_and(|s| s != d.config.id())
        {
            continue;
        }
        let result = if matches!(args.command, CollectCommand::Doctor(_)) {
            match integrations::doctor(&d.config, &location.root, &plan.source_directory).await {
                Ok(version) => serde_json::json!({"ready":true,"version":version}),
                Err(error) => {
                    failed = true;
                    serde_json::json!({"ready":false,"message":error,"install":"Install the integration's open-source CLI in your project environment or PATH; Scryr does not auto-install tools"})
                }
            }
        } else {
            serde_json::json!({"executable":integrations::executable(&d.config),"configuration":d.config})
        };
        rows.push(serde_json::json!({"manifest":d.manifest_id,"section":d.section,"collector":d.config.id(),"integration":d.config.kind(),"details":result}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?
    );
    if failed {
        Err("some collector tools need attention".into())
    } else {
        Ok(())
    }
}
