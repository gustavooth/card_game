use royale_shared::{ANIM_ATTACK, ANIM_IDLE, Side};

use crate::Match;
use crate::types::*;
use crate::util::{MS_PER_TICK, distance};

impl Match {
    pub fn update_combat(&mut self) {
        let ids: Vec<u32> = self
            .entities
            .iter()
            .filter(|entity| entity.attack.is_some() && !entity.is_dead())
            .map(|entity| entity.id)
            .collect();

        for id in ids {
            let Some(index) = self.entity_index(id) else {
                continue;
            };
            if self.entities[index].is_stunned() {
                self.entities[index].windup = 0;
                continue;
            }
            if self.entities[index].is_tower && !self.entities[index].active {
                continue;
            }
            let Some(target_id) = self.entities[index].target else {
                self.entities[index].windup = 0;
                continue;
            };
            let Some(target) = self.entity(target_id) else {
                self.entities[index].windup = 0;
                self.entities[index].target = None;
                continue;
            };
            if target.is_dead() || target.side == self.entities[index].side {
                self.entities[index].windup = 0;
                self.entities[index].target = None;
                continue;
            }
            let attack = match self.entities[index].attack.clone() {
                Some(attack) => attack,
                None => continue,
            };
            let in_range = distance(
                self.entities[index].x,
                self.entities[index].y,
                target.x,
                target.y,
            ) <= attack.range + target.radius + self.entities[index].radius + 0.05;

            if self.entities[index].windup > 0 {
                if self.entities[index].windup_target != Some(target_id) {
                    self.entities[index].windup = 0;
                    continue;
                }
                self.entities[index].windup -= MS_PER_TICK as i32;
                if self.entities[index].windup <= 0 {
                    self.entities[index].windup = 0;
                    self.execute_attack(index, target_id);
                }
                continue;
            }

            if in_range && self.entities[index].attack_cd <= 0 && !self.entities[index].is_snared()
            {
                if attack.windup_ms > 0 {
                    self.entities[index].windup = attack.windup_ms as i32;
                    self.entities[index].windup_target = Some(target_id);
                    self.entities[index].anim = ANIM_ATTACK;
                } else {
                    self.execute_attack(index, target_id);
                }
            } else if !in_range {
                self.entities[index].windup = 0;
            }
        }

        self.update_projectiles();
    }

