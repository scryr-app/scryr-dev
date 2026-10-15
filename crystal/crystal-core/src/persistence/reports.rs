//! Organization-scoped operational report storage.
use super::{DatabasePool, schema};
use crate::{manifest::ManifestRequestContext, reports::Report};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

pub(super) const CREATE_TABLE: &str = "CREATE TABLE IF NOT EXISTS manifest_reports (clerk_org_id TEXT NOT NULL, manifest_id TEXT NOT NULL, fingerprint TEXT NOT NULL, kind TEXT NOT NULL, scope TEXT NOT NULL, observed_at TEXT NOT NULL, recorded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, content TEXT NOT NULL, PRIMARY KEY(clerk_org_id, manifest_id, fingerprint))";

/// Store a validated report, returning false for an identical retry.
/// # Errors
/// Rejects invalid reports, unauthorized principals and storage failures.
pub async fn record_report(
    pool: &DatabasePool,
    context: &ManifestRequestContext,
    id: &str,
    report: Report,
) -> Result<bool, String> {
    crate::reports::validate_manifest_id(id)?;
    if context.clerk_org_id.is_empty() || !context.can_write_generated_manifests() {
        return Err("active organization role cannot report".into());
    }
    report.validate()?;
    let serialized = serde_json::to_string(&report).map_err(|e| e.to_string())?;
    if serialized.len() > 2_000_000 {
        return Err("report exceeds 2 MB".into());
    }
    let fingerprint = Sha256::digest(serialized.as_bytes()).iter().fold(
        String::with_capacity(64),
        |mut output, byte| {
            let _ = write!(output, "{byte:02x}");
            output
        },
    );
    let params = vec![
        context.clerk_org_id.clone(),
        id.into(),
        fingerprint,
        report.kind().into(),
        report.scope.clone(),
        report
            .observed_at
            .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        serialized,
    ];
    schema::ensure_table(pool).await?;
    let sql = "INSERT INTO manifest_reports (clerk_org_id,manifest_id,fingerprint,kind,scope,observed_at,content) VALUES (?,?,?,?,?,?,?) ON CONFLICT DO NOTHING";
    let count = match pool {
        DatabasePool::Sqlite(p) => {
            let mut q = sqlx::query(sql);
            for v in &params {
                q = q.bind(v);
            }
            q.execute(p)
                .await
                .map_err(|e| e.to_string())?
                .rows_affected()
        }
        DatabasePool::Turso(db) => db
            .connect()
            .map_err(|e| e.to_string())?
            .execute(
                sql,
                params
                    .into_iter()
                    .map(libsql::Value::Text)
                    .collect::<Vec<_>>(),
            )
            .await
            .map_err(|e| e.to_string())?,
    };
    Ok(count > 0)
}

async fn query(
    pool: &DatabasePool,
    sql: &'static str,
    params: Vec<String>,
) -> Result<Vec<Report>, String> {
    schema::ensure_table(pool).await?;
    let contents: Vec<String> = match pool {
        DatabasePool::Sqlite(p) => {
            let mut q = sqlx::query_scalar(sql);
            for v in &params {
                q = q.bind(v);
            }
            q.fetch_all(p).await.map_err(|e| e.to_string())?
        }
        DatabasePool::Turso(db) => {
            let c = db.connect().map_err(|e| e.to_string())?;
            let mut rows = c
                .query(
                    sql,
                    params
                        .into_iter()
                        .map(libsql::Value::Text)
                        .collect::<Vec<_>>(),
                )
                .await
                .map_err(|e| e.to_string())?;
            let mut out = vec![];
            while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
                out.push(row.get(0).map_err(|e| e.to_string())?);
            }
            out
        }
    };
    contents
        .into_iter()
        .map(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
        .collect()
}

/// Read newest observations, scoped to the authenticated organization.
/// # Errors
/// Rejects invalid pagination, identities and storage failures.
pub async fn read_reports(
    pool: &DatabasePool,
    org: &str,
    id: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<Report>, String> {
    crate::reports::validate_manifest_id(id)?;
    if org.is_empty() || !(1..=100).contains(&limit) {
        return Err("organization and limit 1..100 required".into());
    }
    query(pool,"SELECT content FROM manifest_reports WHERE clerk_org_id=? AND manifest_id=? ORDER BY observed_at DESC, CAST(json_extract(content, '$.attempt') AS INTEGER) DESC, fingerprint DESC LIMIT ? OFFSET ?",vec![org.into(),id.into(),limit.to_string(),offset.to_string()]).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::ReportData;
    fn principal(org: &str) -> ManifestRequestContext {
        ManifestRequestContext {
            clerk_user_id: "u".into(),
            clerk_org_id: org.into(),
            clerk_org_slug: None,
            clerk_org_role: Some("org:admin".into()),
            clerk_org_permissions: vec![],
        }
    }
    fn report(time: &str, passing: u64) -> Result<Report, String> {
        Ok(Report {
            schema_version: 1,
            source: "junit".into(),
            scope: "unit".into(),
            run_id: "1".into(),
            attempt: 1,
            commit_sha: None,
            branch: Some("main".into()),
            observed_at: time.parse().map_err(|_| "time")?,
            report_url: None,
            data: ReportData::Tests {
                passing,
                failing: 0,
                errors: 0,
                skipped: 0,
                duration: 1.0,
            },
        })
    }
    #[tokio::test]
    async fn reports_are_durable_deduplicated_ordered_and_isolated()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("reports.db");
        let pool = super::super::connect_sqlite_path(&path).await?;
        let r = report("2026-09-08T10:00:00Z", 5)?;
        assert!(record_report(&pool, &principal("a"), "api", r.clone()).await?);
        assert!(!record_report(&pool, &principal("a"), "api", r.clone()).await?);
        assert!(
            record_report(
                &pool,
                &principal("a"),
                "api",
                report("2026-09-07T10:00:00Z", 1)?
            )
            .await?
        );
        assert!(read_reports(&pool, "b", "api", 100, 0).await?.is_empty());
        let mut unauthorized = principal("a");
        unauthorized.clerk_org_role = Some("org:viewer".into());
        assert!(
            record_report(&pool, &unauthorized, "api", r.clone())
                .await
                .is_err()
        );
        assert!(read_reports(&pool, "a", "api", 0, 0).await.is_err());
        drop(pool);
        let pool = super::super::connect_sqlite_path(&path).await?;
        assert_eq!(read_reports(&pool, "a", "api", 100, 0).await?.len(), 2);
        Ok(())
    }
}
