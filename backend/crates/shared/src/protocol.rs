use serde::Serialize;
use uuid::Uuid;

/// Comandos enviados do WebSocket para o loop autoritativo.
#[derive(Debug, Clone)]
pub enum GameCommand {
    JoinQueue {
        user_id: Uuid,
        username: String,
    },
    LeaveQueue {
        user_id: Uuid,
    },
    Deploy {
        user_id: Uuid,
        slot: u8,
        x: f32,
        y: f32,
    },
    Disconnect {
        user_id: Uuid,
    },
    Reconnect {
        user_id: Uuid,
    },
}

/// Snapshot completo da arena enviado a 30 Hz.
#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    #[serde(rename = "t")]
    pub tag: &'static str,
    #[serde(rename = "k")]
    pub tick: u32,
    #[serde(rename = "m")]
    pub time_ms: u32,
    #[serde(rename = "ph")]
    pub phase: u8,
    #[serde(rename = "ot")]
    pub overtime: bool,
    #[serde(rename = "p")]
    pub players: Vec<PlayerSnapshot>,
    #[serde(rename = "e")]
    pub entities: Vec<Vec<f32>>,
    #[serde(rename = "ev")]
    pub events: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerSnapshot {
    #[serde(rename = "s")]
    pub side: u8,
    #[serde(rename = "e")]
    pub elixir: f32,
    #[serde(rename = "c")]
    pub crowns: u8,
    #[serde(rename = "d")]
    pub tower_damage: f32,
    #[serde(rename = "h")]
    pub hand: Vec<[f32; 2]>,
    #[serde(rename = "n")]
    pub next_card: [f32; 2],
}

/// Layout fixo dos vetores de entidade no snapshot.
pub mod entity_index {
    pub const ID: usize = 0;
    pub const KIND: usize = 1;
    pub const SIDE: usize = 2;
    pub const TYPE_INDEX: usize = 3;
    pub const HP: usize = 4;
    pub const MAX_HP: usize = 5;
    pub const X: usize = 6;
    pub const Y: usize = 7;
    pub const RADIUS: usize = 8;
    pub const FLYING: usize = 9;
    pub const SHIELD: usize = 10;
    pub const ANIM: usize = 11;
    pub const FACING: usize = 12;
    pub const LIFE_MS: usize = 13;
    pub const ACTIVE: usize = 14;
    pub const EFFECT_COUNT: usize = 15;
    pub const BASE_LEN: usize = 16;
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueMsg {
    #[serde(rename = "t")]
    pub tag: &'static str,
    #[serde(rename = "p")]
    pub position: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchFoundMsg {
    #[serde(rename = "t")]
    pub tag: &'static str,
    #[serde(rename = "r")]
    pub room: String,
    #[serde(rename = "s")]
    pub side: u8,
    #[serde(rename = "o")]
    pub opponent: String,
    #[serde(rename = "a")]
    pub arena: String,
    #[serde(rename = "rc")]
    pub reconnect: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchEndMsg {
    #[serde(rename = "t")]
    pub tag: &'static str,
    #[serde(rename = "res")]
    pub result: String,
    #[serde(rename = "w")]
    pub winner: Option<u8>,
    #[serde(rename = "c")]
    pub crowns: [u8; 2],
    #[serde(rename = "d")]
    pub damage: [f32; 2],
    #[serde(rename = "m")]
    pub duration_ms: u32,
    #[serde(rename = "rs")]
    pub reason: String,
    #[serde(rename = "rep")]
    pub replay_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMsg {
    #[serde(rename = "t")]
    pub tag: &'static str,
    #[serde(rename = "u")]
    pub user: String,
    #[serde(rename = "m")]
    pub text: String,
    #[serde(rename = "ts")]
    pub ts: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorMsg {
    #[serde(rename = "t")]
    pub tag: &'static str,
    #[serde(rename = "c")]
    pub code: &'static str,
    #[serde(rename = "m")]
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PongMsg {
    #[serde(rename = "t")]
    pub tag: &'static str,
    #[serde(rename = "ts")]
    pub ts: u64,
}

/// Codifica uma mensagem de controle/rede como MessagePack nomeado.
pub fn encode_message<T: Serialize>(value: &T) -> Vec<u8> {
    rmp_serde::to_vec_named(value).unwrap_or_default()
}

/// Cabeçalho + metadados + frames de replay (antes do gzip).
pub const REPLAY_MAGIC: &[u8; 4] = b"RYL2";

pub fn encode_replay(frame_interval: u32, metadata: &[u8], frames: &[Vec<u8>]) -> Vec<u8> {
    let mut out =
        Vec::with_capacity(frames.iter().map(|f| f.len() + 4).sum::<usize>() + metadata.len() + 24);
    out.extend_from_slice(REPLAY_MAGIC);
    out.extend_from_slice(&(frame_interval as u16).to_le_bytes());
    out.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
    out.extend_from_slice(metadata);
    out.extend_from_slice(&(frames.len() as u32).to_le_bytes());
    for frame in frames {
        out.extend_from_slice(&(frame.len() as u32).to_le_bytes());
        out.extend_from_slice(frame);
    }
    out
}
