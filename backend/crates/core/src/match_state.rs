use std::collections::VecDeque;
use std::sync::Arc;

use royale_defs::{CardType, Catalog, Definitions};
use royale_shared::{EVENT_DEPLOY, MAX_ELIXIR, OVERTIME_MS, REGULATION_MS, Side, TICK_MS};
use uuid::Uuid;

use crate::rng::Rng;
use crate::types::*;
use crate::util::{MS_PER_TICK, distance};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployError {
    NotInMatch,
    MatchEnded,
    InvalidSlot,
    NotEnoughElixir,
    InvalidPlacement,
}

impl DeployError {
    pub fn code(self) -> &'static str {
        match self {
            DeployError::NotInMatch => "not_in_match",
            DeployError::MatchEnded => "match_ended",
            DeployError::InvalidSlot => "invalid_slot",
            DeployError::NotEnoughElixir => "not_enough_elixir",
            DeployError::InvalidPlacement => "invalid_placement",
        }
    }
}

pub struct Match {
    pub room_id: String,
    pub arena: royale_defs::ArenaDef,
    pub defs: Arc<Definitions>,
    pub catalog: Catalog,
    pub players: [Player; 2],
    pub entities: Vec<Entity>,
    pub projectiles: Vec<Projectile>,
    pub zones: Vec<Zone>,
    pub rollers: Vec<RollingSpell>,
    pub pending: Vec<PendingAction>,
    pub dead: Vec<u32>,
    pub events: Vec<Event>,
    pub tick: u32,
    pub elapsed_ms: u32,
    pub phase: Phase,
    pub winner: Option<Side>,
    pub end_reason: Option<EndReason>,
    pub next_id: u32,
    pub rng: Rng,
    pub started_at_epoch: i64,
}

impl Match {
    pub fn new(
        defs: Arc<Definitions>,
        catalog: Catalog,
        room_id: String,
        arena_slug: &str,
        players: [(Uuid, String, Vec<u16>); 2],
        seed: u64,
        started_at_epoch: i64,
    ) -> Option<Self> {
        let arena = defs.arenas.get(arena_slug)?.clone();
        let blue = Player::new(
            players[0].0,
            players[0].1.clone(),
            Side::Blue,
            players[0].2.clone(),
        )?;
        let red = Player::new(
            players[1].0,
            players[1].1.clone(),
            Side::Red,
            players[1].2.clone(),
        )?;
        let mut game = Self {
            room_id,
            arena,
            defs,
            catalog,
            players: [blue, red],
            entities: Vec::new(),
            projectiles: Vec::new(),
            zones: Vec::new(),
            rollers: Vec::new(),
            pending: Vec::new(),
            dead: Vec::new(),
            events: Vec::new(),
            tick: 0,
            elapsed_ms: 0,
            phase: Phase::Regulation,
            winner: None,
            end_reason: None,
            next_id: 0,
            rng: Rng::new(seed),
            started_at_epoch,
        };
        game.spawn_towers();
        Some(game)
    }

    pub fn player_index(&self, user_id: Uuid) -> Option<usize> {
        self.players
            .iter()
            .position(|player| player.user_id == user_id)
    }

    pub fn side_of(&self, user_id: Uuid) -> Option<Side> {
        self.player_index(user_id)
            .map(|index| self.players[index].side)
    }

    pub fn tick(&mut self) {
        if self.phase == Phase::Ended {
            return;
        }
        self.tick += 1;
        self.elapsed_ms = (self.tick as u64 * 1000 / royale_shared::TICK_HZ as u64) as u32;
        self.events.clear();

        self.process_pending();
        self.regen_elixir();
        self.tick_effects();
        self.update_skills();
        self.decrement_timers();
        self.refresh_targets();
        self.update_jumps();
        self.update_dashes();
        self.update_movement();
        self.update_combat();
        self.update_spells();
        self.apply_environment_constraints();
        self.expire_lifetime();
        self.process_deaths();
        self.update_phase();
        self.reset_attack_anim();
    }

    fn regen_elixir(&mut self) {
        let mult = if self.phase == Phase::Overtime {
            2.0
        } else {
            1.0
        };
        let gain = (MS_PER_TICK / royale_shared::ELIXIR_REGEN_MS as f32) * mult;
        for player in &mut self.players {
            player.elixir = (player.elixir + gain).min(MAX_ELIXIR);
        }
    }

    fn expire_lifetime(&mut self) {
        let expired: Vec<u32> = self
            .entities
            .iter()
            .filter(|entity| !entity.is_tower && entity.life_ms == 0 && !entity.is_dead())
            .map(|entity| entity.id)
            .collect();
        for id in expired {
            self.mark_dead(id);
        }
    }

