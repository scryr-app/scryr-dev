//! One local owner reconciles typed plans and supervises bounded collection jobs.
#![allow(clippy::missing_docs_in_private_items)]
use super::{files, integrations, openmetrics};
use chrono::Utc;
use crystal_core::{
    collectors::{CollectorConfig, CollectorDeclaration, declarations, fingerprint},
    evidence::{CollectorState, CollectorStatus, EvidenceObservation, EvidenceResult},
    manifest::ManifestRequestContext,
    persistence::{self, DatabasePool},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::watch,
    task::{AbortHandle, JoinSet},
};

#[derive(Clone)]
pub(crate) struct Plan {
    pub declarations: Vec<CollectorDeclaration>,
    pub revision: String,
    pub forge_configs: BTreeMap<String, String>,
    pub source_directory: PathBuf,
    source_scope: String,
}
impl Plan {
    pub(crate) fn parse(json: &str, root: &Path, entrypoint: &Path) -> Result<Self, String> {
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let entrypoint = entrypoint.canonicalize().map_err(|e| e.to_string())?;
        if !entrypoint.starts_with(&root) || !entrypoint.is_file() {
            return Err("collector entrypoint must be a file within the registered project".into());
        }
        let source_directory = entrypoint
            .parent()
            .ok_or("collector entrypoint has no parent")?
            .to_owned();
        let relative = source_directory
            .strip_prefix(&root)
            .map_err(|e| e.to_string())?;
        let source_scope = if relative.as_os_str().is_empty() {
            ".".into()
        } else {
            relative.to_string_lossy().into_owned()
        };
        let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let declarations = declarations(&value)?;
        let mut forge_configs = BTreeMap::new();
        for d in &declarations {
            if let CollectorConfig::MiseTask(c) = &d.config
                && let Some(forge) = &c.forge
            {
                forge_configs.insert(
                    key(d),
                    crystal_core::generation::render_mise_toml(json, Some(forge))?,
                );
            }
        }
        let revision = fingerprint(&(&value, &source_scope))?;
        Ok(Self {
            declarations,
            revision,
            forge_configs,
            source_directory,
            source_scope,
        })
    }
}
pub(super) fn key(d: &CollectorDeclaration) -> String {
    format!(
        "{}\t{}\t{}",
        d.manifest_id,
        d.section.as_str(),
        d.config.id()
    )
}
fn upstream_key(plan: &Plan, d: &CollectorDeclaration) -> Option<String> {
    if !matches!(
        d.config,
        CollectorConfig::GrypeScan(_) | CollectorConfig::GrantLicense(_)
    ) {
        return None;
    }
    plan.declarations
        .iter()
        .find(|candidate| {
            matches!(candidate.config, CollectorConfig::SyftInventory(_))
                && d.config.sbom().map_or_else(
                    || candidate.manifest_id == d.manifest_id,
                    |r| {
                        candidate.manifest_id == *r.manifest_id.as_ref().unwrap_or(&d.manifest_id)
                            && candidate.config.id() == r.collector_id
                    },
                )
        })
        .map(key)
}
fn effective(plan: &Plan, d: &CollectorDeclaration) -> Result<String, String> {
    let upstream = upstream_key(plan, d)
        .and_then(|key| plan.declarations.iter().find(|c| self::key(c) == key));
    fingerprint(&(
        d.revision.clone(),
        &plan.source_scope,
        upstream.map(|d| (&d.manifest_id, d.config.id(), &d.revision)),
        plan.forge_configs.get(&key(d)),
    ))
}
const fn heavy(c: &CollectorConfig) -> bool {
    matches!(
        c,
        CollectorConfig::Pytest(_)
            | CollectorConfig::Vitest(_)
            | CollectorConfig::Nextest(_)
            | CollectorConfig::SyftInventory(_)
            | CollectorConfig::GrypeScan(_)
            | CollectorConfig::GrantLicense(_)
            | CollectorConfig::Hyperfine(_)
            | CollectorConfig::Clippy(_)
            | CollectorConfig::MiseTask(_)
    )
}
fn context() -> ManifestRequestContext {
    ManifestRequestContext {
        clerk_user_id: "local-dev-user".into(),
        clerk_org_id: "local-dev-org".into(),
        clerk_org_slug: None,
        clerk_org_role: Some("admin".into()),
        clerk_org_permissions: vec![],
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct Selection {
    pub manifest: Option<String>,
    pub section: Option<String>,
    pub collector: Option<String>,
}
impl Selection {
    fn matches(&self, d: &CollectorDeclaration) -> bool {
        self.manifest.as_ref().is_none_or(|v| *v == d.manifest_id)
            && self
                .section
                .as_ref()
                .is_none_or(|v| v == d.section.as_str())
            && self.collector.as_ref().is_none_or(|v| v == d.config.id())
    }
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub(super) enum Request {
    Run { selection: Selection },
    Status,
    Pause { paused: bool },
}
#[derive(Serialize, Deserialize)]
pub(super) struct Response {
    pub message: String,
    pub statuses: Vec<CollectorStatus>,
    pub error: bool,
}

pub(super) struct Location {
    pub root: PathBuf,
    pub state: PathBuf,
    pub database: PathBuf,
    pub workspace: String,
    pub socket: PathBuf,
}
impl Location {
    fn managed_directory(&self) -> Option<&Path> {
        self.state
            .parent()
            .filter(|p| p.file_name().is_some_and(|n| n == "collection"))
            .and_then(Path::parent)
            .or(Some(&self.state))
    }
    pub(super) fn new(root: &Path, scryr_dir: Option<&Path>) -> Result<Self, String> {
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let database=persistence::sqlite_path_from_env().ok_or("local collection requires file-backed SQLite; remote storage is not a laptop execution target")?;
        let database = std::path::absolute(database).map_err(|e| e.to_string())?;
        let database = if database.exists() {
            database.canonicalize().map_err(|e| e.to_string())?
        } else {
            let parent = database.parent().ok_or("database path has no parent")?;
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            parent.canonicalize().map_err(|e| e.to_string())?.join(
                database
                    .file_name()
                    .ok_or("database path has no filename")?,
            )
        };
        let workspace = fingerprint(&(root.to_string_lossy(), database.to_string_lossy()))?;
        let state = scryr_dir
            .map_or_else(|| root.join(".scryr"), Path::to_owned)
            .join("collection")
            .join(&workspace[..16]);
        let state = std::path::absolute(state).map_err(|e| e.to_string())?;
        let socket = PathBuf::from("/tmp").join(format!("scryr-{}.sock", &workspace[..32]));
        Ok(Self {
            root,
            state,
            database,
            workspace,
            socket,
        })
    }
}
struct Lease {
    _file: std::fs::File,
    socket: PathBuf,
}
impl Lease {
    fn acquire(location: &Location) -> Result<Self, String> {
        files::private_dir(&location.state)?;
        let lock_dir = location
            .root
            .join(".scryr/collection")
            .join(&location.workspace[..16]);
        files::private_dir(&lock_dir)?;
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_dir.join("owner.lock"))
            .map_err(|e| e.to_string())?;
        lock.try_lock().map_err(
            |_| "this workspace already has a collector owner; use scryr collect run/status",
        )?;
        // Only the lock holder may remove a stale socket from a previous process.
        if location.socket.exists() {
            std::fs::remove_file(&location.socket).map_err(|e| e.to_string())?;
        }
        Ok(Self {
            _file: lock,
            socket: location.socket.clone(),
        })
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket);
    }
}

pub(super) async fn contact(
    location: &Location,
    request: &Request,
) -> Result<Option<Response>, String> {
    let stream = match tokio::net::UnixStream::connect(&location.socket).await {
        Ok(s) => s,
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            ) =>
        {
            return Ok(None);
        }
        Err(e) => return Err(e.to_string()),
    };
    let mut stream = BufReader::new(stream);
    let mut bytes = serde_json::to_vec(request).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    stream
        .get_mut()
        .write_all(&bytes)
        .await
        .map_err(|e| e.to_string())?;
    let mut response = String::new();
    tokio::time::timeout(Duration::from_secs(5), stream.read_line(&mut response))
        .await
        .map_err(|_| "collector owner did not respond")?
        .map_err(|e| e.to_string())?;
    if response.len() > 2_000_000 {
        return Err("collector owner response too large".into());
    }
    serde_json::from_str(&response)
        .map(Some)
        .map_err(|e| e.to_string())
}
#[derive(Clone)]
struct Entry {
    declaration: CollectorDeclaration,
    effective: String,
    status: CollectorStatus,
    input: String,
    changed: Instant,
    due: Option<Instant>,
    failures: u32,
    attempt: u32,
    previous_result: Option<String>,
    last_evidence: Option<Instant>,
}
struct Running {
    abort: AbortHandle,
    effective: String,
    heavy: bool,
    benchmark: bool,
    directory: PathBuf,
}
struct Completion {
    key: String,
    effective: String,
    observation: EvidenceObservation,
    result: Result<integrations::Collected, String>,
}

