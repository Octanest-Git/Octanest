//! Diagnostic write/read on `instances` (D-09–D-13) — single-row upsert on `id = 1`.

use octanest_core::DbProbeResponse;
use sqlx::Row;

use crate::dialect::Dialect;
use crate::pool::DbPool;

pub async fn probe(pool: &DbPool, dialect: Dialect) -> Result<DbProbeResponse, String> {
    match pool {
        DbPool::Postgres(p) => {
            sqlx::query(
                "INSERT INTO instances (id, dialect, probe_count, probed_at)
VALUES (1, $1, 1, now())
ON CONFLICT (id) DO UPDATE
SET probe_count = instances.probe_count + 1,
    dialect = EXCLUDED.dialect,
    probed_at = now()",
            )
            .bind(dialect.as_str())
            .execute(p)
            .await
            .map_err(|e| format!("db probe failed: {e}"))?;

            let row = sqlx::query(
                "SELECT dialect,
       probe_count,
       to_char(probed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS probed_at
FROM instances WHERE id = 1",
            )
            .fetch_one(p)
            .await
            .map_err(|e| format!("db probe failed: {e}"))?;

            Ok(DbProbeResponse {
                dialect: row
                    .try_get("dialect")
                    .map_err(|e| format!("db probe failed: {e}"))?,
                probe_count: row
                    .try_get("probe_count")
                    .map_err(|e| format!("db probe failed: {e}"))?,
                probed_at: row
                    .try_get("probed_at")
                    .map_err(|e| format!("db probe failed: {e}"))?,
            })
        }
        DbPool::MySql(p) => {
            sqlx::query(
                "INSERT INTO instances (id, dialect, probe_count, probed_at)
VALUES (1, ?, 1, NOW()) AS new
ON DUPLICATE KEY UPDATE
  probe_count = instances.probe_count + 1,
  dialect = new.dialect,
  probed_at = NOW()",
            )
            .bind(dialect.as_str())
            .execute(p)
            .await
            .map_err(|e| format!("db probe failed: {e}"))?;

            let row = sqlx::query(
                "SELECT dialect,
       probe_count,
       DATE_FORMAT(probed_at, '%Y-%m-%dT%H:%i:%sZ') AS probed_at
FROM instances WHERE id = 1",
            )
            .fetch_one(p)
            .await
            .map_err(|e| format!("db probe failed: {e}"))?;

            Ok(DbProbeResponse {
                dialect: row
                    .try_get("dialect")
                    .map_err(|e| format!("db probe failed: {e}"))?,
                probe_count: row
                    .try_get("probe_count")
                    .map_err(|e| format!("db probe failed: {e}"))?,
                probed_at: row
                    .try_get("probed_at")
                    .map_err(|e| format!("db probe failed: {e}"))?,
            })
        }
        DbPool::Sqlite(p) => {
            sqlx::query(
                "INSERT INTO instances (id, dialect, probe_count, probed_at)
VALUES (1, ?1, 1, strftime('%Y-%m-%d %H:%M:%S','now'))
ON CONFLICT(id) DO UPDATE SET
  probe_count = instances.probe_count + 1,
  dialect = excluded.dialect,
  probed_at = strftime('%Y-%m-%d %H:%M:%S','now')",
            )
            .bind(dialect.as_str())
            .execute(p)
            .await
            .map_err(|e| format!("db probe failed: {e}"))?;

            let row = sqlx::query(
                "SELECT dialect,
       probe_count,
       strftime('%Y-%m-%dT%H:%M:%SZ', probed_at) AS probed_at
FROM instances WHERE id = 1",
            )
            .fetch_one(p)
            .await
            .map_err(|e| format!("db probe failed: {e}"))?;

            Ok(DbProbeResponse {
                dialect: row
                    .try_get("dialect")
                    .map_err(|e| format!("db probe failed: {e}"))?,
                probe_count: row
                    .try_get("probe_count")
                    .map_err(|e| format!("db probe failed: {e}"))?,
                probed_at: row
                    .try_get("probed_at")
                    .map_err(|e| format!("db probe failed: {e}"))?,
            })
        }
    }
}