    fn update_phase(&mut self) {
        if self.phase == Phase::Ended {
            return;
        }
        match self.phase {
            Phase::Regulation => {
                if self.elapsed_ms >= REGULATION_MS {
                    let blue = self.players[0].crowns;
                    let red = self.players[1].crowns;
                    match blue.cmp(&red) {
                        std::cmp::Ordering::Greater => {
                            self.end_match(Some(Side::Blue), EndReason::RegulationCrowns)
                        }
                        std::cmp::Ordering::Less => {
                            self.end_match(Some(Side::Red), EndReason::RegulationCrowns)
                        }
                        std::cmp::Ordering::Equal => {
                            self.phase = Phase::Overtime;
                        }
                    }
                }
            }
            Phase::Overtime => {
                if self.elapsed_ms >= REGULATION_MS + OVERTIME_MS {
                    let blue_crowns = self.players[0].crowns;
                    let red_crowns = self.players[1].crowns;
                    let winner = match blue_crowns.cmp(&red_crowns) {
                        std::cmp::Ordering::Greater => Some(Side::Blue),
                        std::cmp::Ordering::Less => Some(Side::Red),
                        std::cmp::Ordering::Equal => {
                            let blue_damage = self.players[0].tower_damage;
                            let red_damage = self.players[1].tower_damage;
                            if (blue_damage - red_damage).abs() < 1.0 {
                                None
                            } else if blue_damage > red_damage {
                                Some(Side::Blue)
                            } else {
                                Some(Side::Red)
                            }
                        }
                    };
                    let reason = if winner.is_some() {
                        EndReason::OvertimeTiebreak
                    } else {
                        EndReason::Draw
                    };
                    self.end_match(winner, reason);
                }
            }
            Phase::Ended => {}
        }
    }

    pub fn deploy(&mut self, user_id: Uuid, slot: u8, x: f32, y: f32) -> Result<(), DeployError> {
        if self.phase == Phase::Ended {
            return Err(DeployError::MatchEnded);
        }
        let Some(player_index) = self.player_index(user_id) else {
            return Err(DeployError::NotInMatch);
        };
        let slot = slot as usize;
        let Some(card_index) = self.players[player_index].hand.get(slot).copied() else {
            return Err(DeployError::InvalidSlot);
        };
        let Some(card) = self.defs.cards.get(card_index as usize).cloned() else {
            return Err(DeployError::InvalidSlot);
        };
        if self.players[player_index].elixir + 0.0001 < card.elixir_cost as f32 {
            return Err(DeployError::NotEnoughElixir);
        }
        if !self.valid_placement(player_index, &card, x, y) {
            return Err(DeployError::InvalidPlacement);
        }
        let side = self.players[player_index].side;
        self.players[player_index].elixir -= card.elixir_cost as f32;
        self.players[player_index].cycle_card(slot, card_index);
        self.emit(EVENT_DEPLOY, side, x, y, card_index);

        let at_tick = self.tick + (card.summon_time_ms / TICK_MS).max(0);
        match card.card_type.clone() {
            CardType::Troop { troop, count } => {
                let positions = crate::spawn::formation_positions(count, x, y);
                for (px, py) in positions {
                    let (nx, ny) = self.clamp_spawn(side, px, py);
                    self.pending.push(PendingAction {
                        at_tick,
                        side,
                        kind: PendingKind::SpawnTroop {
                            card_slug: card.slug.clone(),
                            troop_slug: troop.clone(),
                            x: nx,
                            y: ny,
                            health: None,
                            no_rebirth: false,
                        },
                    });
                }
            }
            CardType::Building { building } => {
                self.pending.push(PendingAction {
                    at_tick,
                    side,
                    kind: PendingKind::SpawnBuilding {
                        card_slug: card.slug.clone(),
                        building_slug: building,
                        x,
                        y,
                        health: None,
                    },
                });
            }
            CardType::Spell { spell } => {
                let delay = self
                    .defs
                    .spells
                    .get(&spell)
                    .map(|spell| spell.delay_ms)
                    .unwrap_or(0);
                self.pending.push(PendingAction {
                    at_tick: at_tick + (delay / TICK_MS) + 1,
                    side,
                    kind: PendingKind::CastSpell {
                        card_slug: card.slug.clone(),
                        spell_slug: spell,
                        x,
                        y,
                    },
                });
            }
        }
        Ok(())
    }

