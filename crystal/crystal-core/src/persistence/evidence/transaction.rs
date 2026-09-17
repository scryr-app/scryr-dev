//! Atomic immutable delivery and derived provider projections across both backends.
use super::DatabasePool;

pub(super) enum Transaction<'a> {
    Sqlite(sqlx::Transaction<'a, sqlx::Sqlite>),
    Turso(libsql::Transaction),
}
impl<'a> Transaction<'a> {
    pub(super) async fn begin(pool: &'a DatabasePool) -> Result<Self, String> {
        match pool {
            DatabasePool::Sqlite(pool) => pool
                .begin_with("BEGIN IMMEDIATE")
                .await
                .map(Self::Sqlite)
                .map_err(|e| e.to_string()),
            DatabasePool::Turso(database) => database
                .connect()
                .map_err(|e| e.to_string())?
                .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
                .await
                .map(Self::Turso)
                .map_err(|e| e.to_string()),
        }
    }
    pub(super) async fn execute(&mut self, sql: &str, params: Vec<String>) -> Result<u64, String> {
        match self {
            Self::Sqlite(tx) => {
                let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.to_owned()));
                for parameter in params {
                    query = query.bind(parameter);
                }
                query
                    .execute(&mut **tx)
                    .await
                    .map(|r| r.rows_affected())
                    .map_err(|e| e.to_string())
            }
            Self::Turso(tx) => tx
                .execute(
                    sql,
                    params
                        .into_iter()
                        .map(libsql::Value::Text)
                        .collect::<Vec<_>>(),
                )
                .await
                .map_err(|e| e.to_string()),
        }
    }
    pub(super) async fn query(
        &mut self,
        sql: &str,
        params: Vec<String>,
    ) -> Result<Vec<String>, String> {
        match self {
            Self::Sqlite(tx) => {
                let mut query = sqlx::query_scalar(sqlx::AssertSqlSafe(sql.to_owned()));
                for parameter in params {
                    query = query.bind(parameter);
                }
                query.fetch_all(&mut **tx).await.map_err(|e| e.to_string())
            }
            Self::Turso(tx) => {
                let mut rows = tx
                    .query(
                        sql,
                        params
                            .into_iter()
                            .map(libsql::Value::Text)
                            .collect::<Vec<_>>(),
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                let mut result = Vec::new();
                while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
                    result.push(row.get(0).map_err(|e| e.to_string())?);
                }
                Ok(result)
            }
        }
    }
    pub(super) async fn commit(self) -> Result<(), String> {
        match self {
            Self::Sqlite(tx) => tx.commit().await.map_err(|e| e.to_string()),
            Self::Turso(tx) => tx.commit().await.map_err(|e| e.to_string()),
        }
    }
}
