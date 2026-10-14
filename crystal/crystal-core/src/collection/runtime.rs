use super::{CollectionPlan, Collector, CollectorCard, Observation, Task, schema};
use async_graphql::Json;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{
    Row, SqliteConnection, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{
    collections::HashMap,
    fmt::Write,
    fs::{File, OpenOptions},
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    sync::Mutex,
    time::Instant,
};
use uuid::Uuid;

/// Shared local runner and isolated observation store. Opening never executes tasks.
#[derive(Clone)]
pub struct CollectionRuntime {
    inner: Arc<Inner>,
}
struct Inner {
    root: PathBuf,
    plan: CollectionPlan,
    revision: String,
    pool: SqlitePool,
    execution: Mutex<()>,
}
impl CollectionRuntime {
    /// Open a validated plan and the manifest-local observation database.
    ///
    /// # Errors
    /// Returns validation, filesystem, or database failures.
    pub async fn open(root: PathBuf, plan: CollectionPlan) -> Result<Self, String> {
        plan.validate()?;
        let root = root.canonicalize().map_err(|error| error.to_string())?;
        let state = root.join(".scryr");
        tokio::fs::create_dir_all(&state)
            .await
            .map_err(|error| error.to_string())?;
        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(state.join("collectors.sqlite"))
                    .create_if_missing(true)
                    .journal_mode(SqliteJournalMode::Wal)
                    .busy_timeout(Duration::from_secs(5)),
            )
            .await
            .map_err(|error| error.to_string())?;
        sqlx::query("CREATE TABLE IF NOT EXISTS collector_observations (id TEXT PRIMARY KEY, collector_id TEXT NOT NULL, plan_revision TEXT NOT NULL, started_ms INTEGER NOT NULL, finished_ms INTEGER NOT NULL, expires_ms INTEGER NOT NULL, status TEXT NOT NULL, exit_code INTEGER, error TEXT, data TEXT)").execute(&pool).await.map_err(|error| error.to_string())?;
        sqlx::query("CREATE INDEX IF NOT EXISTS collector_history ON collector_observations (collector_id, plan_revision, finished_ms DESC)").execute(&pool).await.map_err(|error| error.to_string())?;
        let revision =
            Sha256::digest(serde_json::to_vec(&plan).map_err(|error| error.to_string())?)
                .iter()
                .fold(String::new(), |mut output, byte| {
                    let _ = write!(output, "{byte:02x}");
                    output
                });
        Ok(Self {
            inner: Arc::new(Inner {
                root,
                plan,
                revision,
                pool,
                execution: Mutex::new(()),
            }),
        })
    }
    /// Execute one collector, or every collector, persisting successes and failures.
    ///
    /// # Errors
    /// Returns an error after persisting failed executions, or when another runner owns the lease.
    pub async fn run_once(&self, collector_id: Option<&str>) -> Result<(), String> {
        if collector_id.is_some_and(|id| {
            !self
                .inner
                .plan
                .collectors
                .iter()
                .any(|collector| collector.id == id)
        }) {
            return Err("unknown collector ID".into());
        }
        let _local = self
            .inner
            .execution
            .try_lock()
            .map_err(|_| "collection already running")?;
        let _lease = self.lease()?;
        let mut failed = Vec::new();
        for collector in &self.inner.plan.collectors {
            if collector_id.is_none_or(|id| id == collector.id)
                && !self.run_collector(collector).await?
            {
                failed.push(collector.id.clone());
            }
        }
        if failed.is_empty() {
            Ok(())
        } else {
            Err(format!("collection failed: {}", failed.join(", ")))
        }
    }
    /// Run collectors sequentially on their intervals, skipping missed intervals.
    ///
    /// Dropping this future cancels the active process and releases the runner lease.
    /// # Errors
    /// Returns lease or storage failures. Individual execution failures do not stop scheduling.
    pub async fn run_scheduler(&self) -> Result<(), String> {
        let _local = self
            .inner
            .execution
            .try_lock()
            .map_err(|_| "collection already running")?;
        let _lease = self.lease()?;
        let mut deadlines: HashMap<String, Instant> = self
            .inner
            .plan
            .collectors
            .iter()
            .map(|collector| (collector.id.clone(), Instant::now()))
            .collect();
        loop {
            for collector in &self.inner.plan.collectors {
                if deadlines
                    .get(&collector.id)
                    .is_some_and(|deadline| *deadline <= Instant::now())
                {
                    self.run_collector(collector).await?;
                    if let Some(deadline) = deadlines.get_mut(&collector.id) {
                        let interval = u64::from(collector.interval_seconds);
                        let skipped = deadline.elapsed().as_secs() / interval + 1;
                        *deadline += Duration::from_secs(interval * skipped);
                    }
                }
            }
            let next = deadlines
                .values()
                .min()
                .copied()
                .unwrap_or_else(|| Instant::now() + Duration::from_mins(1));
            tokio::time::sleep_until(next).await;
        }
    }
    fn lease(&self) -> Result<File, String> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.inner.root.join(".scryr/collectors.lock"))
            .map_err(|error| error.to_string())?;
        file.try_lock()
            .map_err(|_| "another collector process is running".to_string())?;
        Ok(file)
    }
    async fn run_collector(&self, collector: &Collector) -> Result<bool, String> {
        let task = self
            .inner
            .plan
            .tasks
            .iter()
            .find(|task| task.id == collector.task_id)
            .ok_or("missing collector task")?;
        let started = now_ms()?;
        let result = execute(&self.inner.root, task).await;
        let finished = now_ms()?;
        let data = result
            .data
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| error.to_string())?;
        sqlx::query("INSERT INTO collector_observations (id, collector_id, plan_revision, started_ms, finished_ms, expires_ms, status, exit_code, error, data) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string()).bind(&collector.id).bind(&self.inner.revision).bind(started).bind(finished).bind(finished + i64::from(collector.retain_for_seconds) * 1000).bind(result.status).bind(result.exit_code).bind(result.error).bind(data).execute(&self.inner.pool).await.map_err(|error| error.to_string())?;
        self.prune().await?;
        Ok(result.status == "success")
    }
    async fn prune(&self) -> Result<(), String> {
        let now = now_ms()?;
        sqlx::query("DELETE FROM collector_observations WHERE expires_ms <= ?")
            .bind(now)
            .execute(&self.inner.pool)
            .await
            .map_err(|error| error.to_string())?;
        for collector in &self.inner.plan.collectors {
            sqlx::query(
                "DELETE FROM collector_observations WHERE collector_id = ? AND finished_ms < ?",
            )
            .bind(&collector.id)
            .bind(now - i64::from(collector.retain_for_seconds) * 1000)
            .execute(&self.inner.pool)
            .await
            .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
    /// Read card observations without executing any task.
    ///
    /// # Errors
    /// Returns storage or corrupt observation errors.
    pub async fn snapshot(&self) -> Result<Vec<CollectorCard>, String> {
        self.prune().await?;
        let mut sources = HashMap::new();
        // One database snapshot keeps every card and its last successful report
        // consistent even if a collector commits while these queries run.
        let mut transaction = self
            .inner
            .pool
            .begin()
            .await
            .map_err(|error| error.to_string())?;
        for collector in &self.inner.plan.collectors {
            let history = self.read(&mut transaction, &collector.id, false).await?;
            let last_success = self
                .read(&mut transaction, &collector.id, true)
                .await?
                .into_iter()
                .next();
            sources.insert(&collector.id, (collector, history, last_success));
        }
        transaction
            .commit()
            .await
            .map_err(|error| error.to_string())?;
        self.inner
            .plan
            .cards
            .iter()
            .map(|card| {
                let (collector, history, last_success) = sources
                    .get(&card.collector_id)
                    .ok_or("missing card source")?;
                Ok(CollectorCard {
                    id: card.id.clone(),
                    kind: card.kind.clone(),
                    title: card.title.clone(),
                    target: card.target.clone(),
                    collector_id: collector.id.clone(),
                    environment: collector.environment.clone(),
                    stale_after_seconds: collector.stale_after_seconds,
                    latest: history.first().cloned(),
                    last_success: last_success.clone(),
                    history: history.clone(),
                })
            })
            .collect()
    }
    async fn read(
        &self,
        connection: &mut SqliteConnection,
        collector_id: &str,
        success_only: bool,
    ) -> Result<Vec<Observation>, String> {
        let rows = sqlx::query("SELECT id, collector_id, plan_revision, strftime('%Y-%m-%dT%H:%M:%fZ', started_ms / 1000.0, 'unixepoch') AS started_at, strftime('%Y-%m-%dT%H:%M:%fZ', finished_ms / 1000.0, 'unixepoch') AS finished_at, status, exit_code, error, data FROM collector_observations WHERE collector_id = ? AND plan_revision = ? AND (? = 0 OR status = 'success') ORDER BY finished_ms DESC, rowid DESC LIMIT ?")
            .bind(collector_id).bind(&self.inner.revision).bind(success_only).bind(if success_only { 1 } else { 20 }).fetch_all(connection).await.map_err(|error| error.to_string())?;
        rows.iter()
            .map(|row| {
                let data: Option<String> =
                    row.try_get("data").map_err(|error| error.to_string())?;
                Ok(Observation {
                    id: row.get("id"),
                    collector_id: row.get("collector_id"),
                    plan_revision: row.get("plan_revision"),
                    started_at: row.get("started_at"),
                    finished_at: row.get("finished_at"),
                    status: row.get("status"),
                    exit_code: row.get("exit_code"),
                    error: row.get("error"),
                    data: data
                        .map(|raw| serde_json::from_str(&raw).map(Json))
                        .transpose()
                        .map_err(|error| error.to_string())?,
                })
            })
            .collect()
    }
}
fn now_ms() -> Result<i64, String> {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis(),
    )
    .map_err(|error| error.to_string())
}
struct Outcome {
    status: &'static str,
    exit_code: Option<i32>,
    error: Option<String>,
    data: Option<Value>,
}
impl Outcome {
    fn failure(status: &'static str, message: &str, exit_code: Option<i32>) -> Self {
        Self {
            status,
            exit_code,
            error: Some(message.to_owned()),
            data: None,
        }
    }
}
struct ProcessGroup {
    pid: u32,
}
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Ok(pid) = i32::try_from(self.pid)
            && pid > 0
        {
            let _ = nix::sys::signal::killpg(
                nix::unistd::Pid::from_raw(pid),
                nix::sys::signal::Signal::SIGKILL,
            );
        }
    }
}
async fn bounded_read(mut stream: impl AsyncRead + Unpin, limit: usize) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut buffer = vec![0; 8192];
    loop {
        let count = stream
            .read(&mut buffer)
            .await
            .map_err(|_| "could not read command output")?;
        if count == 0 {
            return Ok(output);
        }
        if output.len() + count > limit {
            return Err("command output exceeded byte limit".into());
        }
        output.extend_from_slice(&buffer[..count]);
    }
}
async fn execute(root: &std::path::Path, task: &Task) -> Outcome {
    let mut command = Command::new(&task.command.argv[0]);
    command
        .args(&task.command.argv[1..])
        .current_dir(root.join(&task.command.cwd))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    command.process_group(0);
    let Ok(mut child) = command.spawn() else {
        return Outcome::failure(
            "failed",
            "could not start command; check executable and working directory",
            None,
        );
    };
    let _group = ProcessGroup {
        pid: child.id().unwrap_or(0),
    };
    let Some(stdout) = child.stdout.take() else {
        return Outcome::failure("failed", "command stdout unavailable", None);
    };
    let Some(stderr) = child.stderr.take() else {
        return Outcome::failure("failed", "command stderr unavailable", None);
    };
    let execution = async {
        let (status, stdout, _) = tokio::try_join!(
            async {
                child
                    .wait()
                    .await
                    .map_err(|_| "could not wait for command".to_owned())
            },
            bounded_read(stdout, task.max_output_bytes as usize),
            bounded_read(stderr, task.max_output_bytes as usize)
        )?;
        Ok::<_, String>((status, stdout))
    };
    let outcome = tokio::time::timeout(
        Duration::from_secs(u64::from(task.timeout_seconds)),
        execution,
    )
    .await;
    match outcome {
        Err(_) => {
            let _ = child.start_kill();
            Outcome::failure("timeout", "command exceeded its execution deadline", None)
        }
        Ok(Err(error)) => {
            let _ = child.start_kill();
            Outcome::failure("failed", &error, None)
        }
        Ok(Ok((status, stdout))) => {
            let exit_code = status.code();
            if !exit_code.is_some_and(|code| task.accepted_exit_codes.contains(&code)) {
                return Outcome::failure(
                    "failed",
                    "command exited with an unaccepted status",
                    exit_code,
                );
            }
            let Ok(data) = serde_json::from_slice::<Value>(&stdout) else {
                return Outcome::failure(
                    "invalid_output",
                    "command did not return a JSON report",
                    exit_code,
                );
            };
            if let Err(error) = schema::validate_data(&task.output_schema, &data) {
                return Outcome::failure("invalid_output", &error, exit_code);
            }
            Outcome {
                status: "success",
                exit_code,
                error: None,
                data: Some(data),
            }
        }
    }
}
#[cfg(test)]
mod tests;
