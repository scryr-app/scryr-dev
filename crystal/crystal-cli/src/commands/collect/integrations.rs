//! Fixed adapters for installed tools; collection never uses a shell command template.
#![allow(clippy::missing_docs_in_private_items)]
use super::{dependencies, files, openmetrics, process, reports};
use chrono::Utc;
use crystal_core::collectors::{CollectorConfig, CollectorConfig as C, CollectorDeclaration};
use crystal_core::evidence::{
    BenchmarkResult, CheckResult, CoverageResult, Diagnostic, EvidenceResult, GitResult,
    InventoryResult, MetricLabel, MetricSample, MetricsResult, PullRequest, PullRequestsResult,
    WorkflowRun, WorkflowsResult,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone)]
pub(super) struct InventoryArtifact {
    pub path: PathBuf,
    pub inventory: InventoryResult,
    pub revision: String,
    pub input: String,
}
pub(super) struct Collected {
    pub result: EvidenceResult,
    pub version: Option<String>,
    pub inventory: Option<InventoryArtifact>,
    pub snapshot: Option<openmetrics::Snapshot>,
    pub source_updated_at: Option<chrono::DateTime<Utc>>,
}

pub(super) const fn executable(config: &CollectorConfig) -> Option<&'static str> {
    use CollectorConfig as C;
    match config {
        C::GitStatus(_) => Some("git"),
        C::GithubPullRequests(_) | C::GithubActions(_) => Some("gh"),
        C::Ruff(_) => Some("ruff"),
        C::Biome(_) => Some("biome"),
        C::Clippy(_) => Some("cargo"),
        C::MiseTask(_) => Some("mise"),
        C::DockerStats(_) => Some("docker"),
        C::Pytest(_) => Some("pytest"),
        C::Vitest(_) => Some("vitest"),
        C::Nextest(_) => Some("cargo-nextest"),
        C::SyftInventory(_) => Some("syft"),
        C::GrypeScan(_) => Some("grype"),
        C::GrantLicense(_) => Some("grant"),
        C::Hyperfine(_) => Some("hyperfine"),
        C::Openmetrics(_) | C::Junit(_) | C::Lcov(_) | C::Cobertura(_) => None,
    }
}
fn environment(config: &CollectorConfig) -> Result<BTreeMap<String, String>, String> {
    let mut env = BTreeMap::new();
    for (name, reference) in &config.common().env {
        env.insert(
            name.clone(),
            std::env::var(&reference.name)
                .map_err(|_| format!("missing environment variable {}", reference.name))?,
        );
    }
    if let CollectorConfig::Openmetrics(c) = config
        && let Some(auth) = &c.auth
    {
        env.insert(
            "SCRYR_METRICS_BEARER".into(),
            std::env::var(&auth.name)
                .map_err(|_| format!("missing environment variable {}", auth.name))?,
        );
    }
    if matches!(
        config,
        CollectorConfig::GithubPullRequests(_) | CollectorConfig::GithubActions(_)
    ) {
        for name in ["GH_TOKEN", "GITHUB_TOKEN", "GH_CONFIG_DIR"] {
            if let Ok(value) = std::env::var(name) {
                env.entry(name.into()).or_insert(value);
            }
        }
        env.insert("GH_PROMPT_DISABLED".into(), "1".into());
    }
    Ok(env)
}
pub(super) async fn doctor(
    config: &CollectorConfig,
    root: &Path,
    source_directory: &Path,
) -> Result<Option<String>, String> {
    let cwd = process::directory(root, source_directory, config.directory())?;
    let env = environment(config)?;
    if let CollectorConfig::Openmetrics(c) = config
        && let Some(auth) = &c.auth
    {
        std::env::var(&auth.name)
            .map_err(|_| format!("missing environment variable {}", auth.name))?;
    }
    let Some(name) = executable(config) else {
        return Ok(None);
    };
    let path = process::resolve(name, &cwd)?;
    let version = if matches!(config, CollectorConfig::Nextest(_)) {
        let out = process::run(
            &path,
            &["nextest".into(), "--version".into()],
            &cwd,
            &env,
            Duration::from_secs(10),
        )
        .await?;
        process::success(&out, &[0])?;
        out.stdout.trim().into()
    } else {
        process::version(&path, &cwd, &env).await?
    };
    if let Some(required) = config
        .common()
        .tool
        .as_ref()
        .and_then(|r| r.version.as_ref())
    {
        process::check_version(&version, required)?;
    }
    if matches!(
        config,
        CollectorConfig::GithubPullRequests(_) | CollectorConfig::GithubActions(_)
    ) {
        let out = process::run(
            &path,
            &["auth".into(), "status".into()],
            &cwd,
            &env,
            Duration::from_secs(15),
        )
        .await?;
        if out.code != 0 {
            return Err("needs login: run gh auth login in your terminal".into());
        }
    }
    Ok(Some(version))
}

