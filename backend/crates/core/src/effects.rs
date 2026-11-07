use royale_defs::{EffectKind, TargetKind};
use royale_shared::Side;

use crate::Match;
use crate::types::*;
use crate::util::{MS_PER_TICK, clamp, distance};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffectMode {
    Enemies,
    Allies,
    Auto,
}

impl Match {
    pub fn entity_index(&self, id: u32) -> Option<usize> {
        self.entities.iter().position(|entity| entity.id == id)
    }

    pub fn entity(&self, id: u32) -> Option<&Entity> {
        self.entities.iter().find(|entity| entity.id == id)
    }

    pub fn emit(&mut self, kind: u8, side: Side, x: f32, y: f32, index: u16) {
        self.events.push(Event {
            kind,
            x,
            y,
            side: side as u8,
            index,
        });
    }

    pub fn apply_effect_preset(
        &mut self,
        target_id: u32,
        preset: &str,
        source_side: Side,
        source_pos: Option<(f32, f32)>,
    ) {
        let Some(def) = self.defs.effects.get(preset).cloned() else {
            return;
        };
        if def.kind == EffectKind::Knockback {
            let amount = def.params.amount.unwrap_or(1.0);
            self.knockback(target_id, amount, source_side, source_pos);
            return;
        }
        let Some(index) = self.entity_index(target_id) else {
            return;
        };
        let entity = &mut self.entities[index];
        if entity.is_dead() {
            return;
        }
        let runtime = EffectInstance {
            preset: preset.to_string(),
            kind: def.kind,
            remaining_ms: def.duration_ms as i32,
            duration_ms: def.duration_ms as i32,
            speed_mult: def.params.speed_mult.unwrap_or(1.0),
            attack_mult: def.params.attack_mult.unwrap_or(1.0),
            dps: def.params.dps.unwrap_or(0.0),
            tick_ms: def.params.tick_ms.unwrap_or(500).max(50),
            tick_acc: 0,
            amount: def.params.amount.unwrap_or(0.0),
            heal_per_sec: def.params.heal_per_sec.unwrap_or(0.0),
            damage_mult: def.params.damage_mult.unwrap_or(1.0),
            source_side,
        };
        if let Some(existing) = entity
            .effects
            .iter_mut()
            .find(|effect| effect.preset == preset)
        {
            existing.remaining_ms = existing.remaining_ms.max(runtime.remaining_ms);
            existing.source_side = source_side;
        } else {
            entity.effects.push(runtime);
        }
    }

    pub fn apply_area_effect(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        preset: &str,
        source_side: Side,
        mode: AffectMode,
    ) {
        let Some(def) = self.defs.effects.get(preset) else {
            return;
        };
        let beneficial = matches!(
            def.kind,
            EffectKind::Rage | EffectKind::Heal | EffectKind::SpeedBoost | EffectKind::AttackBoost
        );
        let targets_allies = match mode {
            AffectMode::Allies => true,
            AffectMode::Enemies => false,
            AffectMode::Auto => beneficial,
        };
        let ids: Vec<u32> = self
            .entities
            .iter()
            .filter(|entity| {
                !entity.is_dead()
                    && !entity.is_tower
                    && distance(center_x, center_y, entity.x, entity.y) <= radius + entity.radius
                    && if targets_allies {
                        entity.side == source_side
                    } else {
                        entity.side != source_side
                    }
            })
            .map(|entity| entity.id)
            .collect();
        for id in ids {
            self.apply_effect_preset(id, preset, source_side, Some((center_x, center_y)));
        }
    }

    pub fn tick_effects(&mut self) {
        let mut dots: Vec<(u32, i32, Side, f32, f32)> = Vec::new();
        let frame_ms = MS_PER_TICK;
        for entity in &mut self.entities {
            if entity.is_tower {
                // Torres recebem efeitos (congelamento etc.), mas não DOT de área.
            }
            let mut index = 0;
            while index < entity.effects.len() {
                let effect = &mut entity.effects[index];
                effect.remaining_ms -= frame_ms as i32;
                match effect.kind {
                    EffectKind::Poison | EffectKind::Burn | EffectKind::Shock => {
                        effect.tick_acc += frame_ms as i32;
                        while effect.tick_acc >= effect.tick_ms as i32 {
                            effect.tick_acc -= effect.tick_ms as i32;
                            let damage =
                                (effect.dps * effect.tick_ms as f32 / 1000.0).round() as i32;
                            if damage > 0 {
                                dots.push((
                                    entity.id,
                                    damage,
                                    effect.source_side,
                                    entity.x,
                                    entity.y,
                                ));
                            }
                        }
                    }
                    EffectKind::Heal => {
                        let heal = effect.heal_per_sec * (frame_ms / 1000.0);
                        entity.hp = (entity.hp + heal.round() as i32).min(entity.max_hp);
                    }
                    _ => {}
                }
                if effect.remaining_ms <= 0 {
                    entity.effects.remove(index);
                } else {
                    index += 1;
                }
            }
        }
        for (id, damage, side, x, y) in dots {
            self.deal_damage(id, damage, side, None, Some((x, y)), false);
        }
    }

