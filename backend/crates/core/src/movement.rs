use crate::Match;
use crate::types::*;
use crate::util::{
    DT, MS_PER_TICK, clamp, crossing_waypoint, distance, in_river, nearest_bridge, on_bridge,
};

fn edge_distance(a: &Entity, b: &Entity) -> f32 {
    distance(a.x, a.y, b.x, b.y) - a.radius - b.radius
}

pub fn valid_target(attacker: &Entity, target: &Entity) -> bool {
    !target.is_dead()
        && !target.untargetable()
        && target.side != attacker.side
        && attacker.targets.can_hit(target.kind, target.flying)
}

impl Match {
    pub fn refresh_targets(&mut self) {
        let ids: Vec<u32> = self
            .entities
            .iter()
            .filter(|entity| entity.attack.is_some() && !entity.is_dead() && !entity.is_tower)
            .map(|entity| entity.id)
            .collect();
        for id in ids {
            let Some(index) = self.entity_index(id) else {
                continue;
            };
            if self.entities[index].is_stunned() {
                continue;
            }
            if self.entities[index].is_tower {
                continue;
            }
            let keep = self.entities[index]
                .target
                .and_then(|target_id| self.entity(target_id).map(|target| (target_id, target)))
                .filter(|(_, target)| {
                    valid_target(&self.entities[index], target)
                        && edge_distance(&self.entities[index], target)
                            <= self.entities[index].sight_range * 1.25
                })
                .map(|(target_id, _)| target_id);
            if keep.is_some() {
                self.entities[index].target = keep;
                continue;
            }
            let best = self.acquire_target(index);
            self.entities[index].target = best;
            if best.is_none() {
                self.entities[index].ramp_target = None;
            }
        }

        // Torres: alvo dentro do alcance de ataque.
        let tower_ids: Vec<u32> = self
            .entities
            .iter()
            .filter(|entity| entity.is_tower && !entity.is_dead())
            .map(|entity| entity.id)
            .collect();
        for id in tower_ids {
            let Some(index) = self.entity_index(id) else {
                continue;
            };
            if !self.entities[index].active {
                self.entities[index].target = None;
                continue;
            }
            let keep = self.entities[index]
                .target
                .and_then(|target_id| self.entity(target_id).map(|target| (target_id, target)))
                .filter(|(_, target)| {
                    valid_target(&self.entities[index], target)
                        && edge_distance(&self.entities[index], target)
                            <= self.entities[index].sight_range * 1.1
                })
                .map(|(target_id, _)| target_id);
            let best = keep.or_else(|| self.acquire_target(index));
            self.entities[index].target = best;
        }
    }

    fn acquire_target(&self, index: usize) -> Option<u32> {
        let attacker = &self.entities[index];
        let mut best: Option<(f32, u32)> = None;
        for target in &self.entities {
            if !valid_target(attacker, target) {
                continue;
            }
            let dist = edge_distance(attacker, target);
            if dist > attacker.sight_range {
                continue;
            }
            if best.is_none_or(|(best_dist, _)| dist < best_dist) {
                best = Some((dist, target.id));
            }
        }
        best.map(|(_, id)| id)
    }