    fn execute_attack(&mut self, index: usize, target_id: u32) {
        if self.entities[index].is_dead() {
            return;
        }
        let Some(target) = self.entity(target_id) else {
            return;
        };
        if target.is_dead() {
            return;
        }
        let target_x = target.x;
        let target_y = target.y;
        let attacker = &self.entities[index];
        let side = attacker.side;
        let attacker_id = attacker.id;
        let ax = attacker.x;
        let ay = attacker.y;
        let mask = attacker.targets;
        let attack = match attacker.attack.clone() {
            Some(attack) => attack,
            None => return,
        };
        let attack_mult = attacker.attack_mult();
        let on_hit: Vec<String> = attacker
            .skills
            .iter()
            .find_map(|skill| match skill {
                SkillRuntime::OnHitEffects(effects) => Some(effects.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let charge = attacker.skills.iter().find_map(|skill| match skill {
            SkillRuntime::Charge(charge) => Some(charge.clone()),
            _ => None,
        });
        let ramp = attacker.skills.iter().find_map(|skill| match skill {
            SkillRuntime::Ramp(ramp) => Some(ramp.clone()),
            _ => None,
        });
        let charged = attacker.skills.iter().find_map(|skill| match skill {
            SkillRuntime::ChargedShot(shot) => Some(shot.clone()),
            _ => None,
        });
        let self_destruct = attacker.self_destruct.clone();
        let heal_on_hit = attacker.heal_on_hit;
        let charged_ready = charged
            .as_ref()
            .is_some_and(|shot| self.entities[index].charged_timer >= shot.charge_ms as i32);

        let mut damage = attack.damage as f32 * attack_mult;
        let mut splash = attack.splash_radius;
        let mut projectile = attack.projectile.clone();
        let chain = attack.chain;
        let mut knockback = attack.knockback;

        if let (Some(shot), true) = (&charged, charged_ready) {
            damage = shot.damage as f32 * attack_mult;
            splash = splash.max(shot.splash_radius);
            if let Some(spec) = shot.projectile.clone() {
                projectile = Some(spec);
            }
            self.entities[index].charged_timer = 0;
        }

        if let Some(charge) = &charge {
            if self.entities[index].charge_ready {
                damage += charge.bonus_damage as f32;
                knockback += charge.knockback;
                self.entities[index].charge_ready = false;
                self.entities[index].charge_moved = 0.0;
            }
        }

        if let Some(ramp) = &ramp {
            if self.entities[index].ramp_target == Some(target_id) {
                self.entities[index].ramp_mult =
                    (self.entities[index].ramp_mult + 0.35).min(ramp.max_mult);
            } else {
                self.entities[index].ramp_target = Some(target_id);
                self.entities[index].ramp_mult = 1.0;
                self.entities[index].ramp_progress = 0;
            }
            damage *= self.entities[index].ramp_mult;
        }

        let damage = damage.round().max(0.0) as i32;
        self.entities[index].attack_cd = attack.cooldown_ms as i32;
        self.entities[index].anim = ANIM_ATTACK;
        self.entities[index].windup_target = None;

        if let Some(invis) = &mut self.entities[index].invis {
            invis.invisible = false;
            invis.left_ms = invis.visible_ms as i32;
        }

        if let Some(spec) = projectile {
            let shots = attack.multi_shot.max(1);
            for shot_index in 0..shots {
                let spread = (shot_index as f32 - (shots as f32 - 1.0) * 0.5) * 0.18;
                self.spawn_projectile(
                    side,
                    Some(attacker_id),
                    ax,
                    ay,
                    target_x + spread,
                    target_y,
                    Some(target_id),
                    spec.clone(),
                    damage,
                    splash,
                    attack.pierce,
                    chain,
                    knockback,
                    &on_hit,
                    mask,
                );
            }
        } else {
            let applied = if attack.multi_shot > 1 {
                (damage as f32 * attack.multi_shot as f32).round() as i32
            } else {
                damage
            };
            if splash > 0.0 {
                self.area_damage(
                    target_x,
                    target_y,
                    splash,
                    applied,
                    side,
                    Some(attacker_id),
                    mask,
                    1.0,
                    &on_hit,
                    true,
                );
                self.knockback(target_id, knockback, side, Some((ax, ay)));
            } else {
                self.deal_damage(
                    target_id,
                    applied,
                    side,
                    Some(attacker_id),
                    Some((ax, ay)),
                    true,
                );
                self.apply_on_hit_effects(attacker_id, target_id, side);
                self.knockback(target_id, knockback, side, Some((ax, ay)));
            }
            if chain > 0 {
                self.chain_damage(
                    target_id,
                    target_x,
                    target_y,
                    side,
                    Some(attacker_id),
                    applied,
                    chain,
                    &on_hit,
                    mask,
                );
            }
        }

        if heal_on_hit > 0 {
            self.heal_entity(attacker_id, heal_on_hit);
        }

        if let Some(explode) = self_destruct {
            self.deal_area_raw(
                ax,
                ay,
                explode.radius,
                explode.damage,
                side,
                Some(attacker_id),
                TargetMask(
                    TargetMask::GROUND
                        | TargetMask::AIR
                        | TargetMask::BUILDINGS
                        | TargetMask::TOWERS,
                ),
                0.6,
                &explode.effects,
            );
            self.mark_dead(attacker_id);
        }
    }

    pub fn apply_on_hit_effects(&mut self, attacker_id: u32, target_id: u32, side: Side) {
        let effects: Vec<String> = self
            .entity(attacker_id)
            .map(|entity| {
                entity
                    .skills
                    .iter()
                    .filter_map(|skill| match skill {
                        SkillRuntime::OnHitEffects(effects) => Some(effects.clone()),
                        _ => None,
                    })
                    .flatten()
                    .collect()
            })
            .unwrap_or_default();
        for effect in effects {
            self.apply_effect_preset(target_id, &effect, side, None);
        }
    }

    pub fn trigger_spawn_skills(&mut self, id: u32) {
        let (deploy_damage, x, y, side, mask) = {
            let Some(entity) = self.entity(id) else {
                return;
            };
            let deploy = entity.skills.iter().find_map(|skill| match skill {
                SkillRuntime::DeployDamage {
                    damage,
                    radius,
                    effects,
                } => Some((*damage, *radius, effects.clone())),
                _ => None,
            });
            let Some((damage, radius, effects)) = deploy else {
                return;
            };
            (
                Some((damage, radius, effects)),
                entity.x,
                entity.y,
                entity.side,
                entity.targets,
            )
        };
        if let Some((damage, radius, effects)) = deploy_damage {
            self.deal_area_raw(x, y, radius, damage, side, Some(id), mask, 1.0, &effects);
        }
    }

    fn deal_area_raw(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        damage: i32,
        side: Side,
        source: Option<u32>,
        mask: TargetMask,
        tower_mult: f32,
        effects: &[String],
    ) {
        self.area_damage(
            x, y, radius, damage, side, source, mask, tower_mult, effects, true,
        );
    }

    fn chain_damage(
        &mut self,
        primary_id: u32,
        x: f32,
        y: f32,
        side: Side,
        source: Option<u32>,
        damage: i32,
        chain: u32,
        effects: &[String],
        mask: TargetMask,
    ) {
        let mut previous = primary_id;
        let mut position = (x, y);
        let mut current_damage = damage;
        let mut visited = vec![primary_id];
        for jump in 0..chain {
            let next = self
                .entities
                .iter()
                .filter(|entity| {
                    !entity.is_dead()
                        && !entity.is_tower
                        && entity.side != side
                        && !visited.contains(&entity.id)
                        && mask.can_hit(entity.kind, entity.flying)
                        && distance(position.0, position.1, entity.x, entity.y)
                            <= 2.6 + entity.radius
                })
                .min_by(|a, b| {
                    distance(position.0, position.1, a.x, a.y)
                        .partial_cmp(&distance(position.0, position.1, b.x, b.y))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|entity| (entity.id, entity.x, entity.y));
            let Some((next_id, nx, ny)) = next else {
                break;
            };
            current_damage = ((current_damage as f32) * 0.7).round() as i32;
            self.deal_damage(next_id, current_damage, side, source, Some(position), true);
            for effect in effects {
                self.apply_effect_preset(next_id, effect, side, Some(position));
            }
            self.emit(royale_shared::EVENT_HIT, side, nx, ny, 0);
            visited.push(next_id);
            previous = next_id;
            position = (nx, ny);
            if jump + 1 >= chain {
                break;
            }
        }
        let _ = previous;
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_projectile(
        &mut self,
        side: Side,
        source: Option<u32>,
        from_x: f32,
        from_y: f32,
        tx: f32,
        ty: f32,
        target: Option<u32>,
        spec: ProjectileSpec,
        damage: i32,
        splash_radius: f32,
        pierce: bool,
        chain: u32,
        knockback: f32,
        effects: &[String],
        mask: TargetMask,
    ) {
        let dx = tx - from_x;
        let dy = ty - from_y;
        let length = (dx * dx + dy * dy).sqrt().max(0.001);
        let distance_left = if pierce { length + 3.5 } else { length + 0.8 };
        let projectile_id = self.next_entity_id();
        self.projectiles.push(Projectile {
            id: projectile_id,
            side,
            x: from_x,
            y: from_y,
            dir_x: dx / length,
            dir_y: dy / length,
            target,
            tx,
            ty,
            speed: spec.speed,
            radius: spec.radius,
            homing: spec.homing && !pierce,
            visual_idx: spec.visual_idx,
            damage,
            splash_radius,
            pierce,
            chain,
            knockback,
            effects: effects.to_vec(),
            source_id: source,
            hit_ids: Vec::new(),
            distance_left,
            traveled: 0.0,
            mask,
        });
    }

    fn update_projectiles(&mut self) {
        let mut remove: Vec<usize> = Vec::new();
        let count = self.projectiles.len();
        for index in 0..count {
            if remove.contains(&index) {
                continue;
            }
            let projectile = self.projectiles[index].clone();
            let mut x = projectile.x;
            let mut y = projectile.y;
            let mut tx = projectile.tx;
            let mut ty = projectile.ty;
            let mut dir_x = projectile.dir_x;
            let mut dir_y = projectile.dir_y;

            if projectile.homing {
                if let Some(target_id) = projectile.target {
                    if let Some(target) = self.entity(target_id) {
                        if !target.is_dead() {
                            tx = target.x;
                            ty = target.y;
                        }
                    }
                }
            }
            let dx = tx - x;
            let dy = ty - y;
            let length = (dx * dx + dy * dy).sqrt();
            if length > 0.001 {
                dir_x = dx / length;
                dir_y = dy / length;
            }
            let step = (projectile.speed * crate::util::DT).min(projectile.distance_left);
            x += dir_x * step;
            y += dir_y * step;

            if projectile.pierce {
                let hits: Vec<(u32, f32, f32)> = self
                    .entities
                    .iter()
                    .filter(|entity| {
                        !entity.is_dead()
                            && entity.side != projectile.side
                            && !projectile.hit_ids.contains(&entity.id)
                            && projectile.mask.can_hit(entity.kind, entity.flying)
                            && distance(x, y, entity.x, entity.y)
                                <= projectile.radius + entity.radius + 0.12
                    })
                    .map(|entity| (entity.id, entity.x, entity.y))
                    .collect();
                let mut hit_something = false;
                for (id, ex, ey) in hits {
                    self.deal_damage(
                        id,
                        projectile.damage,
                        projectile.side,
                        projectile.source_id,
                        Some((x, y)),
                        true,
                    );
                    for effect in &projectile.effects {
                        self.apply_effect_preset(id, effect, projectile.side, Some((x, y)));
                    }
                    self.emit(royale_shared::EVENT_HIT, projectile.side, ex, ey, 0);
                    if let Some(projectile) = self.projectiles.get_mut(index) {
                        projectile.hit_ids.push(id);
                    }
                    hit_something = true;
                }
                if hit_something && projectile.chain > 0 {
                    if let Some((first_id, _, _)) =
                        self.projectiles[index].hit_ids.first().and_then(|id| {
                            self.entity(*id)
                                .map(|entity| (entity.id, entity.x, entity.y))
                        })
                    {
                        self.chain_damage(
                            first_id,
                            x,
                            y,
                            projectile.side,
                            projectile.source_id,
                            projectile.damage,
                            projectile.chain,
                            &projectile.effects,
                            projectile.mask,
                        );
                    }
                }
                if let Some(projectile) = self.projectiles.get_mut(index) {
                    projectile.x = x;
                    projectile.y = y;
                    projectile.dir_x = dir_x;
                    projectile.dir_y = dir_y;
                    projectile.tx = tx;
                    projectile.ty = ty;
                    projectile.distance_left -= step;
                    projectile.traveled += step;
                    if projectile.distance_left <= 0.01 {
                        remove.push(index);
                    }
                }
                continue;
            }

            let hit_target = projectile
                .target
                .and_then(|id| self.entity(id))
                .filter(|target| {
                    !target.is_dead()
                        && distance(x, y, target.x, target.y)
                            <= projectile.radius + target.radius + 0.16
                })
                .map(|target| (target.id, target.x, target.y));
            let reached_end = length <= 0.001 || projectile.distance_left <= step + 0.01;

            if hit_target.is_some() || reached_end {
                let (impact_x, impact_y) = hit_target.map(|(_, ex, ey)| (ex, ey)).unwrap_or((x, y));
                if projectile.splash_radius > 0.0 {
                    self.area_damage(
                        impact_x,
                        impact_y,
                        projectile.splash_radius,
                        projectile.damage,
                        projectile.side,
                        projectile.source_id,
                        projectile.mask,
                        1.0,
                        &projectile.effects,
                        true,
                    );
                } else if let Some((id, _, _)) = hit_target {
                    self.deal_damage(
                        id,
                        projectile.damage,
                        projectile.side,
                        projectile.source_id,
                        Some((impact_x, impact_y)),
                        true,
                    );
                    for effect in &projectile.effects {
                        self.apply_effect_preset(
                            id,
                            effect,
                            projectile.side,
                            Some((impact_x, impact_y)),
                        );
                    }
                }
                if let Some((id, ex, ey)) = hit_target {
                    self.knockback(
                        id,
                        projectile.knockback,
                        projectile.side,
                        Some((impact_x, impact_y)),
                    );
                    if projectile.chain > 0 {
                        self.chain_damage(
                            id,
                            impact_x,
                            impact_y,
                            projectile.side,
                            projectile.source_id,
                            projectile.damage,
                            projectile.chain,
                            &projectile.effects,
                            projectile.mask,
                        );
                    }
                    self.emit(royale_shared::EVENT_HIT, projectile.side, ex, ey, 0);
                }
                remove.push(index);
                continue;
            }

            if let Some(projectile) = self.projectiles.get_mut(index) {
                projectile.x = x;
                projectile.y = y;
                projectile.dir_x = dir_x;
                projectile.dir_y = dir_y;
                projectile.distance_left -= step;
                projectile.traveled += step;
            }
        }
        remove.sort_unstable();
        remove.dedup();
        for index in remove.into_iter().rev() {
            if index < self.projectiles.len() {
                self.projectiles.remove(index);
            }
        }
    }

    pub fn reset_attack_anim(&mut self) {
        for entity in &mut self.entities {
            if entity.windup == 0 && !entity.moving && entity.anim == ANIM_ATTACK {
                entity.anim = ANIM_IDLE;
            }
        }
    }
}
