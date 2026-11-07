use royale_defs::EffectKind;
use royale_shared::{EVENT_DEATH, EVENT_MATCH_END, EVENT_TOWER_DOWN, Side};

use crate::Match;
use crate::effects::AffectMode;
use crate::types::*;
use crate::util::{MS_PER_TICK, distance};

impl Match {
    pub fn update_skills(&mut self) {
        let frame = MS_PER_TICK as i32;
        let ids: Vec<u32> = self
            .entities
            .iter()
            .filter(|entity| !entity.is_dead())
            .map(|entity| entity.id)
            .collect();

        for id in ids {
            let Some(index) = self.entity_index(id) else {
                continue;
            };
            let stunned = self.entities[index].is_stunned();
            let burrowed = self.entities[index].burrowed;
            let side = self.entities[index].side;
            let x = self.entities[index].x;
            let y = self.entities[index].y;
            let card_slug = self.entities[index].card_slug.clone();
            let skills = self.entities[index].skills.clone();

            for skill in &skills {
                match skill {
                    SkillRuntime::SpawnPeriodic(periodic) => {
                        if stunned || burrowed {
                            continue;
                        }
                        if self.entities[index].spawn_timer > 0 {
                            self.entities[index].spawn_timer -= frame;
                        }
                        if self.entities[index].spawn_timer <= 0 {
                            self.entities[index].spawn_timer = periodic.interval_ms as i32;
                            let alive = self
                                .entities
                                .iter()
                                .filter(|entity| {
                                    !entity.is_dead()
                                        && entity.spawn_serial == id
                                        && entity.slug == periodic.troop
                                })
                                .count() as u32;
                            if alive < periodic.max_alive {
                                let count = periodic.count.max(1);
                                for (px, py) in crate::spawn::formation_positions(count, x, y) {
                                    let (nx, ny) = self.clamp_spawn(side, px, py);
                                    self.spawn_troop(
                                        side,
                                        &card_slug,
                                        &periodic.troop,
                                        nx,
                                        ny,
                                        None,
                                        true,
                                        Some(id),
                                    );
                                }
                            }
                        }
                    }
                    SkillRuntime::Aura(aura) => {
                        if stunned {
                            continue;
                        }
                        if aura.dps > 0.0 {
                            let damage = (aura.dps * crate::util::DT).round().max(1.0) as i32;
                            let targets: Vec<u32> = self
                                .entities
                                .iter()
                                .filter(|entity| {
                                    !entity.is_dead()
                                        && entity.side != side
                                        && !entity.is_tower
                                        && distance(x, y, entity.x, entity.y)
                                            <= aura.radius + entity.radius
                                })
                                .map(|entity| entity.id)
                                .collect();
                            for target in targets {
                                self.deal_damage(
                                    target,
                                    damage,
                                    side,
                                    Some(id),
                                    Some((x, y)),
                                    false,
                                );
                            }
                        }
                        if aura.heal_per_sec > 0.0 {
                            let heal =
                                (aura.heal_per_sec * crate::util::DT).round().max(1.0) as i32;
                            let allies: Vec<u32> = self
                                .entities
                                .iter()
                                .filter(|entity| {
                                    !entity.is_dead()
                                        && entity.side == side
                                        && !entity.is_tower
                                        && distance(x, y, entity.x, entity.y)
                                            <= aura.radius + entity.radius
                                })
                                .map(|entity| entity.id)
                                .collect();
                            for ally in allies {
                                self.heal_entity(ally, heal);
                            }
                        }
                        if !aura.effects.is_empty() {
                            for effect in &aura.effects {
                                let mode = match self.defs.effects.get(effect).map(|def| def.kind) {
                                    Some(kind) => {
                                        if matches!(
                                            kind,
                                            EffectKind::Rage
                                                | EffectKind::Heal
                                                | EffectKind::SpeedBoost
                                                | EffectKind::AttackBoost
                                        ) {
                                            AffectMode::Allies
                                        } else {
                                            AffectMode::Enemies
                                        }
                                    }
                                    None => AffectMode::Enemies,
                                };
                                self.apply_area_effect(x, y, aura.radius, effect, side, mode);
                            }
                        }
                    }
                    SkillRuntime::Hook(hook) => {
                        if stunned {
                            continue;
                        }
                        if self.entities[index].hook_timer > 0 {
                            self.entities[index].hook_timer -= frame;
                        }
                        if self.entities[index].hook_timer <= 0 {
                            let closest = self
                                .entities
                                .iter()
                                .filter(|entity| {
                                    !entity.is_dead()
                                        && !entity.is_tower
                                        && entity.side != side
                                        && !entity.untargetable()
                                        && distance(x, y, entity.x, entity.y) <= hook.range
                                })
                                .min_by(|a, b| {
                                    distance(x, y, a.x, a.y)
                                        .partial_cmp(&distance(x, y, b.x, b.y))
                                        .unwrap_or(std::cmp::Ordering::Equal)
                                })
                                .map(|entity| (entity.id, entity.x, entity.y));
                            if let Some((target_id, tx, ty)) = closest {
                                let dx = x - tx;
                                let dy = y - ty;
                                let length = (dx * dx + dy * dy).sqrt().max(0.001);
                                let pull = hook.pull_distance.min(length);
                                if let Some(target) = self
                                    .entities
                                    .iter_mut()
                                    .find(|entity| entity.id == target_id)
                                {
                                    target.x += dx / length * pull;
                                    target.y += dy / length * pull;
                                }
                                self.deal_damage(
                                    target_id,
                                    hook.damage,
                                    side,
                                    Some(id),
                                    Some((x, y)),
                                    true,
                                );
                                self.emit(royale_shared::EVENT_HIT, side, tx, ty, 0);
                                self.entities[index].hook_timer = hook.cooldown_ms as i32;
                            }
                        }
                    }
                    SkillRuntime::ElixirGenerate(generate) => {
                        if stunned {
                            continue;
                        }
                        if self.entities[index].elixir_timer > 0 {
                            self.entities[index].elixir_timer -= frame;
                        }
                        if self.entities[index].elixir_timer <= 0 {
                            self.add_elixir(side, generate.amount);
                            self.entities[index].elixir_timer = generate.interval_ms as i32;
                        }
                    }
                    SkillRuntime::ChargedShot(shot) => {
                        let moving = self.entities[index].moving;
                        let windup = self.entities[index].windup;
                        if !moving && windup == 0 && !stunned {
                            self.entities[index].charged_timer =
                                (self.entities[index].charged_timer + frame)
                                    .min(shot.charge_ms as i32);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn add_elixir(&mut self, side: Side, amount: f32) {
        let player = &mut self.players[side.index()];
        player.elixir = (player.elixir + amount).min(royale_shared::MAX_ELIXIR);
        self.emit(royale_shared::EVENT_ELIXIR, side, 0.0, 0.0, 0);
    }

    pub fn mark_dead(&mut self, id: u32) {
        if let Some(index) = self.entity_index(id) {
            self.entities[index].hp = self.entities[index].hp.min(0);
        }
        if !self.dead.contains(&id) {
            self.dead.push(id);
        }
    }

    pub fn process_deaths(&mut self) {
        let dead = std::mem::take(&mut self.dead);
        for id in dead {
            let Some(index) = self.entity_index(id) else {
                continue;
            };
            let entity = self.entities.remove(index);
            if entity.is_tower {
                self.handle_tower_death(&entity);
                continue;
            }

            self.emit(EVENT_DEATH, entity.side, entity.x, entity.y, 0);
            let death = entity.death_spec.clone();
            if !death.is_empty() {
                if death.damage > 0 && death.radius > 0.0 {
                    self.area_damage(
                        entity.x,
                        entity.y,
                        death.radius,
                        death.damage,
                        entity.side,
                        Some(entity.id),
                        TargetMask(
                            TargetMask::GROUND
                                | TargetMask::AIR
                                | TargetMask::BUILDINGS
                                | TargetMask::TOWERS,
                        ),
                        0.5,
                        &death.effects,
                        false,
                    );
                } else {
                    for effect in &death.effects {
                        self.apply_area_effect(
                            entity.x,
                            entity.y,
                            death.radius.max(1.0),
                            effect,
                            entity.side,
                            AffectMode::Auto,
                        );
                    }
                }
                for spawn in &death.spawn {
                    self.queue_spawn(
                        entity.side,
                        &entity.card_slug,
                        &spawn.troop,
                        spawn.count,
                        entity.x,
                        entity.y,
                        None,
                        false,
                        spawn.delay_ms,
                    );
                }
            }

            if let Some((effects, radius)) = &entity.death_effects {
                for effect in effects {
                    self.apply_area_effect(
                        entity.x,
                        entity.y,
                        *radius,
                        effect,
                        entity.side,
                        AffectMode::Auto,
                    );
                }
            }

            if let Some(rebirth) = &entity.rebirth {
                if !rebirth.troop.is_empty() {
                    self.queue_spawn(
                        entity.side,
                        &entity.card_slug,
                        &rebirth.troop,
                        1,
                        entity.x,
                        entity.y,
                        Some(rebirth.health),
                        true,
                        rebirth.delay_ms,
                    );
                }
            }

            if let Some((building, health)) = &entity.transform_on_death {
                self.pending.push(PendingAction {
                    at_tick: self.tick + 1,
                    side: entity.side,
                    kind: PendingKind::SpawnBuilding {
                        card_slug: entity.card_slug.clone(),
                        building_slug: building.clone(),
                        x: entity.x,
                        y: entity.y,
                        health: Some(*health),
                    },
                });
            }

            for skill in &entity.skills {
                if let SkillRuntime::ElixirOnDeath { amount } = skill {
                    self.add_elixir(entity.side.enemy(), *amount);
                }
            }
        }
    }

    fn handle_tower_death(&mut self, tower: &Entity) {
        let owner = tower.side;
        let attacker = owner.enemy();
        self.emit(EVENT_TOWER_DOWN, owner, tower.x, tower.y, 0);
        match tower.tower_role {
            TowerRole::King => {
                self.players[attacker.index()].crowns = 3;
                self.end_match(Some(attacker), EndReason::KingDestroyed);
            }
            TowerRole::Princess => {
                let player = &mut self.players[attacker.index()];
                player.crowns = (player.crowns + 1).min(2);
                let mut activated: Vec<(f32, f32)> = Vec::new();
                for entity in &mut self.entities {
                    if entity.is_tower
                        && entity.side == owner
                        && entity.tower_role == TowerRole::King
                        && !entity.active
                    {
                        entity.active = true;
                        activated.push((entity.x, entity.y));
                    }
                }
                for (x, y) in activated {
                    self.emit(royale_shared::EVENT_KING_ACTIVE, owner, x, y, 0);
                }
                if self.phase == Phase::Overtime {
                    self.end_match(Some(attacker), EndReason::SuddenDeath);
                }
            }
            TowerRole::None => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn queue_spawn(
        &mut self,
        side: Side,
        card_slug: &str,
        troop_slug: &str,
        count: u32,
        x: f32,
        y: f32,
        health: Option<i32>,
        no_rebirth: bool,
        delay_ms: u32,
    ) {
        let positions = crate::spawn::formation_positions(count, x, y);
        let ticks = (delay_ms / royale_shared::TICK_MS).max(1);
        for (px, py) in positions {
            self.pending.push(PendingAction {
                at_tick: self.tick + ticks,
                side,
                kind: PendingKind::SpawnTroop {
                    card_slug: card_slug.to_string(),
                    troop_slug: troop_slug.to_string(),
                    x: px,
                    y: py,
                    health,
                    no_rebirth,
                },
            });
        }
    }

    pub fn end_match(&mut self, winner: Option<Side>, reason: EndReason) {
        if self.phase == Phase::Ended {
            return;
        }
        self.phase = Phase::Ended;
        self.winner = winner;
        self.end_reason = Some(reason);
        self.emit(
            EVENT_MATCH_END,
            winner.unwrap_or(Side::Blue),
            self.arena.width * 0.5,
            self.arena.height * 0.5,
            0,
        );
    }

    pub fn result(&self) -> MatchResult {
        MatchResult {
            winner: self.winner,
            end_reason: self.end_reason.unwrap_or(EndReason::Draw),
            crowns: [self.players[0].crowns, self.players[1].crowns],
            damage: [self.players[0].tower_damage, self.players[1].tower_damage],
            duration_ms: self.elapsed_ms,
        }
    }
}
