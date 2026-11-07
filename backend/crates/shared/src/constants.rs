/// Frequência autoritativa da simulação e da rede.
pub const TICK_HZ: u32 = 30;
/// Duração de um tick em nanossegundos (30 Hz exato).
pub const TICK_NANOS: u64 = 33_333_333;
/// Duração de um tick em milissegundos (arredondada).
pub const TICK_MS: u32 = 33;

/// Arena clássica.
pub const ARENA_WIDTH: f32 = 18.0;
pub const ARENA_HEIGHT: f32 = 32.0;
pub const RIVER_TOP: f32 = 15.5;
pub const RIVER_BOTTOM: f32 = 16.5;

/// Regras de partida.
pub const REGULATION_MS: u32 = 180_000;
pub const OVERTIME_MS: u32 = 120_000;
pub const MAX_ELIXIR: f32 = 10.0;
pub const ELIXIR_REGEN_MS: u32 = 2_800;
pub const HAND_SIZE: usize = 4;
pub const DECK_SIZE: usize = 8;

/// Rede.
pub const CONTENT_VERSION: u32 = 1;
pub const CHAT_MAX_CHARS: usize = 200;
pub const SESSION_COOKIE: &str = "royale_session";
pub const SESSION_DAYS: i64 = 30;
pub const REPLAY_FRAME_EVERY: u32 = 3;
pub const MAX_REPLAYS_PER_USER: i64 = 10;

/// Efeitos e animações (índices estáveis usados no snapshot).
pub const ANIM_IDLE: u8 = 0;
pub const ANIM_WALK: u8 = 1;
pub const ANIM_ATTACK: u8 = 2;
pub const ANIM_BURROW: u8 = 3;
pub const ANIM_DEAD: u8 = 4;

pub const EVENT_DEPLOY: u8 = 1;
pub const EVENT_HIT: u8 = 2;
pub const EVENT_DEATH: u8 = 3;
pub const EVENT_SPELL: u8 = 4;
pub const EVENT_TOWER_DOWN: u8 = 5;
pub const EVENT_KING_ACTIVE: u8 = 6;
pub const EVENT_MATCH_END: u8 = 7;
pub const EVENT_ELIXIR: u8 = 8;

pub const PHASE_REGULATION: u8 = 0;
pub const PHASE_OVERTIME: u8 = 1;
pub const PHASE_ENDED: u8 = 2;

/// Tipos de entidade no snapshot.
pub const ENTITY_TROOP: u8 = 0;
pub const ENTITY_BUILDING: u8 = 1;
pub const ENTITY_TOWER: u8 = 2;
pub const ENTITY_PROJECTILE: u8 = 3;
pub const ENTITY_ZONE: u8 = 4;
