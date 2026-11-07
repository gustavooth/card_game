//! Persistência em PostgreSQL. Todo o SQL do projeto vive neste crate.

mod migrations;
pub mod models;
pub mod queries;

use deadpool_postgres::{Manager, ManagerConfig, Pool, Runtime};
use thiserror::Error;
use tokio_postgres::NoTls;

#[derive(Clone)]
pub struct Db {
    pub pool: Pool,
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("configuração do banco: {0}")]
    Config(String),
    #[error("pool do banco: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("erro do PostgreSQL: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("conflito: {0}")]
    Conflict(&'static str),
    #[error("registro não encontrado")]
    NotFound,
}

impl Db {
    pub fn connect(url: &str, max_size: usize) -> Result<Self, DbError> {
        let pg_config: tokio_postgres::Config = url
            .parse()
            .map_err(|error: tokio_postgres::Error| DbError::Config(error.to_string()))?;
        let manager = Manager::from_config(
            pg_config,
            NoTls,
            ManagerConfig {
                recycling_method: deadpool_postgres::RecyclingMethod::Fast,
            },
        );
        let pool = Pool::builder(manager)
            .max_size(max_size)
            .runtime(Runtime::Tokio1)
            .build()
            .map_err(|error| DbError::Config(error.to_string()))?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), DbError> {
        migrations::run(&self.pool).await
    }

    pub async fn ping(&self) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.simple_query("SELECT 1").await?;
        Ok(())
    }
}