    fn valid_placement(
        &self,
        player_index: usize,
        card: &royale_defs::CardDef,
        x: f32,
        y: f32,
    ) -> bool {
        if x < 0.3 || x > self.arena.width - 0.3 || y < 0.3 || y > self.arena.height - 0.3 {
            return false;
        }
        let side = self.players[player_index].side;
        let anywhere = self.card_deploys_anywhere(card);
        if matches!(card.card_type, CardType::Spell { .. }) || anywhere {
            return true;
        }
        let own_half = match side {
            Side::Blue => y >= self.arena.river_bottom + 0.1,
            Side::Red => y <= self.arena.river_top - 0.1,
        };
        if !own_half {
            let lane = if x < self.arena.width * 0.5 { 0 } else { 1 };
            if !self.enemy_princess_alive(side, lane) {
                let limit_ok = match side {
                    Side::Blue => y >= 5.4,
                    Side::Red => y <= self.arena.height - 5.4,
                };
                if !limit_ok {
                    return false;
                }
            } else {
                return false;
            }
        }
        if y > self.arena.river_top && y < self.arena.river_bottom {
            return false;
        }
        let spawn_radius = self.card_radius(card);
        for entity in &self.entities {
            if entity.is_dead() {
                continue;
            }
            if entity.is_tower || entity.kind == EntityKind::Building {
                if distance(x, y, entity.x, entity.y) < entity.radius + spawn_radius + 0.15 {
                    return false;
                }
            }
        }
        true
    }

    fn card_deploys_anywhere(&self, card: &royale_defs::CardDef) -> bool {
        let skills = match &card.card_type {
            CardType::Troop { troop, .. } => self
                .defs
                .troops
                .get(troop)
                .map(|troop| troop.skills.clone())
                .unwrap_or_default(),
            CardType::Building { building } => self
                .defs
                .buildings
                .get(building)
                .map(|building| building.skills.clone())
                .unwrap_or_default(),
            CardType::Spell { .. } => return true,
        };
        skills.iter().any(|reference| {
            self.defs.skills.get(&reference.skill).is_some_and(|skill| {
                matches!(
                    skill.kind,
                    royale_defs::SkillKind::DeployAnywhere | royale_defs::SkillKind::Burrow
                )
            })
        })
    }

    fn card_radius(&self, card: &royale_defs::CardDef) -> f32 {
        match &card.card_type {
            CardType::Troop { troop, .. } => self
                .defs
                .troops
                .get(troop)
                .map(|troop| troop.radius)
                .unwrap_or(0.4),
            CardType::Building { building } => self
                .defs
                .buildings
                .get(building)
                .map(|building| building.radius)
                .unwrap_or(0.6),
            CardType::Spell { .. } => 0.2,
        }
    }

    fn enemy_princess_alive(&self, side: Side, lane: u8) -> bool {
        let enemy = side.enemy();
        self.entities.iter().any(|entity| {
            entity.is_tower
                && !entity.is_dead()
                && entity.side == enemy
                && entity.tower_role == TowerRole::Princess
                && entity.lane == lane
        })
    }

    pub fn deck_for(&self, user_id: Uuid) -> Option<Vec<String>> {
        let index = self.player_index(user_id)?;
        let player = &self.players[index];
        let mut cards: Vec<u16> = player.hand.clone();
        cards.push(player.next);
        cards.extend(player.queue.iter().copied());
        Some(
            cards
                .into_iter()
                .filter_map(|card| {
                    self.defs
                        .cards
                        .get(card as usize)
                        .map(|card| card.slug.clone())
                })
                .collect(),
        )
    }

    /// Ordena o deck para o formato cíclico esperado por `Player::new`.
    pub fn normalize_deck(mut deck: Vec<u16>) -> VecDeque<u16> {
        deck.drain(..).collect()
    }
}

impl Match {
    pub fn entity_count(&self) -> usize {
        self.entities.len() + self.projectiles.len() + self.zones.len()
    }
}

impl Match {
    /// Tropas ativas na arena, no formato de estado compartilhado.
    pub fn troops(&self) -> Vec<TroopState> {
        self.entities
            .iter()
            .filter(|entity| entity.kind == EntityKind::Troop)
            .map(|entity| TroopState {
                id: entity.id,
                card_slug: entity.card_slug.clone(),
                troop_slug: entity.slug.clone(),
                health: entity.hp,
                pos_x: entity.x,
                pos_y: entity.y,
                effects: entity.effects.clone(),
            })
            .collect()
    }

    pub fn time(&self) -> u32 {
        self.elapsed_ms
    }
}