    pub fn objective_point(&self, entity: &Entity) -> Option<(f32, f32)> {
        let enemy = entity.side.enemy();
        let mut princesses: Vec<&Entity> = Vec::new();
        let mut kings: Vec<&Entity> = Vec::new();
        for tower in &self.entities {
            if !tower.is_tower || tower.is_dead() || tower.side != enemy {
                continue;
            }
            match tower.tower_role {
                TowerRole::Princess => princesses.push(tower),
                TowerRole::King => kings.push(tower),
                TowerRole::None => {}
            }
        }
        let pool = if princesses.is_empty() {
            kings
        } else {
            princesses
        };
        pool.into_iter()
            .min_by(|a, b| {
                distance(entity.x, entity.y, a.x, a.y)
                    .partial_cmp(&distance(entity.x, entity.y, b.x, b.y))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|tower| (tower.x, tower.y))
    }

    pub fn update_movement(&mut self) {
        let ids: Vec<u32> = self
            .entities
            .iter()
            .filter(|entity| {
                !entity.is_dead() && !entity.is_tower && entity.kind == EntityKind::Troop
            })
            .map(|entity| entity.id)
            .collect();

        for id in ids {
            let Some(index) = self.entity_index(id) else {
                continue;
            };
            let stunned = self.entities[index].is_stunned();
            let snared = self.entities[index].is_snared();

            if !stunned && !snared {
                self.update_special_movement(index);
            }

            if self.entities[index].jump.is_some() {
                continue;
            }
            if self.entities[index].dash_end_tick > self.tick {
                continue;
            }

            let entity = &self.entities[index];
            let target_pos = entity
                .target
                .and_then(|target_id| self.entity(target_id))
                .map(|target| (target.x, target.y, target.radius, target.id))
                .or_else(|| self.objective_point(entity).map(|(x, y)| (x, y, 0.5, 0)));

            let Some((tx, ty, target_radius, target_id)) = target_pos else {
                self.entities[index].moving = false;
                self.entities[index].anim = royale_shared::ANIM_IDLE;
                continue;
            };

            let entity = &self.entities[index];
            let attack_range = entity.attack.as_ref().map(|a| a.range).unwrap_or(0.0);
            let in_range = distance(entity.x, entity.y, tx, ty)
                <= attack_range + target_radius + entity.radius + 0.02;

            if in_range {
                self.entities[index].moving = false;
                if self.entities[index].anim != royale_shared::ANIM_ATTACK {
                    self.entities[index].anim = royale_shared::ANIM_IDLE;
                }
                self.entities[index].idle_ms += MS_PER_TICK as i32;
                if self.entities[index].idle_ms
                    >= self.entities[index]
                        .skills
                        .iter()
                        .find_map(|skill| match skill {
                            SkillRuntime::Charge(charge) => Some(charge.reset_ms as i32),
                            _ => None,
                        })
                        .unwrap_or(2000)
                    && self.entities[index].windup == 0
                {
                    self.entities[index].charge_moved = 0.0;
                    self.entities[index].charge_ready = false;
                }
                continue;
            }

            self.entities[index].idle_ms = 0;
            self.entities[index].moving = true;
            self.entities[index].anim = royale_shared::ANIM_WALK;

            let mut goal = (tx, ty);
            let entity = &self.entities[index];
            if !entity.flying {
                if let Some(waypoint) =
                    crossing_waypoint(&self.arena, entity.x, entity.y, tx, ty, entity.radius)
                {
                    goal = waypoint;
                }
            }

            let mut speed_mult = entity.speed_mult();
            if entity.is_snared() {
                speed_mult = 0.0;
            }
            if entity.burrowed {
                speed_mult *= entity
                    .skills
                    .iter()
                    .find_map(|skill| match skill {
                        SkillRuntime::Burrow(burrow) => Some(burrow.speed_mult),
                        _ => None,
                    })
                    .unwrap_or(1.4);
            }
            if entity.charge_ready {
                speed_mult *= entity
                    .skills
                    .iter()
                    .find_map(|skill| match skill {
                        SkillRuntime::Charge(charge) => Some(charge.speed_mult),
                        _ => None,
                    })
                    .unwrap_or(1.0);
            }

            let dx = goal.0 - entity.x;
            let dy = goal.1 - entity.y;
            let length = (dx * dx + dy * dy).sqrt();
            if length < 0.001 {
                continue;
            }
            let step = entity.base_speed.max(0.05) * speed_mult * DT;
            let step = step.min(length);
            let nx = entity.x + dx / length * step;
            let ny = entity.y + dy / length * step;

            let entity = &mut self.entities[index];
            entity.x = nx;
            entity.y = ny;
            entity.last_move_tick = self.tick;
            if dx.abs() > 0.02 {
                entity.facing = if dx > 0.0 { 1 } else { -1 };
            }
            if let Some(SkillRuntime::Charge(charge)) = entity
                .skills
                .iter()
                .find(|skill| matches!(skill, SkillRuntime::Charge(_)))
            {
                entity.charge_moved += step;
                if entity.charge_moved >= charge.min_distance {
                    entity.charge_ready = true;
                }
            }
            let _ = target_id;
        }

        self.update_tower_facing();
        self.resolve_collisions();
        self.apply_environment_constraints();
    }

    fn update_special_movement(&mut self, index: usize) {
        let tick = self.tick;
        let entity = &self.entities[index];
        if entity.jump.is_some() || entity.dash_end_tick > tick {
            return;
        }

        let dash_skill = entity.skills.iter().find_map(|skill| match skill {
            SkillRuntime::Dash(dash) => Some(dash.clone()),
            _ => None,
        });
        let jump_skill = entity.skills.iter().find_map(|skill| match skill {
            SkillRuntime::Jump(jump) => Some(jump.clone()),
            _ => None,
        });
        let target = entity.target.and_then(|id| {
            self.entity(id)
                .map(|target| (target.id, target.x, target.y))
        });
        let Some((target_id, target_x, target_y)) = target else {
            return;
        };
        let target_distance = distance(entity.x, entity.y, target_x, target_y);

        if let Some(jump) = jump_skill {
            let crossing_river = (entity.y < self.arena.river_top
                && target_y > self.arena.river_bottom)
                || (entity.y > self.arena.river_bottom && target_y < self.arena.river_top);
            if (jump.damage > 0 || jump.radius > 0.0 || crossing_river)
                && self.entities[index].jump_cd <= 0
                && target_distance <= jump.trigger_range
                && target_distance > 0.6
            {
                let entity = &mut self.entities[index];
                let ticks = (jump.windup_ms / royale_shared::TICK_MS).max(1);
                entity.jump = Some(JumpState {
                    start_tick: tick,
                    end_tick: tick + ticks,
                    sx: entity.x,
                    sy: entity.y,
                    tx: target_x,
                    ty: target_y,
                    damage: jump.damage,
                    radius: jump.radius,
                });
                entity.jump_cd = 11_000;
                entity.attack_cd = entity.attack_cd.max(500);
                return;
            }
        }

        if let Some(dash) = dash_skill {
            if self.entities[index].dash_timer <= 0
                && self.entities[index].dash_end_tick <= tick
                && target_distance <= dash.trigger_range
                && target_distance > 0.5
            {
                let entity = &mut self.entities[index];
                entity.dash_end_tick = tick + (dash.duration_ms / royale_shared::TICK_MS).max(1);
                entity.dash_target = Some(target_id);
                entity.dash_timer = 6500;
                entity.charge_moved = 0.0;
                entity.charge_ready = false;
            }
        }
    }

    pub fn update_jumps(&mut self) {
        let tick = self.tick;
        let ids: Vec<u32> = self
            .entities
            .iter()
            .filter(|entity| entity.jump.is_some())
            .map(|entity| entity.id)
            .collect();
        for id in ids {
            let Some(index) = self.entity_index(id) else {
                continue;
            };
            let Some(jump) = self.entities[index].jump.clone() else {
                continue;
            };
            let total = jump.end_tick.saturating_sub(jump.start_tick).max(1);
            let done = tick.saturating_sub(jump.start_tick).min(total);
            let progress = done as f32 / total as f32;
            let entity = &mut self.entities[index];
            entity.x = jump.sx + (jump.tx - jump.sx) * progress;
            entity.y = jump.sy + (jump.ty - jump.sy) * progress;
            entity.moving = true;
            entity.anim = royale_shared::ANIM_WALK;
            if progress >= 1.0 {
                self.entities[index].jump = None;
                self.entities[index].x = jump.tx;
                self.entities[index].y = jump.ty;
                let side = self.entities[index].side;
                let source = self.entities[index].id;
                self.area_damage(
                    jump.tx,
                    jump.ty,
                    jump.radius,
                    jump.damage,
                    side,
                    Some(source),
                    TargetMask(
                        TargetMask::GROUND
                            | TargetMask::AIR
                            | TargetMask::BUILDINGS
                            | TargetMask::TOWERS,
                    ),
                    0.35,
                    &[],
                    true,
                );
                self.entities[index].attack_cd = 400;
                self.entities[index].anim = royale_shared::ANIM_ATTACK;
            }
        }
    }

    pub fn update_dashes(&mut self) {
        let tick = self.tick;
        let ids: Vec<u32> = self
            .entities
            .iter()
            .filter(|entity| {
                entity.dash_end_tick > tick && entity.dash_target.is_some() && !entity.is_dead()
            })
            .map(|entity| entity.id)
            .collect();
        for id in ids {
            let Some(index) = self.entity_index(id) else {
                continue;
            };
            let target_id = self.entities[index].dash_target.unwrap_or(0);
            let Some(target) = self.entity(target_id) else {
                self.entities[index].dash_end_tick = 0;
                self.entities[index].dash_target = None;
                continue;
            };
            let (tx, ty, target_radius) = (target.x, target.y, target.radius);
            let entity = &self.entities[index];
            let dx = tx - entity.x;
            let dy = ty - entity.y;
            let dist = (dx * dx + dy * dy).sqrt();
            let stop = entity.radius + target_radius + 0.15;
            if dist <= stop {
                let side = entity.side;
                let source = entity.id;
                let bonus = entity
                    .skills
                    .iter()
                    .find_map(|skill| match skill {
                        SkillRuntime::Dash(dash) => Some(dash.bonus_damage),
                        _ => None,
                    })
                    .unwrap_or(0);
                self.deal_damage(
                    target_id,
                    bonus,
                    side,
                    Some(source),
                    Some((entity.x, entity.y)),
                    true,
                );
                self.apply_on_hit_effects(source, target_id, side);
                self.entities[index].dash_end_tick = 0;
                self.entities[index].dash_target = None;
                self.entities[index].attack_cd = 500;
                self.entities[index].anim = royale_shared::ANIM_ATTACK;
                continue;
            }
            let speed = entity.base_speed.max(0.2) * 3.4 * DT;
            let step = speed.min(dist - stop);
            let entity = &mut self.entities[index];
            entity.x += dx / dist * step;
            entity.y += dy / dist * step;
            entity.facing = if dx >= 0.0 { 1 } else { -1 };
            entity.moving = true;
            entity.anim = royale_shared::ANIM_WALK;
            if entity.target != Some(target_id) {
                entity.target = Some(target_id);
            }
        }
    }

    pub fn decrement_timers(&mut self) {
        let frame = MS_PER_TICK as i32;
        for entity in &mut self.entities {
            entity.attack_cd = (entity.attack_cd - frame).max(0);
            entity.dash_timer = (entity.dash_timer - frame).max(0);
            entity.jump_cd = (entity.jump_cd - frame).max(0);
            entity.life_ms = if entity.life_ms > 0 {
                (entity.life_ms - frame).max(0)
            } else {
                entity.life_ms
            };
            if let Some(invis) = &mut entity.invis {
                if !invis.invisible {
                    invis.left_ms -= frame;
                    if invis.left_ms <= 0 {
                        invis.invisible = true;
                    }
                }
            }
            if entity.burrow_left > 0 {
                entity.burrow_left = (entity.burrow_left - frame).max(0);
                if entity.burrow_left == 0 {
                    entity.burrowed = false;
                }
            }
        }
    }

    pub fn resolve_collisions(&mut self) {
        let troop_indices: Vec<usize> = self
            .entities
            .iter()
            .enumerate()
            .filter(|(_, entity)| {
                entity.kind == EntityKind::Troop && !entity.is_dead() && !entity.is_tower
            })
            .map(|(index, _)| index)
            .collect();

        // Empurrão entre unidades.
        let mut deltas = vec![(0.0f32, 0.0f32); self.entities.len()];
        for a in 0..troop_indices.len() {
            for b in (a + 1)..troop_indices.len() {
                let ia = troop_indices[a];
                let ib = troop_indices[b];
                let ea = &self.entities[ia];
                let eb = &self.entities[ib];
                if ea.flying != eb.flying {
                    continue;
                }
                let dx = eb.x - ea.x;
                let dy = eb.y - ea.y;
                let dist = (dx * dx + dy * dy).sqrt();
                let min_dist = ea.radius + eb.radius;
                if dist >= min_dist || dist < 0.0001 {
                    continue;
                }
                let overlap = (min_dist - dist) * 0.5;
                let ux = dx / dist;
                let uy = dy / dist;
                let total_mass = (ea.mass + eb.mass).max(0.1);
                let wa = eb.mass / total_mass;
                let wb = ea.mass / total_mass;
                deltas[ia].0 -= ux * overlap * wa * 2.0;
                deltas[ia].1 -= uy * overlap * wa * 2.0;
                deltas[ib].0 += ux * overlap * wb * 2.0;
                deltas[ib].1 += uy * overlap * wb * 2.0;
            }
        }
        for &index in &troop_indices {
            let (dx, dy) = deltas[index];
            if dx == 0.0 && dy == 0.0 {
                continue;
            }
            let entity = &mut self.entities[index];
            entity.x += dx;
            entity.y += dy;
        }

        // Empurrão contra construções e torres (estáticas).
        let statics: Vec<(f32, f32, f32, usize)> = self
            .entities
            .iter()
            .enumerate()
            .filter(|(_, entity)| {
                (entity.kind == EntityKind::Building || entity.kind == EntityKind::Tower)
                    && !entity.is_dead()
            })
            .map(|(index, entity)| (entity.x, entity.y, entity.radius, index))
            .collect();
        for index in &troop_indices {
            for (sx, sy, sr, _) in &statics {
                let entity = &mut self.entities[*index];
                let dx = entity.x - sx;
                let dy = entity.y - sy;
                let dist = (dx * dx + dy * dy).sqrt();
                let min_dist = entity.radius + sr;
                if dist >= min_dist || dist < 0.0001 {
                    continue;
                }
                let push = min_dist - dist;
                entity.x += dx / dist * push;
                entity.y += dy / dist * push;
            }
        }
    }

    pub fn apply_environment_constraints(&mut self) {
        let width = self.arena.width;
        let height = self.arena.height;
        let river_top = self.arena.river_top;
        let river_bottom = self.arena.river_bottom;
        let arena = self.arena.clone();
        for entity in &mut self.entities {
            if entity.kind == EntityKind::Tower {
                continue;
            }
            if entity.kind == EntityKind::Troop {
                entity.x = clamp(entity.x, entity.radius, width - entity.radius);
                entity.y = clamp(entity.y, entity.radius, height - entity.radius);
            }
            if entity.flying {
                continue;
            }
            if entity.kind == EntityKind::Troop && in_river(&arena, entity.y) {
                if on_bridge(&arena, entity.x) {
                    let bridge = nearest_bridge(&arena, entity.x, entity.x);
                    let (left, right) = crate::util::bridge_bounds(&arena, bridge);
                    entity.x = clamp(entity.x, left + entity.radius, right - entity.radius);
                } else {
                    let mid = (river_top + river_bottom) * 0.5;
                    entity.y = if entity.y < mid {
                        river_top - entity.radius - 0.01
                    } else {
                        river_bottom + entity.radius + 0.01
                    };
                }
            }
        }
    }

    fn update_tower_facing(&mut self) {
        let updates: Vec<(usize, f32)> = self
            .entities
            .iter()
            .enumerate()
            .filter_map(|(index, entity)| {
                if !entity.is_tower {
                    return None;
                }
                let target = entity.target.and_then(|id| self.entity(id))?;
                Some((index, target.x - entity.x))
            })
            .collect();
        for (index, dx) in updates {
            if dx.abs() > 0.05 {
                self.entities[index].facing = if dx > 0.0 { 1 } else { -1 };
            }
        }
    }
}
