use royale_shared::Side;

use crate::Match;
use crate::effects::AffectMode;
use crate::types::*;
use crate::util::{MS_PER_TICK, distance};

impl Match {
    pub fn cast_spell(&mut self, side: Side, card_slug: &str, spell_slug: &str, x: f32, y: f32) {
        let Some(def) = self.defs.spells.get(spell_slug).cloned() else {
            return;
        };
        let mask = Match::targets_mask(&def.targets);
        let spell_idx = self.catalog.spell_index(spell_slug).unwrap_or(0);
        self.emit(royale_shared::EVENT_SPELL, side, x, y, spell_idx);

        if def.damage > 0 || !def.effects.is_empty() {
            let mut hostile_effects: Vec<String> = Vec::new();
            for effect in &def.effects {
                let beneficial = self.defs.effects.get(effect).is_some_and(|def| {
                    matches!(
                        def.kind,
                        royale_defs::EffectKind::Rage
                            | royale_defs::EffectKind::Heal
                            | royale_defs::EffectKind::SpeedBoost
                            | royale_defs::EffectKind::AttackBoost
                    )
                });
                if beneficial {
                    self.apply_area_effect(x, y, def.radius, effect, side, AffectMode::Allies);
                } else {
                    hostile_effects.push(effect.clone());
                }
            }
            self.area_damage(
                x,
                y,
                def.radius,
                def.damage,
                side,
                None,
                mask,
                def.tower_damage_mult,
                &hostile_effects,
                false,
            );
        }

        if let Some(rolling) = &def.rolling {
            let dir = if side == Side::Blue { -1.0 } else { 1.0 };
            let roller_id = self.next_entity_id();
            self.rollers.push(RollingSpell {
                id: roller_id,
                side,
                spell_idx,
                x,
                y,
                dir_x: 0.0,
                dir_y: dir,
                speed: rolling.speed.max(0.5),
                distance_left: rolling.distance.max(0.5),
                width: rolling.width.max(0.4),
                damage: def.damage,
                tower_damage_mult: def.tower_damage_mult,
                effects: def.effects.clone(),
                knockback: rolling.knockback,
                hit_ids: Vec::new(),
                mask,
            });
        }

        let pull = def.pull.as_ref().map(|pull| pull.strength);
        let zone_duration = def
            .zone
            .as_ref()
            .map(|zone| zone.duration_ms)
            .or_else(|| def.pull.as_ref().map(|pull| pull.duration_ms))
            .or_else(|| {
                def.spawn
                    .as_ref()
                    .map(|spawn| spawn.count * spawn.interval_ms + 600)
            });

        if let Some(duration) = zone_duration {
            let zone_def = def.zone.as_ref();
            let spawn = def.spawn.as_ref().map(|spawn| ZoneSpawn {
                troop: spawn.troop.clone(),
                remaining: spawn.count,
                interval_ms: spawn.interval_ms.max(100),
                timer: spawn.first_delay_ms as i32,
                radius: spawn.radius,
            });
            let affects_allies = zone_def.map(|zone| zone.affects_allies).unwrap_or(false);
            let mut zone_effects = zone_def
                .map(|zone| zone.effects.clone())
                .unwrap_or_default();
            if let Some(effect) = zone_def.and_then(|zone| zone.effect.clone()) {
                zone_effects.push(effect);
            }
            let zone_id = self.next_entity_id();
            self.zones.push(Zone {
                id: zone_id,
                side,
                spell_idx,
                x,
                y,
                radius: def.radius,
                remaining_ms: duration as i32,
                dps: zone_def.map(|zone| zone.dps).unwrap_or(0.0),
                effects: zone_effects,
                affects_allies,
                pull,
                spawn,
                slow_mult: zone_def.and_then(|zone| zone.slow_mult),
                tick_acc: 0,
                tower_damage_mult: def.tower_damage_mult,
                mask,
            });
        }

        let _ = card_slug;
    }

    pub fn update_spells(&mut self) {
        self.update_zones();
        self.update_rollers();
    }

