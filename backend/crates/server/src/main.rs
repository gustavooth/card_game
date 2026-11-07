mod config;
mod game;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use royale_auth::AuthService;
use royale_defs::Catalog;
use royale_net::{AppState, NetConfig, NetDeps};
use tokio::sync::mpsc::unbounded_channel;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::game::{World, WorldDeps, run_loop};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,royale_server=debug")),
        )
        .init();

    let config = Config::from_env()?;
    let loaded = royale_defs::load().context("falha ao carregar definições JSON")?;
    let defs = Arc::new(loaded.defs);
    let catalog: Catalog = loaded.catalog;
    let default_deck = royale_core::default_deck(&defs, &catalog);
    tracing::info!(
        cards = defs.cards.len(),
        troops = defs.troops.len(),
        spells = defs.spells.len(),
        buildings = defs.buildings.len(),
        "definições carregadas"
    );

    let db = connect_database(&config).await?;
    db.migrate().await.context("falha ao aplicar migrações")?;
    tracing::info!("banco pronto");

    let auth = AuthService::new(db.clone(), config.session_days);
    let (game_tx, game_rx) = unbounded_channel();

    let state = AppState::new(NetDeps {
        db: db.clone(),
        auth: auth.clone(),
        defs: defs.clone(),
        catalog: catalog.clone(),
        game_tx,
        default_deck: default_deck.clone(),
        config: NetConfig {
            cookie_secure: config.cookie_secure,
            session_days: config.session_days,
            allowed_origins: config.allowed_origins.clone(),
        },
    });

    let world = World::new(WorldDeps {
        state: state.clone(),
        defs: defs.clone(),
        catalog: catalog.clone(),
        default_deck,
    });
    tokio::spawn(run_loop(game_rx, world));

    let cleanup_auth = auth.clone();
    tokio::spawn(async move {
        loop {
            cleanup_auth.cleanup_sessions().await;
            tokio::time::sleep(Duration::from_secs(6 * 3600)).await;
        }
    });

    let app = royale_net::router(state);
    let listener = tokio::net::TcpListener::bind(config.bind)
        .await
        .with_context(|| format!("falha ao abrir {}", config.bind))?;
    tracing::info!(address = %config.bind, "servidor ouvindo");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("servidor terminou com erro")?;
    Ok(())
}

async fn connect_database(config: &Config) -> Result<royale_db::Db> {
    let mut attempt = 0;
    loop {
        attempt += 1;
        match royale_db::Db::connect(&config.database_url, config.db_pool_size) {
            Ok(db) => match db.ping().await {
                Ok(()) => return Ok(db),
                Err(error) => {
                    if attempt >= 30 {
                        return Err(error).context("PostgreSQL indisponível");
                    }
                    tracing::warn!(%error, attempt, "aguardando PostgreSQL");
                }
            },
            Err(error) => {
                if attempt >= 30 {
                    return Err(error).context("configuração do PostgreSQL inválida");
                }
                tracing::warn!(%error, attempt, "aguardando PostgreSQL");
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("falha ao instalar handler de Ctrl+C");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("falha ao instalar handler de SIGTERM")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("encerrando servidor");
}