struct Runner<'a> {
    executable: PathBuf,
    cwd: PathBuf,
    env: BTreeMap<String, String>,
    config: &'a CollectorConfig,
}
impl Runner<'_> {
    async fn run(&self, args: Vec<String>, codes: &[i32]) -> Result<process::Output, String> {
        let out = process::run(
            &self.executable,
            &args,
            &self.cwd,
            &self.env,
            Duration::from_secs_f64(self.config.timeout()),
        )
        .await?;
        process::success(&out, codes)?;
        Ok(out)
    }
}
fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}
fn json_output(out: &process::Output) -> Result<Value, String> {
    serde_json::from_str(&out.stdout).map_err(|e| format!("invalid tool JSON: {e}"))
}
fn array(value: &Value) -> Result<&Vec<Value>, String> {
    value
        .as_array()
        .ok_or_else(|| "expected a JSON array".into())
}
fn string(value: &Value, key: &str) -> Result<String, String> {
    value[key]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("missing tool field {key}"))
}
fn optional(value: &Value, key: &str) -> Option<String> {
    value[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}
fn diagnostic(
    mut message: String,
    path: Option<String>,
    line: Option<u64>,
    severity: &str,
) -> Diagnostic {
    if let Some((end, _)) = message.char_indices().nth(2000) {
        message.truncate(end);
    }
    Diagnostic {
        message,
        path,
        line,
        severity: severity.into(),
    }
}
fn check(name: &str, out: &process::Output, diagnostics: Vec<Diagnostic>) -> EvidenceResult {
    EvidenceResult::Check(CheckResult {
        name: name.into(),
        passed: out.code == 0,
        diagnostics,
        duration_seconds: out.duration,
    })
}
fn reject_overrides(arguments: &[String], reserved: &[&str]) -> Result<(), String> {
    if arguments.iter().any(|a| {
        reserved
            .iter()
            .any(|r| a == r || a.starts_with(&format!("{r}=")) || a.starts_with(&format!("{r}.")))
    }) {
        return Err("collector arguments cannot override generated report configuration".into());
    }
    Ok(())
}

#[allow(clippy::too_many_lines, clippy::too_many_arguments)] // Exhaustive adapters share explicit project/source boundaries and per-run artifacts.
pub(super) async fn execute(
    declaration: &CollectorDeclaration,
    root: &Path,
    source_directory: &Path,
    run_dir: &Path,
    upstream: Option<&InventoryArtifact>,
    previous: Option<&openmetrics::Snapshot>,
    input: &str,
    forge_config: Option<&Path>,
) -> Result<Collected, String> {
    let c = &declaration.config;
    let version = doctor(c, root, source_directory).await?;
    let cwd = process::directory(root, source_directory, c.directory())?;
    let runner = Runner {
        executable: executable(c)
            .map(|n| process::resolve(n, &cwd))
            .transpose()?
            .unwrap_or_default(),
        cwd,
        env: environment(c)?,
        config: c,
    };
    let mut collected = Collected {
        result: EvidenceResult::Coverage(CoverageResult {
            suite: String::new(),
            covered: 0,
            total: 0,
        }),
        version,
        inventory: None,
        snapshot: None,
        source_updated_at: files::source_time(root, source_directory, c)?,
    };
    collected.result = match c {
        C::GitStatus(_) => EvidenceResult::Git(git(&runner).await?),
        C::GithubPullRequests(v) => {
            let out = runner
                .run(
                    args(&[
                        "pr",
                        "list",
                        "--repo",
                        &v.repository,
                        "--limit",
                        &v.limit.to_string(),
                        "--json",
                        "number,title,url,state,reviewDecision,headRefName,headRefOid",
                    ]),
                    &[0],
                )
                .await?;
            let value = json_output(&out)?;
            let list = array(&value)?;
            let items = list
                .iter()
                .map(|p| {
                    Ok(PullRequest {
                        number: p["number"].as_u64().ok_or("missing PR number")?,
                        title: string(p, "title")?,
                        url: string(p, "url")?,
                        state: string(p, "state")?,
                        review_decision: optional(p, "reviewDecision"),
                        head_ref_name: optional(p, "headRefName"),
                        head_ref_oid: optional(p, "headRefOid"),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            EvidenceResult::PullRequests(PullRequestsResult {
                repository: v.repository.clone(),
                complete: items.len() < (v.limit as usize),
                items,
            })
        }
        C::GithubActions(v) => {
            let mut arguments = args(&[
                "run",
                "list",
                "--repo",
                &v.repository,
                "--limit",
                &v.limit.to_string(),
                "--json",
                "databaseId,attempt,name,status,conclusion,headBranch,headSha,url,updatedAt",
            ]);
            if let Some(branch) = &v.branch {
                arguments.extend(args(&["--branch", branch]));
            }
            if let Some(workflow) = &v.workflow {
                arguments.extend(args(&["--workflow", workflow]));
            }
            let out = runner.run(arguments, &[0]).await?;
            let value = json_output(&out)?;
            let items = array(&value)?
                .iter()
                .map(|p| {
                    Ok(WorkflowRun {
                        run_id: p["databaseId"]
                            .as_u64()
                            .ok_or("missing workflow ID")?
                            .to_string(),
                        attempt: u32::try_from(p["attempt"].as_u64().ok_or("missing run attempt")?)
                            .map_err(|e| e.to_string())?,
                        name: string(p, "name")?,
                        status: string(p, "status")?,
                        conclusion: optional(p, "conclusion"),
                        branch: string(p, "headBranch")?,
                        commit: string(p, "headSha")?,
                        url: string(p, "url")?,
                        updated_at: string(p, "updatedAt")?
                            .parse::<chrono::DateTime<Utc>>()
                            .map_err(|e| e.to_string())?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            EvidenceResult::Workflows(WorkflowsResult {
                repository: v.repository.clone(),
                complete: items.len() < (v.limit as usize),
                items,
            })
        }
        C::Ruff(v) => {
            let mut arguments = args(&["check", "--output-format", "json", "--"]);
            arguments.extend(v.paths.clone());
            let out = runner.run(arguments, &[0, 1]).await?;
            let diagnostics = array(&json_output(&out)?)?
                .iter()
                .map(|d| {
                    Ok(diagnostic(
                        string(d, "message")?,
                        optional(d, "filename"),
                        d["location"]["row"].as_u64(),
                        "error",
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            check("Ruff", &out, diagnostics)
        }
        C::Biome(v) => {
            let mut arguments = args(&["check", "--reporter=rdjson", "--"]);
            arguments.extend(v.paths.clone());
            let out = runner.run(arguments, &[0, 1]).await?;
            let value = json_output(&out)?;
            let diagnostics = array(&value["diagnostics"])?
                .iter()
                .map(|d| {
                    Ok(diagnostic(
                        string(d, "message")?,
                        d["location"]["path"].as_str().map(str::to_owned),
                        d["location"]["range"]["start"]["line"].as_u64(),
                        d["severity"].as_str().unwrap_or("error"),
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            check("Biome", &out, diagnostics)
        }
        C::Clippy(v) => {
            let mut arguments = args(&["clippy", "--message-format=json"]);
            if v.all_targets {
                arguments.push("--all-targets".into());
            }
            arguments.extend(args(&["--", "-D", "warnings"]));
            let out = runner.run(arguments, &[0, 101]).await?;
            let mut diagnostics = Vec::new();
            let mut finished = false;
            for line in out.stdout.lines() {
                let value: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
                if value["reason"] == "build-finished" {
                    finished = true;
                }
                if value["reason"] == "compiler-message" {
                    let d = &value["message"];
                    let span = d["spans"]
                        .as_array()
                        .and_then(|a| a.iter().find(|s| s["is_primary"] == true));
                    diagnostics.push(diagnostic(
                        string(d, "message")?,
                        span.and_then(|s| optional(s, "file_name")),
                        span.and_then(|s| s["line_start"].as_u64()),
                        d["level"].as_str().unwrap_or("error"),
                    ));
                }
            }
            if !finished {
                return Err("Clippy did not emit a complete build report".into());
            }
            check("Clippy", &out, diagnostics)
        }
        C::MiseTask(v) => {
            let mut arguments = Vec::new();
            if forge_config.is_some() {
                arguments.extend(args(&[
                    "--cd",
                    runner.cwd.to_str().ok_or("invalid task directory")?,
                ]));
            }
            arguments.extend(args(&["run", &v.task, "--"]));
            arguments.extend(v.args.clone());
            let mut env = runner.env.clone();
            env.insert("MISE_AUTO_INSTALL".into(), "false".into());
            env.insert("MISE_TASK_RUN_AUTO_INSTALL".into(), "false".into());
            if let Some(path) = forge_config {
                env.insert("MISE_CONFIG_FILE".into(), path.to_string_lossy().into());
                env.insert(
                    "MISE_TRUSTED_CONFIG_PATHS".into(),
                    path.to_string_lossy().into(),
                );
            }
            let out = process::run(
                &runner.executable,
                &arguments,
                &runner.cwd,
                &env,
                Duration::from_secs_f64(c.timeout()),
            )
            .await?;
            if out.code < 0 {
                return Err("mise task was terminated".into());
            }
            check(
                &v.task,
                &out,
                if out.code == 0 {
                    vec![]
                } else {
                    vec![diagnostic(
                        "Task failed; run mise in your terminal for full output".into(),
                        None,
                        None,
                        "error",
                    )]
                },
            )
        }
        C::Pytest(v) => {
            reject_overrides(&v.args, &["--junitxml", "--junit-xml"])?;
            let report = run_dir.join("pytest.xml");
            let mut arguments = v.args.clone();
            arguments.extend(args(&[
                "--junitxml",
                report.to_str().ok_or("invalid artifact path")?,
                "--",
            ]));
            arguments.extend(v.paths.clone());
            let output = runner.run(arguments, &[0, 1]).await?;
            test_report(&output, &report, c.id())?
        }
        C::Vitest(v) => {
            reject_overrides(&v.args, &["--reporter", "--outputFile", "--watch"])?;
            let report = run_dir.join("vitest.xml");
            let mut arguments = args(&[
                "run",
                "--reporter=junit",
                &format!("--outputFile={}", report.display()),
            ]);
            arguments.extend(v.args.clone());
            let output = runner.run(arguments, &[0, 1]).await?;
            test_report(&output, &report, c.id())?
        }
        C::Nextest(v) => {
            reject_overrides(&v.args, &["--config-file", "--profile", "-P"])?;
            let report = run_dir.join("nextest.xml");
            let config = run_dir.join("nextest.toml");
            let cargo = process::resolve("cargo", &runner.cwd)?;
            let located = process::run(
                &cargo,
                &args(&["locate-project", "--workspace", "--message-format", "plain"]),
                &runner.cwd,
                &runner.env,
                Duration::from_secs(15),
            )
            .await?;
            process::success(&located, &[0])?;
            let workspace = PathBuf::from(located.stdout.trim())
                .canonicalize()
                .map_err(|e| e.to_string())?;
            let workspace = workspace
                .parent()
                .ok_or("Cargo workspace has no directory")?;
            if !workspace.starts_with(root) {
                return Err("Nextest workspace is outside the selected project; select the Cargo workspace root with --manifest-dir".into());
            }
            let source_config = workspace.join(".config/nextest.toml");
            let mut value = if source_config.is_file() {
                toml::from_str::<toml::Value>(&files::read(&source_config)?)
                    .map_err(|e| e.to_string())?
            } else {
                toml::Value::Table(toml::map::Map::new())
            };
            let profile = toml::toml! { inherits = "default" [junit] path = "placeholder" };
            let table = value
                .as_table_mut()
                .ok_or("invalid nextest configuration")?;
            let profiles = table
                .entry("profile")
                .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
                .as_table_mut()
                .ok_or("invalid nextest profiles")?;
            profiles.insert("scryr".into(), toml::Value::Table(profile));
            profiles
                .get_mut("scryr")
                .ok_or("missing generated profile")?["junit"]["path"] =
                toml::Value::String(report.to_string_lossy().into());
            files::write(
                &config,
                toml::to_string(&value)
                    .map_err(|e| e.to_string())?
                    .as_bytes(),
            )?;
            let mut arguments = args(&[
                "nextest",
                "run",
                "--config-file",
                config.to_str().ok_or("invalid path")?,
                "--profile",
                "scryr",
            ]);
            arguments.extend(v.args.clone());
            let output = runner.run(arguments, &[0, 100]).await?;
            test_report(&output, &report, c.id())?
        }
        C::Junit(v) => EvidenceResult::Test(reports::junit(
            &files::reports(root, source_directory, &v.files)?,
            &v.suite,
        )?),
        C::Lcov(v) => EvidenceResult::Coverage(reports::coverage(
            &files::reports(root, source_directory, &v.files)?,
            "lcov",
            &v.suite,
        )?),
        C::Cobertura(v) => EvidenceResult::Coverage(reports::coverage(
            &files::reports(root, source_directory, &v.files)?,
            "cobertura",
            &v.suite,
        )?),
        C::SyftInventory(v) => {
            let mut arguments = args(&["scan", "dir:.", "-o", "syft-json"]);
            for exclude in &v.exclude {
                arguments.extend(args(&["--exclude", exclude]));
            }
            let out = runner.run(arguments, &[0]).await?;
            let inventory = dependencies::inventory(&out.stdout)?;
            let path = run_dir.join("inventory.syft.json");
            files::write(&path, out.stdout.as_bytes())?;
            collected.inventory = Some(InventoryArtifact {
                path,
                inventory: inventory.clone(),
                revision: declaration.revision.clone(),
                input: input.into(),
            });
            EvidenceResult::Inventory(inventory)
        }
        C::GrypeScan(_) => {
            let artifact = upstream.ok_or("waiting for a successful Syft inventory")?;
            let out = runner
                .run(
                    args(&[&format!("sbom:{}", artifact.path.display()), "-o", "json"]),
                    &[0],
                )
                .await?;
            EvidenceResult::Vulnerability(dependencies::vulnerabilities(
                &out.stdout,
                &artifact.inventory,
            )?)
        }
        C::GrantLicense(v) => {
            let artifact = upstream.ok_or("waiting for a successful Syft inventory")?;
            let out = runner
                .run(
                    args(&[
                        "list",
                        "--disable-file-search",
                        artifact.path.to_str().ok_or("invalid SBOM path")?,
                        "-o",
                        "json",
                    ]),
                    &[0],
                )
                .await?;
            EvidenceResult::License(dependencies::licenses(
                &out.stdout,
                &artifact.inventory,
                &v.policy,
            )?)
        }
        C::Openmetrics(v) => {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs_f64(c.timeout()))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| e.to_string())?;
            let mut request = client.get(&v.endpoint).header(
                "Accept",
                "application/openmetrics-text; version=1.0.0, text/plain; version=0.0.4;q=0.9",
            );
            if let Some(auth) = &v.auth {
                request = request.bearer_auth(
                    std::env::var(&auth.name).map_err(|_| "missing metrics auth variable")?,
                );
            }
            let mut response = request
                .send()
                .await
                .map_err(|e| format!("metrics connection failed: {}", e.without_url()))?;
            if !response.status().is_success() {
                return Err(format!("metrics endpoint returned {}", response.status()));
            }
            let content_type = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|h| h.to_str().ok())
                .unwrap_or("")
                .to_owned();
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|e| e.without_url().to_string())?
            {
                if bytes.len() + chunk.len() > 4_000_000 {
                    return Err("metrics response exceeds 4 MB".into());
                }
                bytes.extend(chunk);
            }
            let snapshot = openmetrics::parse(
                std::str::from_utf8(&bytes).map_err(|e| e.to_string())?,
                &content_type,
                v.max_series as usize,
                Utc::now(),
            )?;
            let result = openmetrics::measurements(v, &snapshot, previous);
            collected.snapshot = Some(snapshot);
            EvidenceResult::Metrics(result)
        }
        C::DockerStats(v) => {
            EvidenceResult::Metrics(docker(&runner, &v.context, &v.containers).await?)
        }
        C::Hyperfine(v) => {
            let command = std::iter::once(
                process::resolve(&v.command.executable, &runner.cwd)?
                    .to_string_lossy()
                    .into_owned(),
            )
            .chain(v.command.args.clone())
            .map(|s| format!("'{}'", s.replace('\'', "'\\''")))
            .collect::<Vec<_>>()
            .join(" ");
            let report = run_dir.join("benchmark.json");
            runner
                .run(
                    args(&[
                        "--shell=none",
                        "--warmup",
                        &v.warmup.to_string(),
                        "--runs",
                        &v.runs.to_string(),
                        "--export-json",
                        report.to_str().ok_or("invalid benchmark report path")?,
                        "--",
                        &command,
                    ]),
                    &[0],
                )
                .await?;
            let value: Value =
                serde_json::from_str(&files::read(&report)?).map_err(|e| e.to_string())?;
            let results = array(&value["results"])?;
            if results.len() != 1 {
                return Err("expected exactly one benchmark result".into());
            }
            let result = &results[0];
            let number = |key: &str| {
                result[key]
                    .as_f64()
                    .filter(|n| n.is_finite() && *n >= 0.0)
                    .ok_or_else(|| format!("invalid benchmark {key}"))
            };
            let times = array(&result["times"])?;
            if times.len() != v.runs as usize
                || times
                    .iter()
                    .any(|t| t.as_f64().is_none_or(|n| !n.is_finite() || n < 0.0))
            {
                return Err("incomplete benchmark samples".into());
            }
            EvidenceResult::Benchmark(BenchmarkResult {
                name: c.id().into(),
                mean_seconds: number("mean")?,
                stddev_seconds: number("stddev")?,
                median_seconds: number("median")?,
                runs: v.runs,
                command,
                baseline_mean_seconds: None,
                machine: machine_identity(&runner).await,
            })
        }
    };
    // Redact values before the result can be persisted or sent anywhere.
    let mut encoded = serde_json::to_value(&collected.result).map_err(|e| e.to_string())?;
    redact_value(&mut encoded, &runner.env);
    collected.result = serde_json::from_value(encoded).map_err(|e| e.to_string())?;
    collected.result.validate()?;
    Ok(collected)
}

async fn machine_identity(runner: &Runner<'_>) -> String {
    let hostname = match process::resolve("hostname", &runner.cwd) {
        Ok(path) => process::run(
            &path,
            &[],
            &runner.cwd,
            &BTreeMap::new(),
            Duration::from_secs(5),
        )
        .await
        .ok()
        .filter(|o| o.code == 0)
        .map(|o| o.stdout.trim().to_owned()),
        Err(_) => None,
    };
    let identity = hostname
        .as_ref()
        .and_then(|h| crystal_core::collectors::fingerprint(h).ok());
    format!(
        "{}:{}",
        machine(),
        identity.as_deref().map_or("unknown", |id| &id[..16])
    )
}

pub(super) fn machine() -> String {
    format!(
        "{}-{}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
    )
}
async fn git(runner: &Runner<'_>) -> Result<GitResult, String> {
    let out = runner
        .run(
            args(&[
                "status",
                "--porcelain=v2",
                "--branch",
                "--untracked-files=normal",
            ]),
            &[0],
        )
        .await?;
    let mut result = GitResult {
        branch: None,
        commit: None,
        dirty: false,
        changed_files: 0,
        ahead: 0,
        behind: 0,
        remote_url: None,
    };
    for line in out.stdout.lines() {
        if let Some(v) = line
            .strip_prefix("# branch.oid ")
            .filter(|s| *s != "(initial)")
        {
            result.commit = Some(v.into());
        } else if let Some(v) = line
            .strip_prefix("# branch.head ")
            .filter(|s| *s != "(detached)")
        {
            result.branch = Some(v.into());
        } else if let Some(v) = line.strip_prefix("# branch.ab ") {
            let parts: Vec<_> = v.split_whitespace().collect();
            if parts.len() == 2 {
                result.ahead = parts[0]
                    .trim_start_matches('+')
                    .parse()
                    .map_err(|_| "invalid git ahead count")?;
                result.behind = parts[1]
                    .trim_start_matches('-')
                    .parse()
                    .map_err(|_| "invalid git behind count")?;
            }
        } else if !line.starts_with('#') && !line.is_empty() {
            result.changed_files += 1;
        }
    }
    result.dirty = result.changed_files > 0;
    let out = runner
        .run(args(&["remote", "get-url", "origin"]), &[0, 2])
        .await?;
    if out.code == 0 {
        let raw = out.stdout.trim();
        result.remote_url = Some(url::Url::parse(raw).map_or_else(
            |_| raw.into(),
            |mut url| {
                let _ = url.set_username("");
                let _ = url.set_password(None);
                url.set_query(None);
                url.to_string()
            },
        ));
    }
    Ok(result)
}
async fn docker(
    runner: &Runner<'_>,
    context: &str,
    containers: &[String],
) -> Result<MetricsResult, String> {
    let inspect = runner
        .run(
            args(&[
                "context",
                "inspect",
                context,
                "--format",
                "{{json .Endpoints.docker.Host}}",
            ]),
            &[0],
        )
        .await?;
    let host: String = serde_json::from_str(inspect.stdout.trim()).map_err(|e| e.to_string())?;
    if !host.starts_with("unix://") && !host.starts_with("npipe://") {
        return Err("DockerStatsCollector requires a local Docker context".into());
    }
    let mut arguments = args(&[
        "--context",
        context,
        "stats",
        "--no-stream",
        "--format",
        "json",
        "--",
    ]);
    arguments.extend_from_slice(containers);
    let out = runner.run(arguments, &[0]).await?;
    let mut samples = Vec::new();
    for line in out.stdout.lines().filter(|s| !s.is_empty()) {
        let value: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
        for (key, name) in [
            ("CPUPerc", "container_cpu_percent"),
            ("MemPerc", "container_memory_percent"),
        ] {
            let number = string(&value, key)?
                .trim_end_matches('%')
                .parse::<f64>()
                .map_err(|_| "invalid Docker percentage")?;
            if !number.is_finite() || number < 0.0 {
                return Err("invalid Docker sample".into());
            }
            samples.push(MetricSample {
                name: name.into(),
                title: None,
                labels: vec![MetricLabel {
                    name: "container".into(),
                    value: string(&value, "Name")?,
                }],
                value: number,
                unit: Some("percent".into()),
                metric_type: "gauge".into(),
            });
        }
    }
    if samples.len() > 200 {
        return Err("too many Docker samples".into());
    }
    Ok(MetricsResult {
        samples,
        scraped_at: Utc::now(),
        complete: true,
    })
}

/// Checkout context is captured locally; GitHub data carries its own remote refs.
pub(super) async fn provenance(
    root: &Path,
    source_directory: &Path,
    config: &CollectorConfig,
) -> Option<GitResult> {
    if matches!(
        config,
        CollectorConfig::GithubActions(_)
            | CollectorConfig::GithubPullRequests(_)
            | CollectorConfig::Openmetrics(_)
            | CollectorConfig::DockerStats(_)
    ) {
        return None;
    }
    let cwd = process::directory(root, source_directory, config.directory()).ok()?;
    let runner = Runner {
        executable: process::resolve("git", &cwd).ok()?,
        cwd,
        env: BTreeMap::new(),
        config,
    };
    git(&runner).await.ok()
}

fn redact_value(value: &mut Value, env: &BTreeMap<String, String>) {
    match value {
        Value::String(text) => *text = process::redact(text, env),
        Value::Array(values) => values.iter_mut().for_each(|v| redact_value(v, env)),
        Value::Object(values) => values.values_mut().for_each(|v| redact_value(v, env)),
        _ => {}
    }
}

fn test_report(
    output: &process::Output,
    report: &Path,
    suite: &str,
) -> Result<EvidenceResult, String> {
    let result = reports::junit(&[files::read(report)?], suite)?;
    if output.code != 0 && result.failing + result.errors == 0 {
        return Err("test process failed but its report contains no failures; refusing to show a passing result".into());
    }
    Ok(EvidenceResult::Test(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn nested_entrypoint_uses_its_tool_cwd_and_existing_reports() -> Result<(), String> {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let root = tmp.path().canonicalize().map_err(|e| e.to_string())?;
        let source = root.join("services/api");
        files::write(&source.join("index.scry"), b"# nested fixture")?;
        files::write(&source.join("tests/test_nested.py"), b"# nested input")?;
        files::write(&root.join("tests/test_root.py"), b"# unrelated root input")?;
        let tool = source.join(".venv/bin/pytest");
        files::write(&tool, br#"#!/bin/sh
if [ "$1" = "--version" ]; then echo 'pytest 8.4.0'; exit 0; fi
[ -f tests/test_nested.py ] && [ ! -f tests/test_root.py ] || exit 2
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--junitxml" ]; then shift; printf '<testsuite><testcase name="nested-execution"/></testsuite>' > "$1"; fi
  shift
done
"#)?;
        std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        let root_tool = root.join(".venv/bin/pytest");
        files::write(&root_tool, b"#!/bin/sh\nexit 42\n")?;
        std::fs::set_permissions(&root_tool, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        let declarations = crystal_core::collectors::declarations(
            &serde_json::json!({"manifests":[{"manifestId":"api","tests":[{"kind":"pytest","paths":["tests"]},{"kind":"junit","files":["results/existing.xml"]}]}]}),
        )?;
        let runner = &declarations[0];
        assert_eq!(
            doctor(&runner.config, &root, &source).await?,
            Some("pytest 8.4.0".into())
        );
        let run = root.join(".scryr/runs/nested");
        files::private_dir(&run)?;
        let collected = execute(runner, &root, &source, &run, None, None, "input", None).await?;
        let EvidenceResult::Test(result) = collected.result else {
            return Err("expected generated report".into());
        };
        assert_eq!(result.cases[0].name, "nested-execution");
        files::write(
            &root.join("results/existing.xml"),
            b"<testsuite><testcase name=\"root-passing\"/></testsuite>",
        )?;
        files::write(
            &source.join("results/existing.xml"),
            b"<testsuite><testcase name=\"nested-failing\"><failure/></testcase></testsuite>",
        )?;
        let imported = execute(
            &declarations[1],
            &root,
            &source,
            &run,
            None,
            None,
            "input",
            None,
        )
        .await?;
        let EvidenceResult::Test(result) = imported.result else {
            return Err("expected imported report".into());
        };
        assert_eq!(result.cases[0].name, "nested-failing");
        assert_eq!(result.failing, 1);
        assert!(imported.source_updated_at.is_some());
        Ok(())
    }
    #[tokio::test]
    async fn pytest_exit_one_is_valid_only_with_a_fresh_failure_report() -> Result<(), String> {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let root = tmp.path().canonicalize().map_err(|e| e.to_string())?;
        let tool = root.join(".venv/bin/pytest");
        files::write(&tool,br#"#!/bin/sh
if [ "$1" = "--version" ]; then echo 'pytest 8.4.0'; exit 0; fi
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--junitxml" ]; then shift; printf '<testsuite><testcase name="expected"><failure message="failed assertion"/></testcase></testsuite>' > "$1"; fi
  shift
done
exit 1
"#)?;
        std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        let declaration = crystal_core::collectors::declarations(
            &serde_json::json!({"manifests":[{"manifestId":"api","tests":[{"kind":"pytest"}]}]}),
        )?
        .remove(0);
        let run = root.join(".scryr/runs/one");
        files::private_dir(&run)?;
        let result = execute(&declaration, &root, &root, &run, None, None, "input", None).await?;
        let EvidenceResult::Test(result) = result.result else {
            return Err("expected test report".into());
        };
        assert_eq!(result.failing, 1);
        files::write(&tool,b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'pytest 8.4.0'; exit 0; fi\nexit 1\n")?;
        std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        let fresh = root.join(".scryr/runs/two");
        files::private_dir(&fresh)?;
        assert!(
            execute(
                &declaration,
                &root,
                &root,
                &fresh,
                None,
                None,
                "input",
                None
            )
            .await
            .is_err()
        );
        Ok(())
    }
    #[tokio::test]
    async fn metrics_http_sampling_uses_counter_deltas() -> Result<(), String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| e.to_string())?;
        let address = listener.local_addr().map_err(|e| e.to_string())?;
        let server = tokio::spawn(async move {
            for counter in [10, 14] {
                let (mut stream, _) = listener.accept().await.map_err(|e| e.to_string())?;
                let mut request = [0u8; 4096];
                stream.read(&mut request).await.map_err(|e| e.to_string())?;
                let body = format!("# TYPE requests_total counter\nrequests_total {counter}\n");
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .map_err(|e| e.to_string())?;
            }
            Ok::<(), String>(())
        });
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let root = tmp.path().canonicalize().map_err(|e| e.to_string())?;
        let declaration=crystal_core::collectors::declarations(&serde_json::json!({"manifests":[{"manifestId":"api","metrics":[{"kind":"openmetrics","endpoint":format!("http://{address}/metrics"),"series":[{"kind":"counter_rate","metric":"requests_total"}]}]}]}))?.remove(0);
        let first = execute(&declaration, &root, &root, &root, None, None, "input", None).await?;
        let EvidenceResult::Metrics(result) = first.result else {
            return Err("expected metrics".into());
        };
        assert!(!result.complete && result.samples.is_empty());
        let mut snapshot = first.snapshot.ok_or("missing snapshot")?;
        snapshot.at = Utc::now() - chrono::Duration::seconds(1);
        let second = execute(
            &declaration,
            &root,
            &root,
            &root,
            None,
            Some(&snapshot),
            "input",
            None,
        )
        .await?;
        let EvidenceResult::Metrics(result) = second.result else {
            return Err("expected metrics".into());
        };
        assert!(result.complete);
        assert!((3.0..=4.1).contains(&result.samples[0].value));
        server.await.map_err(|e| e.to_string())??;
        Ok(())
    }
    #[test]
    fn inconsistent_runner_and_report_cannot_turn_green() -> Result<(), String> {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let report = tmp.path().join("tests.xml");
        files::write(
            &report,
            b"<testsuite><testcase name=\"flaky\"/></testsuite>",
        )?;
        let output = process::Output {
            code: 100,
            stdout: String::new(),
            stderr: String::new(),
            duration: 1.0,
        };
        assert!(test_report(&output, &report, "rust").is_err());
        let mut value = serde_json::json!({"message":"contains a\"secret-value"});
        redact_value(
            &mut value,
            &BTreeMap::from([("TOKEN".into(), "a\"secret-value".into())]),
        );
        assert_eq!(value["message"], "contains [redacted]");
        Ok(())
    }
}
