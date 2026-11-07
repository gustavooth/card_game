use deadpool_postgres::Pool;

use crate::DbError;

/// Migrações embutidas, executadas em ordem na inicialização.
const MIGRATIONS: [(&str, &str); 2] = [
    ("0001_init", include_str!("../migrations/0001_init.sql")),
    (
        "0002_match_indexes",
        include_str!("../migrations/0002_match_indexes.sql"),
    ),
];

pub async fn run(pool: &Pool) -> Result<(), DbError> {
    let mut client = pool.get().await?;
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                 version TEXT PRIMARY KEY,
                 applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
             )",
        )
        .await?;

    for (version, sql) in MIGRATIONS {
        let applied = client
            .query_opt(
                "SELECT 1 FROM schema_migrations WHERE version = $1",
                &[&version],
            )
            .await?
            .is_some();
        if applied {
            continue;
        }
        let transaction = client.transaction().await?;
        transaction.batch_execute(sql).await?;
        transaction
            .execute(
                "INSERT INTO schema_migrations (version) VALUES ($1)",
                &[&version],
            )
            .await?;
        transaction.commit().await?;
        tracing::info!(migration = version, "migração aplicada");
    }
    Ok(())
}