    fn update_zones(&mut self) {
        let frame = MS_PER_TICK as i32;
        let mut remove: Vec<usize> = Vec::new();
        let zone_count = self.zones.len();
        for index in 0..zone_count {
            let zone = self.zones[index].clone();
            let remaining = zone.remaining_ms - frame;

            let dot_effect = zone.effects.iter().any(|effect| {
                self.defs.effects.get(effect).is_some_and(|def| {
                    matches!(
                        def.kind,
                        royale_defs::EffectKind::Poison
                            | royale_defs::EffectKind::Burn
                            | royale_defs::EffectKind::Shock
                    )
                })
            });
            if zone.dps > 0.0 && !dot_effect {
                let damage = (zone.dps * crate::util::DT).round().max(1.0) as i32;
                let targets: Vec<u32> = self
                    .entities
                    .iter()
                    .filter(|entity| {
                        !entity.is_dead()
                            && entity.side != zone.side
                            && zone.mask.can_hit(entity.kind, entity.flying)
                            && distance(zone.x, zone.y, entity.x, entity.y)
                                <= zone.radius + entity.radius
                    })
                    .map(|entity| entity.id)
                    .collect();
                for id in targets {
                    let is_tower = self.entity(id).is_some_and(|entity| entity.is_tower);
                    let applied = if is_tower {
                        (damage as f32 * zone.tower_damage_mult).round() as i32
                    } else {
                        damage
                    };
                    self.deal_damage(id, applied, zone.side, None, Some((zone.x, zone.y)), false);
                }
            }

            if !zone.effects.is_empty() {
                let mode = if zone.affects_allies {
                    AffectMode::Allies
                } else {
                    AffectMode::Auto
                };
                for effect in &zone.effects {
                    self.apply_area_effect(zone.x, zone.y, zone.radius, effect, zone.side, mode);
                }
            }

            if let Some(strength) = zone.pull {
                let enemies: Vec<u32> = self
                    .entities
                    .iter()
                    .filter(|entity| {
                        !entity.is_dead()
                            && !entity.is_tower
                            && entity.side != zone.side
                            && !entity.untargetable()
                            && distance(zone.x, zone.y, entity.x, entity.y) <= zone.radius
                    })
                    .map(|entity| entity.id)
                    .collect();
                for id in enemies {
                    if let Some(entity) = self.entities.iter_mut().find(|entity| entity.id == id) {
                        let dx = zone.x - entity.x;
                        let dy = zone.y - entity.y;
                        let length = (dx * dx + dy * dy).sqrt().max(0.001);
                        let step = (strength * crate::util::DT).min(length);
                        entity.x += dx / length * step;
                        entity.y += dy / length * step;
                    }
                }
            }

            let mut spawn_request: Option<String> = None;
            if let Some(spawn) = &mut self.zones[index].spawn {
                if spawn.remaining > 0 {
                    spawn.timer -= frame;
                    if spawn.timer <= 0 {
                        spawn.timer = spawn.interval_ms as i32;
                        spawn.remaining -= 1;
                        spawn_request = Some(spawn.troop.clone());
                    }
                }
            }
            if let Some(troop) = spawn_request {
                let (sx, sy) = {
                    let spawn_radius = self.zones[index]
                        .spawn
                        .as_ref()
                        .map(|spawn| spawn.radius)
                        .unwrap_or(1.0);
                    let angle = self.rng.range(0.0, std::f32::consts::TAU);
                    let radius = self.rng.range(0.0, spawn_radius.max(0.2));
                    (zone.x + angle.cos() * radius, zone.y + angle.sin() * radius)
                };
                let (nx, ny) = self.clamp_spawn(zone.side, sx, sy);
                self.spawn_troop(zone.side, "", &troop, nx, ny, None, true, None);
            }

            let spawn_done = self.zones[index]
                .spawn
                .as_ref()
                .is_none_or(|spawn| spawn.remaining == 0);
            if let Some(zone_mut) = self.zones.get_mut(index) {
                zone_mut.remaining_ms = remaining.max(0);
            }
            if remaining <= 0 && spawn_done {
                remove.push(index);
            }
        }
        remove.sort_unstable();
        remove.dedup();
        for index in remove.into_iter().rev() {
            if index < self.zones.len() {
                self.zones.remove(index);
            }
        }
    }

    fn update_rollers(&mut self) {
        let mut remove: Vec<usize> = Vec::new();
        let count = self.rollers.len();
        for index in 0..count {
            let roller = self.rollers[index].clone();
            let step = roller.speed * crate::util::DT;
            let x = roller.x + roller.dir_x * step;
            let y = roller.y + roller.dir_y * step;
            let radius = roller.width * 0.5;

            let hits: Vec<(u32, f32, f32)> = self
                .entities
                .iter()
                .filter(|entity| {
                    !entity.is_dead()
                        && entity.side != roller.side
                        && !roller.hit_ids.contains(&entity.id)
                        && roller.mask.can_hit(entity.kind, entity.flying)
                        && distance(x, y, entity.x, entity.y) <= radius + entity.radius
                })
                .map(|entity| (entity.id, entity.x, entity.y))
                .collect();
            for (id, ex, ey) in hits {
                let is_tower = self.entity(id).is_some_and(|entity| entity.is_tower);
                let damage = if is_tower {
                    (roller.damage as f32 * roller.tower_damage_mult).round() as i32
                } else {
                    roller.damage
                };
                self.deal_damage(id, damage, roller.side, None, Some((x, y)), false);
                for effect in &roller.effects {
                    self.apply_effect_preset(id, effect, roller.side, Some((x, y)));
                }
                self.knockback(id, roller.knockback, roller.side, Some((x, y)));
                self.emit(royale_shared::EVENT_HIT, roller.side, ex, ey, 0);
                if let Some(roller) = self.rollers.get_mut(index) {
                    roller.hit_ids.push(id);
                }
            }

            let distance_left = roller.distance_left - step;
            let out = x < 0.0 || x > self.arena.width || y < 0.0 || y > self.arena.height;
            if let Some(roller) = self.rollers.get_mut(index) {
                roller.x = x;
                roller.y = y;
                roller.distance_left = distance_left;
            }
            if distance_left <= 0.0 || out {
                remove.push(index);
            }
        }
        remove.sort_unstable();
        remove.dedup();
        for index in remove.into_iter().rev() {
            if index < self.rollers.len() {
                self.rollers.remove(index);
            }
        }
    }
}