pub(crate) async fn supervise(
    root: PathBuf,
    scryr_dir: Option<PathBuf>,
    plans: watch::Receiver<Option<Plan>>,
    paused: bool,
) -> Result<(), String> {
    let location = Location::new(&root, scryr_dir.as_deref())?;
    let mut owner = Owner::new(location, paused).await?;
    owner.run(plans, None).await
}
pub(super) async fn once(
    location: Location,
    plan: Plan,
    selection: Selection,
) -> Result<(), String> {
    let mut owner = Owner::new(location, false).await?;
    owner.replace(plan, false).await?;
    owner.queue(&selection)?;
    let (sender, receiver) = watch::channel(None);
    let result = owner.run(receiver, Some(())).await;
    drop(sender);
    result
}
struct Owner {
    location: Location,
    _lease: Lease,
    listener: tokio::net::UnixListener,
    pool: DatabasePool,
    entries: BTreeMap<String, Entry>,
    plan: Option<Plan>,
    pending: BTreeSet<String>,
    running: BTreeMap<String, Running>,
    tasks: JoinSet<Completion>,
    inventories: BTreeMap<String, integrations::InventoryArtifact>,
    snapshots: BTreeMap<String, openmetrics::Snapshot>,
    paused: bool,
    failures: bool,
    network_due: BTreeMap<String, Instant>,
}
impl Owner {
    async fn new(location: Location, paused: bool) -> Result<Self, String> {
        let lease = Lease::acquire(&location)?;
        let listener =
            tokio::net::UnixListener::bind(&location.socket).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&location.socket, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| e.to_string())?;
        }
        let pool = persistence::connect_sqlite_path(&location.database)
            .await
            .map_err(|e| e.to_string())?;
        persistence::ensure_table(&pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Self {
            location,
            _lease: lease,
            listener,
            pool,
            entries: BTreeMap::new(),
            plan: None,
            pending: BTreeSet::new(),
            running: BTreeMap::new(),
            tasks: JoinSet::new(),
            inventories: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            paused,
            failures: false,
            network_due: BTreeMap::new(),
        })
    }
    async fn replace(&mut self, plan: Plan, startup: bool) -> Result<(), String> {
        // Prepare every fallible input before changing the last valid registry.
        let mut prepared = BTreeMap::new();
        for d in &plan.declarations {
            let k = key(d);
            let revision = effective(&plan, d)?;
            let input = if let Some(e) = self.entries.get(&k).filter(|e| e.effective == revision) {
                e.input.clone()
            } else {
                files::input_ignoring(
                    &self.location.root,
                    &plan.source_directory,
                    &d.config,
                    self.location.managed_directory(),
                )?
            };
            prepared.insert(k, (revision, input));
        }
        let mut previous = std::mem::take(&mut self.entries);
        for d in &plan.declarations {
            let key = key(d);
            let (effective, input) = prepared.remove(&key).ok_or("missing prepared collector")?;
            if let Some(mut old) = previous.remove(&key).filter(|e| e.effective == effective) {
                old.declaration = d.clone();
                self.entries.insert(key, old);
                continue;
            }
            if let Some(run) = self.running.remove(&key) {
                run.abort.abort();
            }
            self.inventories.remove(&key);
            self.snapshots.remove(&key);
            self.pending.remove(&key);
            let state = if self.paused {
                CollectorState::Disabled
            } else {
                CollectorState::Waiting
            };
            let status = CollectorStatus {
                manifest_id: d.manifest_id.clone(),
                section: d.section,
                collector_id: d.config.id().into(),
                workspace_id: self.location.workspace.clone(),
                integration: d.config.kind().into(),
                collector_revision: d.revision.clone(),
                state,
                message: None,
                updated_at: Utc::now(),
                freshness_seconds: Duration::from_secs_f64(d.config.common().freshness).as_secs(),
                input_fingerprint: Some(input.clone()),
                next_run_at: None,
                last_attempt_at: None,
            };
            if startup && d.config.schedule().startup {
                self.pending.insert(key.clone());
            }
            let due = d
                .config
                .schedule()
                .every
                .map(|s| Instant::now() + Duration::from_secs_f64(s));
            self.entries.insert(
                key,
                Entry {
                    declaration: d.clone(),
                    effective,
                    status,
                    input,
                    changed: Instant::now(),
                    due,
                    failures: 0,
                    attempt: 0,
                    previous_result: None,
                    last_evidence: None,
                },
            );
        }
        for (key, mut old) in previous {
            if let Some(run) = self.running.remove(&key) {
                run.abort.abort();
            }
            self.pending.remove(&key);
            self.inventories.remove(&key);
            self.snapshots.remove(&key);
            old.status.state = CollectorState::Disabled;
            old.status.message = Some("Collector removed from the current plan".into());
            old.status.updated_at = Utc::now();
            persistence::record_collector_status(&self.pool, &context(), old.status).await?;
        }
        self.plan = Some(plan);
        self.persist_statuses().await
    }
    fn queue(&mut self, selection: &Selection) -> Result<usize, String> {
        if self.paused {
            return Err("collection is paused; run scryr collect resume".into());
        }
        let matches: Vec<_> = self
            .entries
            .iter()
            .filter(|(_, e)| selection.matches(&e.declaration))
            .map(|(k, _)| k.clone())
            .collect();
        if matches.is_empty() {
            return Err("no declared collector matches these selectors".into());
        }
        for key in &matches {
            let entry = &self.entries[key];
            if !entry.declaration.config.schedule().manual {
                return Err(format!(
                    "{} does not enable manual collection",
                    entry.declaration.config.id()
                ));
            }
        }
        for key in &matches {
            self.pending.insert(key.clone());
            if let Some(upstream) = self
                .plan
                .as_ref()
                .and_then(|p| upstream_key(p, &self.entries[key].declaration))
                && !self.inventories.contains_key(&upstream)
            {
                self.pending.insert(upstream);
            }
        }
        Ok(matches.len())
    }
    async fn persist_statuses(&self) -> Result<(), String> {
        let statuses: Vec<_> = self.entries.values().map(|e| e.status.clone()).collect();
        files::write(
            &self.location.state.join("status.json"),
            &serde_json::to_vec(&statuses).map_err(|e| e.to_string())?,
        )?;
        for status in statuses {
            if let Err(e) =
                persistence::record_collector_status(&self.pool, &context(), status).await
            {
                eprintln!("Collector status delivery: {e}");
            }
        }
        Ok(())
    }
    async fn run(
        &mut self,
        mut plans: watch::Receiver<Option<Plan>>,
        once: Option<()>,
    ) -> Result<(), String> {
        let mut tick = tokio::time::interval(Duration::from_millis(500));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut scan = Instant::now();
        let mut delivery = Instant::now();
        loop {
            tokio::select! {
                changed=plans.changed(),if once.is_none()=>{if changed.is_err(){break;}let next=plans.borrow_and_update().clone();if let Some(plan)=next&& let Err(e)=self.replace(plan,true).await {eprintln!("Collection plan reload failed: {e}");}},
                connection=self.listener.accept()=>{if let Ok((stream,_))=connection {self.control(stream).await;}},
                result=self.tasks.join_next(),if !self.tasks.is_empty()=>{match result {
                    Some(Ok(completion)) => self.complete(completion).await?,
                    Some(Err(error)) => { if let Some(key) = self.running.iter().find(|(_,r)| r.abort.id() == error.id()).map(|(k,_)|k.clone()) { self.running.remove(&key); if let Some(entry)=self.entries.get_mut(&key) {entry.status.state=CollectorState::Error;entry.status.message=Some(format!("collector task stopped: {error}"));entry.status.updated_at=Utc::now();self.failures=true;self.persist_statuses().await?;} } },
                    None=>{},
                }},
                _=tick.tick()=>{
                    if scan.elapsed()>=Duration::from_secs(2) {self.inputs(once.is_none()).await?;scan=Instant::now();}
                    if delivery.elapsed()>=Duration::from_secs(5) {self.drain().await?;delivery=Instant::now();}
                    self.launch().await?;
                    if once.is_some() && self.pending.is_empty() && self.running.is_empty() {self.drain().await?;return if self.failures {Err("one or more collectors failed; run scryr collect status for details".into())}else{Ok(())};}
                },
                _=tokio::signal::ctrl_c()=>break,
            }
        }
        self.tasks.abort_all();
        while self.tasks.join_next().await.is_some() {}
        for entry in self.entries.values_mut() {
            if entry.status.state == CollectorState::Running {
                entry.status.state = CollectorState::Cancelled;
                entry.status.message = Some("Local collector owner stopped".into());
                entry.status.updated_at = Utc::now();
            }
        }
        self.persist_statuses().await
    }
    async fn control(&mut self, stream: tokio::net::UnixStream) {
        let mut stream = BufReader::new(stream);
        let mut bytes = Vec::new();
        let read = tokio::time::timeout(
            Duration::from_secs(2),
            tokio::io::AsyncReadExt::take(&mut stream, 16_384).read_until(b'\n', &mut bytes),
        )
        .await;
        let result = match read {
            Ok(Ok(_)) => serde_json::from_slice::<Request>(&bytes).map_err(|e| e.to_string()),
            _ => Err("invalid local request".into()),
        };
        let result = match result {
            Ok(Request::Run { selection }) => self
                .queue(&selection)
                .map(|n| format!("Queued {n} collector(s)")),
            Ok(Request::Status) => Ok("Local collector owner is running".into()),
            Ok(Request::Pause { paused }) => {
                self.paused = paused;
                if paused {
                    self.pending.clear();
                    self.tasks.abort_all();
                    self.running.clear();
                }
                for e in self.entries.values_mut() {
                    e.status.state = if paused {
                        CollectorState::Disabled
                    } else {
                        CollectorState::Waiting
                    };
                    e.status.updated_at = Utc::now();
                }
                let _ = self.persist_statuses().await;
                Ok(if paused {
                    "Collection paused"
                } else {
                    "Collection resumed"
                }
                .into())
            }
            Err(e) => Err(e),
        };
        let response = Response {
            error: result.is_err(),
            message: result.unwrap_or_else(|e| e),
            statuses: self.entries.values().map(|e| e.status.clone()).collect(),
        };
        if let Ok(mut bytes) = serde_json::to_vec(&response) {
            bytes.push(b'\n');
            let _ = stream.get_mut().write_all(&bytes).await;
        }
    }
    async fn inputs(&mut self, scheduled: bool) -> Result<(), String> {
        let Some(plan) = &self.plan else {
            return Ok(());
        };
        let root = self.location.root.clone();
        let source_directory = plan.source_directory.clone();
        let configs: Vec<_> = self
            .entries
            .iter()
            .map(|(k, e)| (k.clone(), e.declaration.config.clone()))
            .collect();
        let excluded = self.location.managed_directory().map(Path::to_owned);
        let values = tokio::task::spawn_blocking(move || {
            let mut cache = BTreeMap::new();
            configs
                .into_iter()
                .map(|(k, c)| {
                    let result = files::input_scope(&c).and_then(|scope| {
                        cache
                            .entry(scope)
                            .or_insert_with(|| {
                                files::input_ignoring(
                                    &root,
                                    &source_directory,
                                    &c,
                                    excluded.as_deref(),
                                )
                            })
                            .clone()
                    });
                    (k, result)
                })
                .collect::<Vec<_>>()
        })
        .await
        .map_err(|e| e.to_string())?;
        for (key, input) in values {
            let Some(e) = self.entries.get_mut(&key) else {
                continue;
            };
            match input {
                Ok(input) => {
                    if input != e.input {
                        e.input.clone_from(&input);
                        e.changed = Instant::now();
                        e.status.input_fingerprint = Some(input);
                        e.status.updated_at = Utc::now();
                        if scheduled && !e.declaration.config.schedule().watch.is_empty() {
                            self.pending.insert(key.clone());
                        }
                    }
                }
                Err(error) => {
                    e.status.state = CollectorState::Error;
                    e.status.message = Some(error);
                    e.status.updated_at = Utc::now();
                }
            }
            if scheduled && e.due.is_some_and(|at| at <= Instant::now()) {
                if !self.running.contains_key(&key) {
                    self.pending.insert(key);
                }
                e.due = e
                    .declaration
                    .config
                    .schedule()
                    .every
                    .map(|n| Instant::now() + Duration::from_secs_f64(n));
            }
            e.status.next_run_at = e.due.map(|at| {
                Utc::now()
                    + chrono::Duration::from_std(at.saturating_duration_since(Instant::now()))
                        .unwrap_or_default()
            });
        }
        self.persist_statuses().await
    }
    #[allow(clippy::too_many_lines)] // Admission, dependency ordering and launch are one scheduler transition.
    async fn launch(&mut self) -> Result<(), String> {
        if self.paused {
            return Ok(());
        }
        let Some(plan) = self.plan.clone() else {
            return Ok(());
        };
        let mut changed = false;
        let benchmark_waiting = self.pending.iter().any(|k| {
            self.entries
                .get(k)
                .is_some_and(|e| matches!(e.declaration.config, CollectorConfig::Hyperfine(_)))
        });
        for key in self.pending.clone() {
            if self.running.len() >= 2 {
                break;
            }
            let Some(e) = self.entries.get(&key) else {
                self.pending.remove(&key);
                continue;
            };
            if self.running.contains_key(&key)
                || e.changed.elapsed()
                    < Duration::from_secs_f64(e.declaration.config.schedule().debounce)
            {
                continue;
            }
            let benchmark = matches!(e.declaration.config, CollectorConfig::Hyperfine(_));
            let is_heavy = heavy(&e.declaration.config);
            if benchmark_waiting && !benchmark {
                continue;
            }
            if self
                .running
                .values()
                .any(|r| r.benchmark || (is_heavy && r.heavy))
                || (benchmark && !self.running.is_empty())
            {
                continue;
            }
            let repository = match &e.declaration.config {
                CollectorConfig::GithubActions(c) => Some(c.repository.clone()),
                CollectorConfig::GithubPullRequests(c) => Some(c.repository.clone()),
                _ => None,
            };
            if repository
                .as_ref()
                .and_then(|r| self.network_due.get(r))
                .is_some_and(|at| *at > Instant::now())
            {
                continue;
            }
            let upstream_key = upstream_key(&plan, &e.declaration);
            let upstream = upstream_key
                .as_ref()
                .and_then(|k| self.inventories.get(k))
                .cloned();
            if let Some(ref upstream_key) = upstream_key {
                let producer = &self.entries[upstream_key];
                let ready = upstream.as_ref().is_some_and(|a| {
                    a.revision == producer.declaration.revision && a.input == producer.input
                });
                if !ready {
                    if !self.running.contains_key(upstream_key)
                        && !self.pending.contains(upstream_key)
                    {
                        if producer.failures > 0 {
                            let entry = self.entries.get_mut(&key).ok_or("missing entry")?;
                            entry.status.state = CollectorState::Waiting;
                            entry.status.message =
                                Some("Upstream inventory failed; rerun its collector".into());
                            entry.status.updated_at = Utc::now();
                            self.pending.remove(&key);
                            self.failures = true;
                            continue;
                        }
                        self.pending.insert(upstream_key.clone());
                    }
                    continue;
                }
            }
            changed = true;
            let e = self.entries.get_mut(&key).ok_or("missing collector")?;
            e.attempt = e.attempt.saturating_add(1);
            e.status.state = CollectorState::Running;
            e.status.message = None;
            e.status.updated_at = Utc::now();
            e.status.last_attempt_at = Some(Utc::now());
            let declaration = e.declaration.clone();
            let effective = e.effective.clone();
            let input = e.input.clone();
            let run_id = uuid::Uuid::new_v4().to_string();
            let run_dir = self.location.state.join("runs").join(&run_id);
            files::private_dir(&run_dir)?;
            let forge_config = plan
                .forge_configs
                .get(&key)
                .map(|s| {
                    let path = run_dir.join("mise.toml");
                    files::write(&path, s.as_bytes()).map(|()| path)
                })
                .transpose()?;
            let root = self.location.root.clone();
            let source_directory = plan.source_directory.clone();
            let previous = self.snapshots.get(&key).cloned();
            let observation = EvidenceObservation {
                schema_version: 1,
                observation_id: uuid::Uuid::new_v4().to_string(),
                manifest_id: declaration.manifest_id.clone(),
                section: declaration.section,
                collector_id: declaration.config.id().into(),
                integration: declaration.config.kind().into(),
                workspace_id: self.location.workspace.clone(),
                environment: "local".into(),
                scope: observation_scope(&declaration.config, &plan.source_scope)?,
                plan_revision: plan.revision.clone(),
                collector_revision: declaration.revision.clone(),
                run_id,
                attempt: e.attempt,
                observed_at: Utc::now(),
                started_at: Utc::now(),
                recorded_at: None,
                source_updated_at: None,
                input_fingerprint: input.clone(),
                commit_sha: None,
                branch: None,
                dirty: None,
                tool_version: None,
                upstream_fingerprint: upstream.as_ref().map(|a| a.inventory.artifact_hash.clone()),
                policy_revision: None,
                result: EvidenceResult::Coverage(crystal_core::evidence::CoverageResult {
                    suite: "pending".into(),
                    covered: 0,
                    total: 0,
                }),
            };
            let artifact_directory = run_dir.clone();
            let task_key = key.clone();
            let task_effective = effective.clone();
            let abort = self.tasks.spawn(async move {
                let mut observation = observation;
                if let Some(git) =
                    integrations::provenance(&root, &source_directory, &declaration.config).await
                {
                    observation.commit_sha = git.commit;
                    observation.branch = git.branch;
                    observation.dirty = Some(git.dirty);
                }
                let result = integrations::execute(
                    &declaration,
                    &root,
                    &source_directory,
                    &run_dir,
                    upstream.as_ref(),
                    previous.as_ref(),
                    &input,
                    forge_config.as_deref(),
                )
                .await;
                Completion {
                    key: task_key,
                    effective: task_effective,
                    observation,
                    result,
                }
            });
            self.running.insert(
                key.clone(),
                Running {
                    abort,
                    effective,
                    heavy: is_heavy,
                    benchmark,
                    directory: artifact_directory,
                },
            );
            self.pending.remove(&key);
            if let Some(repo) = repository {
                self.network_due
                    .insert(repo, Instant::now() + Duration::from_secs(2));
            }
        }
        if changed {
            self.persist_statuses().await?;
        }
        Ok(())
    }
    #[allow(clippy::too_many_lines)] // A completed job atomically advances evidence, provenance and its dependents.
    async fn complete(&mut self, mut done: Completion) -> Result<(), String> {
        if self
            .running
            .get(&done.key)
            .is_some_and(|r| r.effective == done.effective)
        {
            self.running.remove(&done.key);
        }
        let Some(entry) = self
            .entries
            .get_mut(&done.key)
            .filter(|e| e.effective == done.effective)
        else {
            return Ok(());
        };
        entry.status.updated_at = Utc::now();
        // A run spanning an edit retains its original input hash; the status now
        // carries the new one, so projections label the result outdated.
        if let Ok(current) = files::input_ignoring(
            &self.location.root,
            &self
                .plan
                .as_ref()
                .ok_or("collector plan is unavailable")?
                .source_directory,
            &entry.declaration.config,
            self.location.managed_directory(),
        ) {
            entry.input.clone_from(&current);
            entry.status.input_fingerprint = Some(current);
        }
        match done.result {
            Ok(mut collected) => {
                done.observation.observed_at = Utc::now();
                done.observation.tool_version = collected.version;
                done.observation.source_updated_at = collected.source_updated_at;
                if let EvidenceResult::Git(git) = &collected.result {
                    done.observation.commit_sha = git.commit.clone();
                    done.observation.branch = git.branch.clone();
                    done.observation.dirty = Some(git.dirty);
                }
                if let EvidenceResult::License(license) = &collected.result {
                    done.observation.policy_revision = Some(license.policy_revision.clone());
                }
                if let EvidenceResult::Benchmark(benchmark) = &mut collected.result
                    && let CollectorConfig::Hyperfine(config) = &entry.declaration.config
                    && let Some(id) = &config.baseline
                {
                    let prior = persistence::evidence_history(
                        &self.pool,
                        &context().clerk_org_id,
                        &entry.declaration.manifest_id,
                        entry.declaration.section,
                        entry.declaration.config.id(),
                        &self.location.workspace,
                        100,
                        0,
                    )
                    .await?;
                    benchmark.baseline_mean_seconds = prior
                        .iter()
                        .find(|o| {
                            o.observation_id == *id
                                && o.input_fingerprint == done.observation.input_fingerprint
                                && o.scope == done.observation.scope
                                && o.tool_version == done.observation.tool_version
                        })
                        .and_then(|o| {
                            if let EvidenceResult::Benchmark(b) = &o.result {
                                (!b.machine.ends_with(":unknown")
                                    && b.machine == benchmark.machine
                                    && b.command == benchmark.command
                                    && b.runs == benchmark.runs)
                                    .then_some(b.mean_seconds)
                            } else {
                                None
                            }
                        });
                }
                done.observation.result = collected.result;
                if let Err(error) = done.observation.validate() {
                    entry.status.state = CollectorState::Error;
                    entry.status.message = Some(format!("invalid collected evidence: {error}"));
                    self.failures = true;
                    return self.persist_statuses().await;
                }
                let hash = fingerprint(&(
                    &done.observation.result,
                    &done.observation.input_fingerprint,
                    &done.observation.upstream_fingerprint,
                    &done.observation.policy_revision,
                ))?;
                let dedup = matches!(
                    entry.declaration.config,
                    CollectorConfig::GitStatus(_)
                        | CollectorConfig::GithubActions(_)
                        | CollectorConfig::GithubPullRequests(_)
                        | CollectorConfig::Junit(_)
                        | CollectorConfig::Lcov(_)
                        | CollectorConfig::Cobertura(_)
                );
                let heartbeat = matches!(
                    entry.declaration.config,
                    CollectorConfig::GitStatus(_)
                        | CollectorConfig::GithubActions(_)
                        | CollectorConfig::GithubPullRequests(_)
                ) && entry.last_evidence.is_none_or(|at| {
                    at.elapsed().as_secs_f64() >= entry.declaration.config.common().freshness / 2.0
                });
                if !dedup || heartbeat || entry.previous_result.as_ref() != Some(&hash) {
                    let bytes = serde_json::to_vec(&done.observation).map_err(|e| e.to_string())?;
                    if bytes.len() > 2_000_000 {
                        entry.status.state = CollectorState::Error;
                        entry.status.message = Some(
                            "Normalized evidence exceeds 2 MB; narrow this collector's scope"
                                .into(),
                        );
                        self.failures = true;
                    } else {
                        let spool = self.location.state.join("spool");
                        files::private_dir(&spool)?;
                        let entries = std::fs::read_dir(&spool)
                            .map_err(|e| e.to_string())?
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|e| e.to_string())?;
                        let size = entries
                            .iter()
                            .filter_map(|e| e.metadata().ok())
                            .map(|m| m.len())
                            .sum::<u64>();
                        if entries.len() >= 1000 || size + bytes.len() as u64 > 100_000_000 {
                            return Err("local observation spool reached its 1000-item/100-MB limit; repair storage before collecting more".into());
                        }
                        files::write(
                            &spool.join(format!("{}.json", done.observation.observation_id)),
                            &bytes,
                        )?;
                        entry.previous_result = Some(hash);
                        entry.last_evidence = Some(Instant::now());
                        entry.status.state = CollectorState::Ready;
                        entry.status.message = None;
                    }
                } else {
                    entry.status.state = CollectorState::Ready;
                    entry.status.message =
                        Some("Source unchanged; last successful evidence retained".into());
                }
                entry.failures = 0;
                if let Some(inventory) = collected.inventory {
                    let changed = self.inventories.get(&done.key).is_none_or(|p| {
                        p.inventory.artifact_hash != inventory.inventory.artifact_hash
                    });
                    self.inventories.insert(done.key.clone(), inventory);
                    if changed && let Some(plan) = &self.plan {
                        for d in &plan.declarations {
                            if d.config.schedule().upstream_changed
                                && upstream_key(plan, d).as_ref() == Some(&done.key)
                            {
                                self.pending.insert(key(d));
                            }
                        }
                    }
                }
                if let Some(snapshot) = collected.snapshot {
                    self.snapshots.insert(done.key.clone(), snapshot);
                }
            }
            Err(error) => {
                self.failures = true;
                entry.failures = entry.failures.saturating_add(1);
                entry.status.state = if error.starts_with("missing tool:") {
                    CollectorState::MissingTool
                } else if error.starts_with("incompatible tool:") {
                    CollectorState::IncompatibleTool
                } else if error.starts_with("needs login:") {
                    CollectorState::NeedsLogin
                } else {
                    CollectorState::Error
                };
                entry.status.message = Some(error.chars().take(2000).collect());
                if entry.declaration.config.schedule().every.is_some() {
                    let seconds = 5u64.saturating_mul(1u64 << entry.failures.min(8)).min(1800);
                    entry.due = Some(
                        Instant::now()
                            + Duration::from_secs(seconds + u64::from(entry.attempt % 5)),
                    );
                }
            }
        }
        self.persist_statuses().await?;
        self.drain().await
    }
    async fn drain(&self) -> Result<(), String> {
        let spool = self.location.state.join("spool");
        if !spool.exists() {
            return Ok(());
        }
        for file in std::fs::read_dir(&spool)
            .map_err(|e| e.to_string())?
            .take(100)
        {
            let path = file.map_err(|e| e.to_string())?.path();
            let decoded = files::read(&path).and_then(|content| {
                serde_json::from_str::<EvidenceObservation>(&content).map_err(|e| e.to_string())
            });
            let observation = match decoded {
                Ok(observation)
                    if Utc::now()
                        .signed_duration_since(observation.observed_at)
                        .num_days()
                        <= 30 =>
                {
                    observation
                }
                invalid => {
                    let reason = invalid
                        .err()
                        .unwrap_or_else(|| "spooled observation expired after 30 days".into());
                    eprintln!("Quarantined {}: {reason}", path.display());
                    let quarantine = self.location.state.join("quarantine");
                    files::private_dir(&quarantine)?;
                    let target =
                        quarantine.join(path.file_name().ok_or("spool entry has no filename")?);
                    std::fs::rename(&path, target).map_err(|e| e.to_string())?;
                    files::write(
                        &self.location.state.join("delivery-error.txt"),
                        reason.as_bytes(),
                    )?;
                    continue;
                }
            };
            match persistence::record_evidence(&self.pool, &context(), observation).await {
                Ok(_) => std::fs::remove_file(&path).map_err(|e| e.to_string())?,
                Err(e) => {
                    eprintln!("Evidence delivery retained for retry: {e}");
                    break;
                }
            }
        }
        files::prune_runs(
            &self.location.state,
            &self
                .inventories
                .values()
                .map(|v| v.path.clone())
                .chain(self.running.values().map(|r| r.directory.clone()))
                .collect(),
        )
    }
}