    pub fn knockback(
        &mut self,
        target_id: u32,
        amount: f32,
        source_side: Side,
        source_pos: Option<(f32, f32)>,
    ) {
        let Some(index) = self.entity_index(target_id) else {
            return;
        };
        let entity = &self.entities[index];
        if entity.is_tower || entity.flying || amount <= 0.0 {
            return;
        }
        let (sx, sy) = source_pos.unwrap_or((entity.x, entity.y + source_side_sign(source_side)));
        let dx = entity.x - sx;
        let dy = entity.y - sy;
        let length = (dx * dx + dy * dy).sqrt();
        let (ux, uy) = if length < 0.001 {
            (0.0, source_side_sign(source_side))
        } else {
            (dx / length, dy / length)
        };
        let old = (entity.x, entity.y);
        let mut nx = entity.x + ux * amount;
        let mut ny = entity.y + uy * amount;
        nx = clamp(nx, entity.radius, self.arena.width - entity.radius);
        ny = clamp(ny, entity.radius, self.arena.height - entity.radius);
        if ny > self.arena.river_top
            && ny < self.arena.river_bottom
            && !crate::util::on_bridge(&self.arena, nx)
        {
            nx = old.0;
            ny = old.1;
        }
        let entity = &mut self.entities[index];
        entity.x = nx;
        entity.y = ny;
    }

    /// Aplica dano autoritativo. Retorna o dano efetivo.
    pub fn deal_damage(
        &mut self,
        target_id: u32,
        raw_damage: i32,
        source_side: Side,
        source_id: Option<u32>,
        source_pos: Option<(f32, f32)>,
        allow_reflect: bool,
    ) -> i32 {
        if raw_damage <= 0 {
            return 0;
        }
        let Some(index) = self.entity_index(target_id) else {
            return 0;
        };
        if self.entities[index].hp <= 0 || self.dead.contains(&target_id) {
            return 0;
        }

        let taken_mult = self.entities[index].damage_taken_mult();
        let mut amount = (raw_damage as f32 * taken_mult).round() as i32;

        if self.entities[index].shield > 0 {
            let shield = self.entities[index].shield;
            let absorbed = shield.min(amount);
            self.entities[index].shield -= absorbed;
            amount -= absorbed;
        }

        let is_tower = self.entities[index].is_tower;
        let owner_side = self.entities[index].side;
        let x = self.entities[index].x;
        let y = self.entities[index].y;
        let role = self.entities[index].tower_role;
        let hp_before = self.entities[index].hp;
        let reflect = self.entities[index]
            .skills
            .iter()
            .find_map(|skill| match skill {
                SkillRuntime::Reflect { percent, max_range } => Some((*percent, *max_range)),
                _ => None,
            });

        self.entities[index].hp -= amount;
        self.entities[index].last_damage_tick = self.tick;
        let effective = amount.min(hp_before.max(0)).max(0);
        let dead = self.entities[index].hp <= 0;

        if is_tower && effective > 0 {
            let attacker = source_side.index();
            self.players[attacker].tower_damage += effective as f32;
            if role == TowerRole::King && !self.entities[index].active {
                self.entities[index].active = true;
                self.emit(royale_shared::EVENT_KING_ACTIVE, owner_side, x, y, 0);
            }
        }

        if effective > 0 {
            self.emit(royale_shared::EVENT_HIT, source_side, x, y, 0);
        }

        if dead && !self.dead.contains(&target_id) {
            self.dead.push(target_id);
        }

        if allow_reflect && effective > 0 {
            if let (Some((percent, max_range)), Some(attacker)) = (reflect, source_id) {
                let attacker_pos = self
                    .entity(attacker)
                    .map(|entity| (entity.x, entity.y))
                    .or(source_pos);
                if let Some((ax, ay)) = attacker_pos {
                    if distance(x, y, ax, ay) <= max_range + 0.5 && attacker != target_id {
                        let ratio = if percent > 1.0 {
                            percent / 100.0
                        } else {
                            percent
                        };
                        let reflected = ((effective as f32) * ratio).round() as i32;
                        if reflected > 0 {
                            self.deal_damage(
                                attacker,
                                reflected,
                                owner_side,
                                Some(target_id),
                                Some((x, y)),
                                false,
                            );
                        }
                    }
                }
            }
        }

        effective
    }

    pub fn heal_entity(&mut self, id: u32, amount: i32) {
        if amount <= 0 {
            return;
        }
        if let Some(index) = self.entity_index(id) {
            let entity = &mut self.entities[index];
            if !entity.is_dead() {
                entity.hp = (entity.hp + amount).min(entity.max_hp);
            }
        }
    }

    /// Dano em área com máscara de alvos.
    pub fn area_damage(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        damage: i32,
        source_side: Side,
        source_id: Option<u32>,
        mask: TargetMask,
        tower_mult: f32,
        effects: &[String],
        allow_reflect: bool,
    ) {
        if damage <= 0 && effects.is_empty() {
            return;
        }
        let targets: Vec<(u32, f32, f32, bool)> = self
            .entities
            .iter()
            .filter(|entity| {
                !entity.is_dead()
                    && entity.side != source_side
                    && mask.can_hit(entity.kind, entity.flying)
                    && distance(center_x, center_y, entity.x, entity.y) <= radius + entity.radius
            })
            .map(|entity| (entity.id, entity.x, entity.y, entity.is_tower))
            .collect();
        for (id, ex, ey, is_tower) in targets {
            let applied = if is_tower {
                (damage as f32 * tower_mult).round() as i32
            } else {
                damage
            };
            self.deal_damage(
                id,
                applied,
                source_side,
                source_id,
                Some((center_x, center_y)),
                allow_reflect,
            );
            for effect in effects {
                self.apply_effect_preset(id, effect, source_side, Some((center_x, center_y)));
            }
            let _ = (ex, ey);
        }
    }

    /// Máscara efetiva de um feitiço/tropa contra o alvo.
    pub fn targets_mask(targets: &[TargetKind]) -> TargetMask {
        TargetMask::from_kinds(targets)
    }
}

fn source_side_sign(side: Side) -> f32 {
    match side {
        Side::Blue => -1.0,
        Side::Red => 1.0,
    }
}
