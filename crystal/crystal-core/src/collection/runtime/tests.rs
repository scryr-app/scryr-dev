use super::*;
use crate::collection::{Card, Command as TaskCommand};
use serde_json::json;

fn plan(script: &str) -> CollectionPlan {
    CollectionPlan {
        tasks: vec![Task {
            id: "report".into(),
            command: TaskCommand {
                argv: vec!["/bin/sh".into(), "-c".into(), script.into()],
                cwd: ".".into(),
            },
            output_schema: json!({"type":"object","properties":{"healthy":{"type":"boolean"}},"required":["healthy"],"additionalProperties":false}),
            timeout_seconds: 1,
            max_output_bytes: 1024,
            accepted_exit_codes: vec![0],
        }],
        collectors: vec![Collector {
            id: "production".into(),
            task_id: "report".into(),
            environment: "production".into(),
            interval_seconds: 30,
            stale_after_seconds: 60,
            retain_for_seconds: 3600,
        }],
        cards: ["status", "history"]
            .into_iter()
            .map(|kind| Card {
                id: kind.into(),
                kind: kind.into(),
                title: kind.into(),
                collector_id: "production".into(),
                target: "api".into(),
            })
            .collect(),
        ..CollectionPlan::default()
    }
}
#[tokio::test]
async fn snapshots_never_execute_and_cards_share_persisted_observations() -> Result<(), String> {
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    let plan = plan("printf '{\"healthy\":true}'");
    let runtime = CollectionRuntime::open(directory.path().into(), plan.clone()).await?;
    assert!(
        runtime
            .snapshot()
            .await?
            .iter()
            .all(|card| card.latest.is_none())
    );
    runtime.run_once(None).await?;
    let cards = runtime.snapshot().await?;
    assert_eq!(
        cards[0].latest.as_ref().map(|item| &item.id),
        cards[1].latest.as_ref().map(|item| &item.id)
    );
    assert_eq!(cards[0].history.len(), 1);
    let reopened = CollectionRuntime::open(directory.path().into(), plan.clone()).await?;
    assert_eq!(reopened.snapshot().await?[0].history.len(), 1);
    let mut changed = plan;
    changed.tasks[0].timeout_seconds = 2;
    let revised = CollectionRuntime::open(directory.path().into(), changed).await?;
    assert!(revised.snapshot().await?[0].latest.is_none());
    Ok(())
}
#[tokio::test]
async fn failures_are_persisted_without_raw_output() -> Result<(), String> {
    for (script, status) in [
        ("printf '{\"healthy\":\"yes\"}'", "invalid_output"),
        ("printf secret; exit 3", "failed"),
        ("sleep 10", "timeout"),
        ("head -c 4096 /dev/zero", "failed"),
    ] {
        let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
        let runtime = CollectionRuntime::open(directory.path().into(), plan(script)).await?;
        assert!(runtime.run_once(None).await.is_err());
        let card = runtime.snapshot().await?.remove(0);
        let latest = card.latest.ok_or("missing persisted failure")?;
        assert_eq!(latest.status, status);
        assert!(latest.data.is_none());
        assert!(
            !latest
                .error
                .as_deref()
                .unwrap_or_default()
                .contains("secret")
        );
    }
    Ok(())
}
#[tokio::test]
async fn accepts_report_exit_codes_and_retains_last_success() -> Result<(), String> {
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    let mut plan =
        plan("if test -f fail; then exit 3; else printf '{\"healthy\":false}'; exit 1; fi");
    plan.tasks[0].accepted_exit_codes.push(1);
    let runtime = CollectionRuntime::open(directory.path().into(), plan).await?;
    runtime.run_once(None).await?;
    tokio::fs::write(directory.path().join("fail"), "")
        .await
        .map_err(|error| error.to_string())?;
    assert!(runtime.run_once(None).await.is_err());
    let card = runtime.snapshot().await?.remove(0);
    assert_eq!(card.latest.ok_or("missing latest")?.status, "failed");
    assert_eq!(
        card.last_success
            .ok_or("missing previous success")?
            .exit_code,
        Some(1)
    );
    sqlx::query("UPDATE collector_observations SET finished_ms = 0")
        .execute(&runtime.inner.pool)
        .await
        .map_err(|error| error.to_string())?;
    assert!(runtime.snapshot().await?[0].history.is_empty());
    Ok(())
}
#[tokio::test]
async fn runner_lease_does_not_prevent_read_only_open() -> Result<(), String> {
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    let plan = plan("printf '{\"healthy\":true}'");
    let runtime = CollectionRuntime::open(directory.path().into(), plan.clone()).await?;
    let _lease = runtime.lease()?;
    let reader = CollectionRuntime::open(directory.path().into(), plan).await?;
    assert!(
        reader
            .snapshot()
            .await?
            .iter()
            .all(|card| card.latest.is_none())
    );
    assert!(reader.run_once(None).await.is_err());
    Ok(())
}
#[test]
fn rejects_unsupported_and_recursive_output_contracts() {
    for schema in [
        json!({"type":"string", "pattern":".*"}),
        json!({"$defs":{"Node":{"$ref":"#/$defs/Node"}}, "$ref":"#/$defs/Node"}),
    ] {
        let mut plan = plan("true");
        plan.tasks[0].output_schema = schema;
        assert!(plan.validate().is_err());
    }
}
#[test]
fn validates_nested_constraints_and_optional_fields() -> Result<(), String> {
    let schema = json!({"$defs":{"Item":{"type":"integer","minimum":1}},"type":"array","minItems":1,"items":{"anyOf":[{"$ref":"#/$defs/Item"},{"type":"null"}]}});
    schema::validate_schema(&schema)?;
    assert!(schema::validate_data(&schema, &json!([1, null])).is_ok());
    assert!(schema::validate_data(&schema, &json!([0])).is_err());
    assert!(schema::validate_data(&schema, &json!([])).is_err());
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn cancellation_and_timeout_kill_descendant_processes() -> Result<(), String> {
    for cancel in [false, true] {
        let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
        let mut plan = plan("touch started; (sleep 2; touch escaped) & wait");
        if cancel {
            plan.tasks[0].timeout_seconds = 10;
        }
        let runtime = CollectionRuntime::open(directory.path().into(), plan).await?;
        let worker = runtime.clone();
        let handle = tokio::spawn(async move { worker.run_once(None).await });
        if cancel {
            tokio::time::timeout(Duration::from_secs(5), async {
                while !directory.path().join("started").exists() {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .map_err(|error| error.to_string())?;
            handle.abort();
            assert!(handle.await.is_err());
        } else {
            assert!(handle.await.map_err(|error| error.to_string())?.is_err());
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(!directory.path().join("escaped").exists());
        let _lease = runtime.lease()?;
    }
    Ok(())
}

#[test]
fn numeric_bounds_preserve_large_integer_precision() -> Result<(), String> {
    let schema = json!({"type":"number","minimum":9_007_199_254_740_993_u64});
    schema::validate_schema(&schema)?;
    assert!(schema::validate_data(&schema, &json!(9_007_199_254_740_992_u64)).is_err());
    assert!(schema::validate_data(&schema, &json!(9_007_199_254_740_992_f64)).is_err());
    assert!(schema::validate_data(&schema, &json!(9_007_199_254_740_993_u64)).is_ok());
    assert!(schema::validate_data(&schema, &json!(9_007_199_254_740_994_f64)).is_ok());
    let fractional = json!({"type":"number","maximum":-0.5});
    assert!(schema::validate_data(&fractional, &json!(0)).is_err());
    assert!(schema::validate_data(&fractional, &json!(-1)).is_ok());
    Ok(())
}

#[tokio::test]
async fn expired_observations_are_removed_after_the_collector_is_removed() -> Result<(), String> {
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    let runtime =
        CollectionRuntime::open(directory.path().into(), plan("printf '{\"healthy\":true}'"))
            .await?;
    runtime.run_once(None).await?;
    sqlx::query("UPDATE collector_observations SET expires_ms = 0")
        .execute(&runtime.inner.pool)
        .await
        .map_err(|error| error.to_string())?;
    let revised =
        CollectionRuntime::open(directory.path().into(), CollectionPlan::default()).await?;
    assert!(revised.snapshot().await?.is_empty());
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM collector_observations")
        .fetch_one(&revised.inner.pool)
        .await
        .map_err(|error| error.to_string())?;
    assert_eq!(remaining, 0);
    Ok(())
}

#[tokio::test]
async fn snapshot_transaction_keeps_latest_and_last_success_consistent() -> Result<(), String> {
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    let runtime =
        CollectionRuntime::open(directory.path().into(), plan("printf '{\"healthy\":true}'"))
            .await?;
    runtime.run_once(None).await?;
    let mut transaction = runtime
        .inner
        .pool
        .begin()
        .await
        .map_err(|error| error.to_string())?;
    let history = runtime.read(&mut transaction, "production", false).await?;
    runtime.run_once(None).await?;
    let success = runtime.read(&mut transaction, "production", true).await?;
    assert_eq!(
        history.first().map(|item| &item.id),
        success.first().map(|item| &item.id)
    );
    transaction
        .commit()
        .await
        .map_err(|error| error.to_string())?;
    let cards = runtime.snapshot().await?;
    assert_eq!(cards[0].history.len(), 2);
    assert_eq!(
        cards[0].latest.as_ref().map(|item| &item.id),
        cards[0].last_success.as_ref().map(|item| &item.id)
    );
    assert_ne!(
        history.first().map(|item| &item.id),
        cards[0].latest.as_ref().map(|item| &item.id)
    );
    Ok(())
}