fn observation_scope(config: &CollectorConfig, source_scope: &str) -> Result<String, String> {
    if let CollectorConfig::Hyperfine(c) = config {
        fingerprint(&(source_scope, &c.command, c.warmup, c.runs, &c.common.env))
    } else {
        Ok(Path::new(source_scope)
            .join(config.directory())
            .to_string_lossy()
            .into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(clippy::needless_pass_by_value)] // Test fixture helper consumes temporary JSON values.
    fn plan(root: &Path, value: serde_json::Value) -> Result<Plan, String> {
        let entrypoint = root.join("index.scry");
        files::write(&entrypoint, b"# fixture source")?;
        Plan::parse(&value.to_string(), root, &entrypoint)
    }
    fn location(root: &Path) -> Location {
        let id = uuid::Uuid::new_v4().to_string();
        Location {
            root: root.to_owned(),
            state: root.join("state"),
            database: root.join("local.db"),
            workspace: id.clone(),
            socket: PathBuf::from(format!("/tmp/scryr-test-{id}.sock")),
        }
    }
    #[tokio::test]
    async fn invalid_reload_retains_plan_and_unrelated_changes_preserve_jobs() -> Result<(), String>
    {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let mut owner = Owner::new(location(tmp.path()), false).await?;
        let a = plan(
            tmp.path(),
            serde_json::json!({"manifests":[{"manifestId":"api","tests":[{"kind":"junit","id":"unit","files":["unit.xml"]}]}]}),
        )?;
        owner.replace(a.clone(), true).await?;
        let key = key(&a.declarations[0]);
        owner.entries.get_mut(&key).ok_or("entry")?.attempt = 7;
        let b = plan(
            tmp.path(),
            serde_json::json!({"description":"unrelated","manifests":[{"manifestId":"api","tests":[{"kind":"junit","id":"unit","files":["unit.xml"]}]}]}),
        )?;
        owner.replace(b, true).await?;
        assert_eq!(owner.entries[&key].attempt, 7);
        let invalid = plan(
            tmp.path(),
            serde_json::json!({"manifests":[{"manifestId":"api","checks":[{"kind":"ruff","directory":"missing"}]}]}),
        )?;
        assert!(owner.replace(invalid, true).await.is_err());
        assert_eq!(owner.entries[&key].attempt, 7);
        assert_eq!(
            owner.plan.as_ref().ok_or("plan")?.declarations[0]
                .config
                .id(),
            "unit"
        );
        Ok(())
    }
    #[tokio::test]
    async fn report_run_records_valid_failures_and_spool_retries_are_idempotent()
    -> Result<(), String> {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        files::write(&tmp.path().join("tests.xml"),b"<testsuite><testcase name=\"fails\"><failure message=\"expected\"/></testcase></testsuite>")?;
        let plan = plan(
            tmp.path(),
            serde_json::json!({"manifests":[{"manifestId":"api","tests":[{"kind":"junit","files":["tests.xml"],"schedule":{"debounce":0}}]}]}),
        )?;
        let mut owner = Owner::new(location(tmp.path()), false).await?;
        owner.replace(plan, false).await?;
        owner.queue(&Selection::default())?;
        let (sender, receiver) = watch::channel(None);
        owner.run(receiver, Some(())).await?;
        drop(sender);
        let observations = persistence::evidence_history(
            &owner.pool,
            "local-dev-org",
            "api",
            crystal_core::evidence::EvidenceSection::Tests,
            "junit",
            &owner.location.workspace,
            10,
            0,
        )
        .await?;
        assert_eq!(observations.len(), 1);
        let EvidenceResult::Test(result) = &observations[0].result else {
            return Err("expected tests".into());
        };
        assert_eq!(result.failing, 1);
        let observation = &observations[0];
        files::write(
            &owner.location.state.join("spool/retry.json"),
            &serde_json::to_vec(observation).map_err(|e| e.to_string())?,
        )?;
        owner.drain().await?;
        assert_eq!(
            persistence::evidence_history(
                &owner.pool,
                "local-dev-org",
                "api",
                crystal_core::evidence::EvidenceSection::Tests,
                "junit",
                &owner.location.workspace,
                10,
                0
            )
            .await?
            .len(),
            1
        );
        Ok(())
    }
    #[tokio::test]
    async fn owner_lock_is_independent_of_custom_state_directory() -> Result<(), String> {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let first = location(tmp.path());
        let mut second = location(tmp.path());
        second.workspace = first.workspace.clone();
        second.socket = first.socket.clone();
        second.state = tmp.path().join("other-state");
        let _owner = Owner::new(first, false).await?;
        assert!(Owner::new(second, false).await.is_err());
        Ok(())
    }
    #[test]
    fn selected_baseline_does_not_change_execution_scope() -> Result<(), String> {
        let a: CollectorConfig = serde_json::from_value(
            serde_json::json!({"kind":"hyperfine","command":{"executable":"echo","args":["hi"]}}),
        )
        .map_err(|e| e.to_string())?;
        let mut b = a.clone();
        if let CollectorConfig::Hyperfine(c) = &mut b {
            c.baseline = Some("run-one".into());
        }
        assert_ne!(a.revision()?, b.revision()?);
        assert_eq!(observation_scope(&a, ".")?, observation_scope(&b, ".")?);
        assert_ne!(
            observation_scope(&a, ".")?,
            observation_scope(&a, "services/api")?
        );
        Ok(())
    }
    #[test]
    fn entrypoint_base_changes_effective_revision_without_changing_sdk_configuration()
    -> Result<(), String> {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let root = tmp.path().canonicalize().map_err(|e| e.to_string())?;
        let nested = root.join("services/api/index.scry");
        let top = root.join("index.scry");
        files::write(&nested, b"# nested fixture")?;
        files::write(&top, b"# root fixture")?;
        let json = serde_json::json!({"manifests":[{"manifestId":"api","tests":[{"kind":"junit","files":["results.xml"]}]}]}).to_string();
        let a = Plan::parse(&json, &root, &top)?;
        let b = Plan::parse(&json, &root, &nested)?;
        assert_eq!(b.source_directory, root.join("services/api"));
        assert_eq!(a.declarations[0].revision, b.declarations[0].revision);
        assert_ne!(a.revision, b.revision);
        assert_ne!(
            effective(&a, &a.declarations[0])?,
            effective(&b, &b.declarations[0])?
        );
        let outside = tempfile::tempdir().map_err(|e| e.to_string())?;
        files::write(&outside.path().join("index.scry"), b"# outside fixture")?;
        assert!(Plan::parse(&json, &root, &outside.path().join("index.scry")).is_err());
        Ok(())
    }
}
