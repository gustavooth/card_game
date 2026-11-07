use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use royale_core::{Match, Phase};
use royale_defs::{Catalog, Definitions};
use royale_net::{AppState, ConnectionHub};
use royale_shared::Side;
use royale_shared::{
    ErrorMsg, GameCommand, MatchEndMsg, MatchFoundMsg, QueueMsg, REPLAY_FRAME_EVERY, TICK_NANOS,
    encode_message,
};
use tokio::sync::mpsc::UnboundedReceiver;
use uuid::Uuid;

use super::persist::{PersistJob, persist_match};

pub struct WorldDeps {
    pub state: AppState,
    pub defs: Arc<Definitions>,
    pub catalog: Catalog,
    pub default_deck: Vec<String>,
}

struct Queued {
    user_id: Uuid,
    username: String,
}

#[allow(dead_code)]
pub struct WorldUser {
    pub id: Uuid,
    pub username: String,
    pub online: bool,
    pub room: Option<String>,
}

struct Room {
    id: String,
    game: Match,
    users: [Uuid; 2],
    names: [String; 2],
    replay_frames: Vec<Vec<u8>>,
    last_snapshot: Option<Vec<u8>>,
    persisted: bool,
    ended_at: Option<Instant>,
}

pub struct World {
    state: AppState,
    defs: Arc<Definitions>,
    catalog: Catalog,
    default_deck: Vec<String>,
    queue: VecDeque<Queued>,
    rooms: HashMap<String, Room>,
    user_room: HashMap<Uuid, String>,
    users: HashMap<Uuid, WorldUser>,
}

pub async fn run_loop(mut receiver: UnboundedReceiver<GameCommand>, mut world: World) {
    let mut ticker = tokio::time::interval(Duration::from_nanos(TICK_NANOS));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            command = receiver.recv() => {
                match command {
                    Some(command) => world.handle(command).await,
                    None => break,
                }
            }
            _ = ticker.tick() => {
                world.tick();
            }
        }
    }
}

impl World {
    pub fn new(deps: WorldDeps) -> Self {
        Self {
            state: deps.state,
            defs: deps.defs,
            catalog: deps.catalog,
            default_deck: deps.default_deck,
            queue: VecDeque::new(),
            rooms: HashMap::new(),
            user_room: HashMap::new(),
            users: HashMap::new(),
        }
    }

    pub async fn handle(&mut self, command: GameCommand) {
        match command {
            GameCommand::JoinQueue { user_id, username } => {
                self.join_queue(user_id, username).await;
            }
            GameCommand::LeaveQueue { user_id } => {
                self.queue.retain(|entry| entry.user_id != user_id);
            }
            GameCommand::Deploy {
                user_id,
                slot,
                x,
                y,
            } => {
                self.deploy(user_id, slot, x, y);
            }
            GameCommand::Disconnect { user_id } => {
                self.queue.retain(|entry| entry.user_id != user_id);
                if let Some(user) = self.users.get_mut(&user_id) {
                    user.online = false;
                }
            }
            GameCommand::Reconnect { user_id } => {
                self.reconnect(user_id);
            }
        }
    }

    async fn join_queue(&mut self, user_id: Uuid, username: String) {
        self.users.insert(
            user_id,
            WorldUser {
                id: user_id,
                username: username.clone(),
                online: true,
                room: self.user_room.get(&user_id).cloned(),
            },
        );
        if let Some(room_id) = self.user_room.get(&user_id).cloned() {
            if let Some(room) = self.rooms.get(&room_id) {
                if room.game.phase != Phase::Ended {
                    self.send_match_found(&room_id, user_id, true);
                    return;
                }
            }
        }
        if self.queue.iter().any(|entry| entry.user_id == user_id) {
            let position = self
                .queue
                .iter()
                .position(|entry| entry.user_id == user_id)
                .unwrap_or(0) as u32
                + 1;
            self.send_queue(user_id, position);
            return;
        }
        if !self.state.hub().is_online(user_id) {
            return;
        }
        self.queue.push_back(Queued { user_id, username });
        let position = self.queue.len() as u32;
        self.send_queue(user_id, position);
        self.try_match().await;
    }

