use std::io::Write;

use flate2::Compression;
use flate2::write::GzEncoder;
use royale_db::models::NewMatch;
use royale_net::AppState;
use royale_shared::{REPLAY_FRAME_EVERY, encode_replay};
use uuid::Uuid;

pub struct PersistJob {
    pub row: NewMatch,
    pub users: [Uuid; 2],
    pub frames: Vec<Vec<u8>>,
    pub meta: [String; 2],
}

pub async fn persist_match(state: AppState, job: PersistJob) {
    if let Err(error) = state.db().insert_match(&job.row).await {
        tracing::error!(%error, "falha ao gravar partida");
        return;
    }
    for (index, user) in job.users.iter().enumerate() {
        let raw = encode_replay(REPLAY_FRAME_EVERY, job.meta[index].as_bytes(), &job.frames);
        let compressed = match tokio::task::spawn_blocking(move || gzip(&raw)).await {
            Ok(Ok(data)) => data,
            Ok(Err(error)) => {
                tracing::error!(%error, "falha ao comprimir replay");
                continue;
            }
            Err(error) => {
                tracing::error!(%error, "falha ao comprimir replay");
                continue;
            }
        };
        if let Err(error) = state
            .db()
            .insert_replay(*user, job.row.id, &compressed)
            .await
        {
            tracing::error!(%error, user = %user, "falha ao gravar replay");
        }
    }
}

fn gzip(raw: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(raw)?;
    encoder.finish()
}