    async fn try_match(&mut self) {
        while self.queue.len() >= 2 {
            let first = self.queue.pop_front().expect("fila verificada");
            let second = match self.queue.pop_front() {
                Some(second) => second,
                None => {
                    self.queue.push_front(first);
                    break;
                }
            };
            if first.user_id == second.user_id {
                self.queue.push_back(second);
                continue;
            }
            if !self.state.hub().is_online(first.user_id)
                || !self.state.hub().is_online(second.user_id)
            {
                if self.state.hub().is_online(first.user_id) {
                    self.queue.push_front(first);
                }
                if self.state.hub().is_online(second.user_id) {
                    self.queue.push_front(second);
                }
                break;
            }
            let deck_first = self.load_deck(first.user_id).await;
            let deck_second = self.load_deck(second.user_id).await;
            let room_id = Uuid::new_v4().to_string();
            let seed = Uuid::new_v4().as_u128() as u64;
            let players = [
                (first.user_id, first.username.clone(), deck_first),
                (second.user_id, second.username.clone(), deck_second),
            ];
            let Some(game) = Match::new(
                self.defs.clone(),
                self.catalog.clone(),
                room_id.clone(),
                "classic",
                players,
                seed,
                Utc::now().timestamp(),
            ) else {
                tracing::error!("falha ao criar partida");
                continue;
            };
            let room = Room {
                id: room_id.clone(),
                game,
                users: [first.user_id, second.user_id],
                names: [first.username.clone(), second.username.clone()],
                replay_frames: Vec::new(),
                last_snapshot: None,
                persisted: false,
                ended_at: None,
            };
            self.rooms.insert(room_id.clone(), room);
            self.user_room.insert(first.user_id, room_id.clone());
            self.user_room.insert(second.user_id, room_id.clone());
            for user_id in [first.user_id, second.user_id] {
                if let Some(user) = self.users.get_mut(&user_id) {
                    user.online = true;
                    user.room = Some(room_id.clone());
                }
            }
            tracing::info!(
                room = %room_id,
                blue = %first.username,
                red = %second.username,
                "partida iniciada"
            );
            self.send_match_found(&room_id, first.user_id, false);
            self.send_match_found(&room_id, second.user_id, false);
        }
    }

    async fn load_deck(&self, user_id: Uuid) -> Vec<u16> {
        let stored = self
            .state
            .db()
            .get_deck(user_id)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| self.default_deck.clone());
        royale_core::deck_indices(&self.defs, &self.catalog, &stored)
            .or_else(|_| royale_core::deck_indices(&self.defs, &self.catalog, &self.default_deck))
            .unwrap_or_default()
    }

    fn send_queue(&self, user_id: Uuid, position: u32) {
        let payload = encode_message(&QueueMsg {
            tag: "queue",
            position,
        });
        self.state.hub().send_user(user_id, payload);
    }

    fn send_match_found(&self, room_id: &str, user_id: Uuid, reconnect: bool) {
        let Some(room) = self.rooms.get(room_id) else {
            return;
        };
        let index = if room.users[0] == user_id { 0 } else { 1 };
        let opponent = room.names[1 - index].clone();
        let side = Side::from_index(index as u8);
        let payload = encode_message(&MatchFoundMsg {
            tag: "match",
            room: room.id.clone(),
            side: side as u8,
            opponent,
            arena: room.game.arena.slug.clone(),
            reconnect,
        });
        self.state.hub().send_user(user_id, payload);
        if let Some(snapshot) = &room.last_snapshot {
            self.state.hub().send_user(user_id, snapshot.clone());
        }
    }

    fn reconnect(&mut self, user_id: Uuid) {
        if let Some(user) = self.users.get_mut(&user_id) {
            user.online = true;
        }
        let Some(room_id) = self.user_room.get(&user_id).cloned() else {
            return;
        };
        self.send_match_found(&room_id, user_id, true);
    }

    fn deploy(&mut self, user_id: Uuid, slot: u8, x: f32, y: f32) {
        let Some(room_id) = self.user_room.get(&user_id).cloned() else {
            self.send_error(user_id, "not_in_match", "Você não está em uma partida");
            return;
        };
        let Some(room) = self.rooms.get_mut(&room_id) else {
            return;
        };
        if let Err(error) = room.game.deploy(user_id, slot, x, y) {
            let message = match error {
                royale_core::DeployError::NotEnoughElixir => "Elixir insuficiente",
                royale_core::DeployError::InvalidPlacement => "Posicionamento inválido",
                royale_core::DeployError::InvalidSlot => "Carta inválida",
                royale_core::DeployError::MatchEnded => "Partida encerrada",
                royale_core::DeployError::NotInMatch => "Você não está na partida",
            };
            self.send_error(user_id, error.code(), message);
        }
    }

    fn send_error(&self, user_id: Uuid, code: &'static str, message: &str) {
        let payload = encode_message(&ErrorMsg {
            tag: "err",
            code,
            message: message.to_string(),
        });
        self.state.hub().send_user(user_id, payload);
    }

    pub fn tick(&mut self) {
        let ids: Vec<String> = self.rooms.keys().cloned().collect();
        let mut jobs: Vec<PersistJob> = Vec::new();
        let hub = self.state.hub().clone();

        for id in ids {
            let Some(room) = self.rooms.get_mut(&id) else {
                continue;
            };
            if room.game.phase == Phase::Ended {
                let expired = room
                    .ended_at
                    .is_some_and(|ended| ended.elapsed() > Duration::from_secs(30));
                if expired {
                    self.user_room.remove(&room.users[0]);
                    self.user_room.remove(&room.users[1]);
                    for user_id in room.users {
                        if let Some(user) = self.users.get_mut(&user_id) {
                            user.room = None;
                        }
                    }
                    self.rooms.remove(&id);
                }
                continue;
            }

            room.game.tick();
            let snapshot = room.game.snapshot();
            let payload = encode_message(&snapshot);
            room.last_snapshot = Some(payload.clone());
            if room.game.tick % REPLAY_FRAME_EVERY == 0 {
                room.replay_frames.push(payload.clone());
            }
            for user in room.users {
                hub.send_user(user, payload.clone());
            }

            if room.game.phase == Phase::Ended {
                room.ended_at = Some(Instant::now());
                finish_room(&hub, room, &mut jobs);
            }
        }

        for job in jobs {
            let state = self.state.clone();
            tokio::spawn(async move {
                persist_match(state, job).await;
            });
        }
    }
}

fn finish_room(hub: &ConnectionHub, room: &mut Room, jobs: &mut Vec<PersistJob>) {
    let result = room.game.result();
    for (index, user) in room.users.iter().enumerate() {
        let side = Side::from_index(index as u8);
        let outcome = match result.winner {
            None => "draw",
            Some(winner) if winner == side => "win",
            Some(_) => "loss",
        };
        let message = MatchEndMsg {
            tag: "end",
            result: outcome.to_string(),
            winner: result.winner.map(|side| side as u8),
            crowns: result.crowns,
            damage: result.damage,
            duration_ms: result.duration_ms,
            reason: result.end_reason.as_str().to_string(),
            replay_id: None,
        };
        hub.send_user(*user, encode_message(&message));
    }
    if !room.persisted {
        room.persisted = true;
        let frames = std::mem::take(&mut room.replay_frames);
        let match_id = Uuid::parse_str(&room.id).unwrap_or_else(|_| Uuid::new_v4());
        let winner_id = result.winner.map(|side| room.users[side.index()]);
        let blue_won = result.winner == Some(Side::Blue);
        let red_won = result.winner == Some(Side::Red);
        let row = royale_db::models::NewMatch {
            id: match_id,
            player_blue: room.users[0],
            player_red: room.users[1],
            winner: winner_id,
            result_blue: if blue_won {
                "win".into()
            } else if red_won {
                "loss".into()
            } else {
                "draw".into()
            },
            result_red: if red_won {
                "win".into()
            } else if blue_won {
                "loss".into()
            } else {
                "draw".into()
            },
            duration_ms: result.duration_ms as i32,
            blue_crowns: result.crowns[0] as i16,
            red_crowns: result.crowns[1] as i16,
            blue_damage: result.damage[0],
            red_damage: result.damage[1],
            end_reason: result.end_reason.as_str().to_string(),
            started_at: chrono::DateTime::<chrono::Utc>::from_timestamp(
                room.game.started_at_epoch,
                0,
            )
            .unwrap_or_else(chrono::Utc::now),
            ended_at: chrono::Utc::now(),
        };
        let meta = [0usize, 1usize].map(|index| {
            let side = Side::from_index(index as u8);
            let outcome = match result.winner {
                None => "draw",
                Some(winner) if winner == side => "win",
                _ => "loss",
            };
            serde_json::json!({
                "side": index,
                "arena": room.game.arena.slug,
                "blue": room.names[0],
                "red": room.names[1],
                "started_at": room.game.started_at_epoch,
                "opponent": room.names[1 - index],
                "result": outcome,
                "crowns": [result.crowns[0], result.crowns[1]],
            })
            .to_string()
        });
        jobs.push(PersistJob {
            row,
            users: room.users,
            frames,
            meta,
        });
        tracing::info!(room = %room.id, winner = ?result.winner, "partida encerrada");
    }
}

#[allow(dead_code)]
impl Room {
    pub fn troops(&self) -> Vec<royale_core::TroopState> {
        self.game.troops()
    }

    pub fn players(&self) -> &[royale_core::Player] {
        &self.game.players
    }

    pub fn time(&self) -> u32 {
        self.game.time()
    }
}
