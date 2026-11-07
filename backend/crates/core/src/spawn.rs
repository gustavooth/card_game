use royale_defs::{BuildingDef, Definitions, SkillKind, TargetKind, TowerDef, TroopDef};
use royale_shared::{ANIM_IDLE, Side};

use crate::Match;
use crate::types::*;
use crate::util::clamp;

pub fn resolve_projectile(
    defs: &Definitions,
    catalog: &royale_defs::Catalog,
    slug: &str,
) -> Option<ProjectileSpec> {
    let def = defs.projectiles.get(slug)?;
    Some(ProjectileSpec {
        speed: if def.speed > 0.0 { def.speed } else { 8.0 },
        radius: def.radius.max(0.05),
        homing: def.homing,
        visual_idx: catalog.projectile_index(slug).unwrap_or(0),
    })
}

pub struct SkillBuild {
    pub skills: Vec<SkillRuntime>,
    pub shield: i32,
    pub deploy_anywhere: bool,
    pub self_destruct: Option<SkillSelfDestruct>,
    pub transform_on_death: Option<(String, i32)>,
    pub rebirth: Option<SkillRebirth>,
    pub death_effects: Option<(Vec<String>, f32)>,
    pub heal_on_hit: i32,
}

impl Default for SkillBuild {
    fn default() -> Self {
        Self {
            skills: Vec::new(),
            shield: 0,
            deploy_anywhere: false,
            self_destruct: None,
            transform_on_death: None,
            rebirth: None,
            death_effects: None,
            heal_on_hit: 0,
        }
    }
}

pub fn build_skills(
    defs: &Definitions,
    catalog: &royale_defs::Catalog,
    skill_refs: &[royale_defs::SkillRef],
) -> SkillBuild {
    let mut build = SkillBuild::default();
    for reference in skill_refs {
        let Some(def) = defs.skills.get(&reference.skill) else {
            continue;
        };
        let params = &def.params;
        match def.kind {
            SkillKind::Charge => build.skills.push(SkillRuntime::Charge(SkillCharge {
                min_distance: params.min_distance.unwrap_or(1.2),
                speed_mult: params.speed_mult.unwrap_or(1.7),
                bonus_damage: params.bonus_damage.unwrap_or(100),
                reset_ms: params.reset_ms.unwrap_or(1500),
                knockback: params.knockback.unwrap_or(0.0),
            })),
            SkillKind::Dash => build.skills.push(SkillRuntime::Dash(SkillDash {
                trigger_range: params.trigger_range.unwrap_or(3.0),
                speed_mult: params.speed_mult.unwrap_or(2.4),
                duration_ms: params.duration_ms.unwrap_or(420),
                bonus_damage: params.bonus_damage.unwrap_or(150),
            })),
            SkillKind::Jump => build.skills.push(SkillRuntime::Jump(SkillJump {
                trigger_range: params.trigger_range.unwrap_or(5.0),
                windup_ms: params.windup_ms.unwrap_or(500),
                damage: params.damage.unwrap_or(300),
                radius: params.radius.unwrap_or(2.0),
            })),
            SkillKind::OnHitEffects => build
                .skills
                .push(SkillRuntime::OnHitEffects(params.effects.clone())),
            SkillKind::DeployDamage => {
                build.skills.push(SkillRuntime::DeployDamage {
                    damage: params.damage.unwrap_or(0),
                    radius: params.radius.unwrap_or(1.5),
                    effects: params.effects.clone(),
                });
            }
            SkillKind::Invisibility => {
                build.skills.push(SkillRuntime::Invisibility {
                    period_ms: params.period_ms.unwrap_or(1600),
                    visible_ms: params.visible_ms.unwrap_or(900),
                });
            }
            SkillKind::Burrow => {
                let burrow = SkillBurrow {
                    travel_ms: params.travel_ms.unwrap_or(900),
                    speed_mult: params.speed_mult.unwrap_or(1.8),
                    deploy_anywhere: params.deploy_anywhere.unwrap_or(true),
                };
                if burrow.deploy_anywhere {
                    build.deploy_anywhere = true;
                }
                build.skills.push(SkillRuntime::Burrow(burrow));
            }
            SkillKind::Hook => build.skills.push(SkillRuntime::Hook(SkillHook {
                range: params.range.unwrap_or(5.0),
                pull_distance: params.pull_distance.unwrap_or(2.2),
                damage: params.damage.unwrap_or(80),
                cooldown_ms: params.cooldown_ms.unwrap_or(6000),
            })),
            SkillKind::Rebirth => {
                let rebirth = SkillRebirth {
                    health: params.health.unwrap_or(800),
                    troop: params.troop.clone().unwrap_or_default(),
                    delay_ms: params.delay_ms.unwrap_or(1200),
                };
                build.rebirth = Some(rebirth.clone());
                build.skills.push(SkillRuntime::Rebirth(rebirth));
            }
            SkillKind::SpawnPeriodic => {
                build
                    .skills
                    .push(SkillRuntime::SpawnPeriodic(SkillSpawnPeriodic {
                        troop: params.troop.clone().unwrap_or_default(),
                        count: params.count.unwrap_or(1),
                        interval_ms: params.interval_ms.unwrap_or(5000),
                        first_ms: params.first_ms.unwrap_or(1500),
                        max_alive: params.max_alive.unwrap_or(6),
                    }));
            }
            SkillKind::Aura => build.skills.push(SkillRuntime::Aura(SkillAura {
                radius: params.radius.unwrap_or(3.0),
                dps: params.dps.unwrap_or(0.0),
                heal_per_sec: params.heal_per_sec.unwrap_or(0.0),
                effects: params.effects.clone(),
                affects_allies: params.affects_allies.unwrap_or(false),
                affects_enemies: params.affects_enemies.unwrap_or(true),
            })),
            SkillKind::Reflect => build.skills.push(SkillRuntime::Reflect {
                percent: params.percent.unwrap_or(0.35),
                max_range: params.max_range.unwrap_or(4.0),
            }),
            SkillKind::Ramp => build.skills.push(SkillRuntime::Ramp(SkillRamp {
                max_mult: params.max_mult.unwrap_or(4.0),
                interval_ms: params.interval_ms.unwrap_or(400),
                reset_ms: params.reset_ms.unwrap_or(1800),
            })),
            SkillKind::Shield => {
                build.shield += params.health.unwrap_or(200);
            }
            SkillKind::DeployAnywhere => {
                build.deploy_anywhere = true;
                build.skills.push(SkillRuntime::DeployAnywhere);
            }
            SkillKind::ElixirGenerate => {
                build
                    .skills
                    .push(SkillRuntime::ElixirGenerate(SkillElixirGenerate {
                        amount: params.amount.unwrap_or(1.0),
                        interval_ms: params.interval_ms.unwrap_or(8400),
                    }));
            }
            SkillKind::ElixirOnDeath => build.skills.push(SkillRuntime::ElixirOnDeath {
                amount: params.amount.unwrap_or(1.0),
            }),
            SkillKind::SelfDestruct => {
                let spec = SkillSelfDestruct {
                    damage: params.damage.unwrap_or(300),
                    radius: params.radius.unwrap_or(1.5),
                    effects: params.effects.clone(),
                };
                build.self_destruct = Some(spec.clone());
                build.skills.push(SkillRuntime::SelfDestruct(spec));
            }
            SkillKind::TransformOnDeath => {
                let transform = (
                    params.building.clone().unwrap_or_default(),
                    params.health.unwrap_or(600),
                );
                build.transform_on_death = Some(transform.clone());
                build.skills.push(SkillRuntime::TransformOnDeath {
                    building: transform.0,
                    health: transform.1,
                });
            }
            SkillKind::HealOnHit => {
                let amount = params.amount.unwrap_or(40.0) as i32;
                build.heal_on_hit = amount;
                build.skills.push(SkillRuntime::HealOnHit { amount });
            }
            SkillKind::ChargedShot => {
                build
                    .skills
                    .push(SkillRuntime::ChargedShot(SkillChargedShot {
                        charge_ms: params.charge_ms.unwrap_or(4000),
                        damage: params.damage.unwrap_or(1000),
                        splash_radius: params.splash_radius.unwrap_or(1.6),
                        projectile: params
                            .projectile
                            .as_deref()
                            .and_then(|slug| resolve_projectile(defs, catalog, slug)),
                    }));
            }
            SkillKind::DeathEffects => {
                let spec = (params.effects.clone(), params.radius.unwrap_or(2.0));
                build.death_effects = Some(spec.clone());
                build.skills.push(SkillRuntime::DeathEffects {
                    effects: spec.0,
                    radius: spec.1,
                });
            }
            SkillKind::Tornado => {}
        }
    }
    build
}

pub fn troop_targets(def: &TroopDef) -> TargetMask {
    TargetMask::from_kinds(&def.targets)
}

pub fn building_targets(def: &BuildingDef) -> TargetMask {
    TargetMask::from_kinds(&def.targets)
}

pub fn tower_targets() -> TargetMask {
    TargetMask::from_kinds(&[
        TargetKind::Ground,
        TargetKind::Air,
        TargetKind::Buildings,
        TargetKind::Towers,
    ])
}

impl Match {
    pub fn next_entity_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    pub fn defs(&self) -> &Definitions {
        &self.defs
    }

    pub fn spawn_towers(&mut self) {
        let arena = self.arena.clone();
        let tower_defs = self.defs.towers.clone();
        let princess = tower_defs.get("princess_tower").cloned();
        let king = tower_defs.get("king_tower").cloned();
        if let (Some(princess), Some(king)) = (princess, king) {
            self.spawn_tower_side(Side::Red, &arena.tower_positions.top, &princess, &king);
            self.spawn_tower_side(Side::Blue, &arena.tower_positions.bottom, &princess, &king);
        }
    }

    fn spawn_tower_side(
        &mut self,
        side: Side,
        positions: &royale_defs::TowerPositions,
        princess: &TowerDef,
        king: &TowerDef,
    ) {
        for (lane, pos) in positions.princess.iter().take(2).enumerate() {
            let entity = self.make_tower(
                side,
                princess,
                pos[0],
                pos[1],
                TowerRole::Princess,
                lane as u8,
            );
            self.entities.push(entity);
        }
        let entity = self.make_tower(
            side,
            king,
            positions.king[0],
            positions.king[1],
            TowerRole::King,
            9,
        );
        self.entities.push(entity);
    }

    fn make_tower(
        &mut self,
        side: Side,
        def: &TowerDef,
        x: f32,
        y: f32,
        role: TowerRole,
        lane: u8,
    ) -> Entity {
        let attack = AttackRuntime::from_def(
            &def.attack,
            def.attack
                .projectile
                .as_deref()
                .and_then(|slug| resolve_projectile(&self.defs, &self.catalog, slug)),
        );
        Entity {
            id: self.next_entity_id(),
            side,
            kind: EntityKind::Tower,
            slug: def.slug.clone(),
            card_slug: String::new(),
            hp: def.health,
            max_hp: def.health,
            x,
            y,
            radius: def.radius,
            mass: 1000.0,
            base_speed: 0.0,
            targets: tower_targets(),
            sight_range: attack.range,
            attack: Some(attack),
            active: role != TowerRole::King,
            is_tower: true,
            tower_role: role,
            lane,
            facing: if side == Side::Blue { -1 } else { 1 },
            life_ms: -1,
            ..Entity::default()
        }
    }

    pub fn spawn_troop(
        &mut self,
        side: Side,
        card_slug: &str,
        troop_slug: &str,
        x: f32,
        y: f32,
        health_override: Option<i32>,
        no_rebirth: bool,
        producer: Option<u32>,
    ) -> Option<u32> {
        let def = self.defs.troops.get(troop_slug)?.clone();
        let build = build_skills(&self.defs, &self.catalog, &def.skills);
        let attack = AttackRuntime::from_def(
            &def.attack,
            def.attack
                .projectile
                .as_deref()
                .and_then(|slug| resolve_projectile(&self.defs, &self.catalog, slug)),
        );
        let rarity = self.defs.card(card_slug).map(|card| card.rarity);
        let max_hp = health_override.unwrap_or(def.health).max(1);
        let mut entity = Entity {
            id: self.next_entity_id(),
            side,
            kind: EntityKind::Troop,
            slug: troop_slug.to_string(),
            card_slug: card_slug.to_string(),
            rarity,
            hp: max_hp,
            max_hp,
            shield: build.shield,
            x,
            y,
            radius: def.radius,
            mass: def.mass,
            flying: def.flying,
            base_speed: def.speed,
            targets: troop_targets(&def),
            sight_range: def.sight_range,
            attack: Some(attack),
            skills: build.skills,
            attack_cd: def.attack.first_hit_ms as i32,
            facing: if side == Side::Blue { -1 } else { 1 },
            life_ms: -1,
            heal_on_hit: build.heal_on_hit,
            self_destruct: build.self_destruct,
            transform_on_death: build.transform_on_death,
            rebirth: if no_rebirth { None } else { build.rebirth },
            death_effects: build.death_effects,
            death_spec: DeathSpec::from_def(&def.death),
            deploy_anywhere: build.deploy_anywhere,
            no_rebirth,
            spawn_serial: producer.unwrap_or(0),
            ..Entity::default()
        };
        entity.anim = ANIM_IDLE;
        entity.spawn_timer = 0;
        entity.hook_timer = 0;
        entity.elixir_timer = 0;
        entity.charged_timer = 0;
        entity.invis = entity.skills.iter().find_map(|skill| match skill {
            SkillRuntime::Invisibility {
                period_ms,
                visible_ms,
            } => Some(InvisState {
                invisible: true,
                left_ms: 0,
                period_ms: *period_ms,
                visible_ms: *visible_ms,
            }),
            _ => None,
        });
        if let Some(SkillRuntime::Burrow(burrow)) = entity
            .skills
            .iter()
            .find(|skill| matches!(skill, SkillRuntime::Burrow(_)))
            .cloned()
        {
            entity.burrow_left = burrow.travel_ms as i32;
            entity.burrowed = true;
        }
        if let Some(SkillRuntime::SpawnPeriodic(periodic)) = entity
            .skills
            .iter()
            .find(|skill| matches!(skill, SkillRuntime::SpawnPeriodic(_)))
            .cloned()
        {
            entity.spawn_timer = periodic.first_ms as i32;
        }
        if let Some(SkillRuntime::Hook(hook)) = entity
            .skills
            .iter()
            .find(|skill| matches!(skill, SkillRuntime::Hook(_)))
            .cloned()
        {
            entity.hook_timer = hook.cooldown_ms as i32;
        }
        if let Some(SkillRuntime::ChargedShot(shot)) = entity
            .skills
            .iter()
            .find(|skill| matches!(skill, SkillRuntime::ChargedShot(_)))
            .cloned()
        {
            entity.charged_timer = shot.charge_ms as i32;
        }
        let id = entity.id;
        self.entities.push(entity);
        self.trigger_spawn_skills(id);
        Some(id)
    }

    pub fn spawn_troop_formation(
        &mut self,
        side: Side,
        card_slug: &str,
        troop_slug: &str,
        count: u32,
        x: f32,
        y: f32,
        health_override: Option<i32>,
        no_rebirth: bool,
    ) {
        let positions = formation_positions(count, x, y);
        for (px, py) in positions {
            let (nx, ny) = self.clamp_spawn(side, px, py);
            self.spawn_troop(
                side,
                card_slug,
                troop_slug,
                nx,
                ny,
                health_override,
                no_rebirth,
                None,
            );
        }
    }

    pub fn spawn_building(
        &mut self,
        side: Side,
        card_slug: &str,
        building_slug: &str,
        x: f32,
        y: f32,
        health_override: Option<i32>,
    ) -> Option<u32> {
        let def = self.defs.buildings.get(building_slug)?.clone();
        let build = build_skills(&self.defs, &self.catalog, &def.skills);
        let attack = def
            .attack
            .as_ref()
            .filter(|attack| attack.damage > 0)
            .map(|attack| {
                AttackRuntime::from_def(
                    attack,
                    attack
                        .projectile
                        .as_deref()
                        .and_then(|slug| resolve_projectile(&self.defs, &self.catalog, slug)),
                )
            });
        let max_hp = health_override.unwrap_or(def.health).max(1);
        let mut entity = Entity {
            id: self.next_entity_id(),
            side,
            kind: EntityKind::Building,
            slug: building_slug.to_string(),
            card_slug: card_slug.to_string(),
            rarity: self.defs.card(card_slug).map(|card| card.rarity),
            hp: max_hp,
            max_hp,
            shield: build.shield,
            x,
            y,
            radius: def.radius,
            mass: 1000.0,
            base_speed: 0.0,
            targets: building_targets(&def),
            sight_range: def.sight_range,
            attack,
            skills: build.skills,
            life_ms: def.lifetime_ms as i32,
            heal_on_hit: build.heal_on_hit,
            self_destruct: build.self_destruct,
            transform_on_death: build.transform_on_death,
            rebirth: build.rebirth,
            death_effects: build.death_effects,
            death_spec: DeathSpec::from_def(&def.death),
            deploy_anywhere: build.deploy_anywhere,
            ..Entity::default()
        };
        entity.facing = if side == Side::Blue { -1 } else { 1 };
        if let Some(SkillRuntime::SpawnPeriodic(periodic)) = entity
            .skills
            .iter()
            .find(|skill| matches!(skill, SkillRuntime::SpawnPeriodic(_)))
            .cloned()
        {
            entity.spawn_timer = periodic.first_ms as i32;
        }
        if let Some(SkillRuntime::ElixirGenerate(generate)) = entity
            .skills
            .iter()
            .find(|skill| matches!(skill, SkillRuntime::ElixirGenerate(_)))
            .cloned()
        {
            entity.elixir_timer = generate.interval_ms as i32;
        }
        if let Some(SkillRuntime::Burrow(burrow)) = entity
            .skills
            .iter()
            .find(|skill| matches!(skill, SkillRuntime::Burrow(_)))
            .cloned()
        {
            entity.burrow_left = burrow.travel_ms as i32;
            entity.burrowed = true;
        }
        let id = entity.id;
        self.entities.push(entity);
        Some(id)
    }

    pub fn process_pending(&mut self) {
        let tick = self.tick;
        let due: Vec<PendingAction> = self
            .pending
            .iter()
            .filter(|action| action.at_tick <= tick)
            .cloned()
            .collect();
        self.pending.retain(|action| action.at_tick > tick);
        for action in due {
            match action.kind {
                PendingKind::SpawnTroop {
                    card_slug,
                    troop_slug,
                    x,
                    y,
                    health,
                    no_rebirth,
                } => {
                    self.spawn_troop_formation(
                        action.side,
                        &card_slug,
                        &troop_slug,
                        1,
                        x,
                        y,
                        health,
                        no_rebirth,
                    );
                }
                PendingKind::SpawnBuilding {
                    card_slug,
                    building_slug,
                    x,
                    y,
                    health,
                } => {
                    self.spawn_building(action.side, &card_slug, &building_slug, x, y, health);
                }
                PendingKind::CastSpell {
                    card_slug,
                    spell_slug,
                    x,
                    y,
                } => {
                    self.cast_spell(action.side, &card_slug, &spell_slug, x, y);
                }
            }
        }
    }

    pub fn clamp_spawn(&mut self, side: Side, x: f32, y: f32) -> (f32, f32) {
        let mut x = clamp(x, 0.5, self.arena.width - 0.5);
        let mut y = clamp(y, 0.5, self.arena.height - 0.5);
        if y > self.arena.river_top && y < self.arena.river_bottom {
            if side == Side::Blue {
                y = self.arena.river_bottom + 0.6;
            } else {
                y = self.arena.river_top - 0.6;
            }
        }
        if y < 0.5 {
            y = 0.5;
        }
        if y > self.arena.height - 0.5 {
            y = self.arena.height - 0.5;
        }
        x += self.rng.range(-0.06, 0.06);
        y += self.rng.range(-0.06, 0.06);
        (x, y)
    }
}

pub fn formation_positions(count: u32, x: f32, y: f32) -> Vec<(f32, f32)> {
    if count <= 1 {
        return vec![(x, y)];
    }
    let cols = count.min(5) as i32;
    let rows = ((count as i32) + cols - 1) / cols;
    let spacing = 0.58;
    let mut out = Vec::with_capacity(count as usize);
    for index in 0..count as i32 {
        let row = index / cols;
        let col = index % cols;
        let row_count = if row == rows - 1 {
            count as i32 - row * cols
        } else {
            cols
        };
        let dx = (col as f32 - (row_count as f32 - 1.0) * 0.5) * spacing;
        let dy = (row as f32 - (rows as f32 - 1.0) * 0.5) * spacing;
        out.push((x + dx, y + dy));
    }
    out
}

/// Exposto para o servidor montar decks padrão.
pub fn default_deck(defs: &Definitions, catalog: &royale_defs::Catalog) -> Vec<String> {
    let fallback = defs.cards.iter().map(|card| card.slug.clone()).take(8);
    let required = crate::spawn::REQUIRED_DECK
        .iter()
        .map(|slug| slug.to_string())
        .collect::<Vec<_>>();
    let mut deck = required
        .into_iter()
        .filter(|slug| catalog.card_index(slug).is_some())
        .collect::<Vec<_>>();
    for slug in fallback {
        if deck.len() >= 8 {
            break;
        }
        if !deck.contains(&slug) {
            deck.push(slug);
        }
    }
    deck.truncate(8);
    deck
}

pub const REQUIRED_DECK: [&str; 8] = [
    "knight",
    "archers",
    "fireball",
    "cannon",
    "skeletons",
    "musketeer",
    "giant",
    "arrows",
];

/// Usado pelo servidor para validar e converter deck de slugs em índices.
pub fn deck_indices(
    defs: &Definitions,
    catalog: &royale_defs::Catalog,
    slugs: &[String],
) -> Result<Vec<u16>, String> {
    if slugs.len() != royale_shared::DECK_SIZE {
        return Err("deck deve ter 8 cartas".into());
    }
    let mut unique = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(8);
    for slug in slugs {
        if !unique.insert(slug.clone()) {
            return Err("deck não pode ter cartas repetidas".into());
        }
        if defs.card(slug).is_none() {
            return Err(format!("carta desconhecida: {slug}"));
        }
        let index = catalog
            .card_index(slug)
            .ok_or_else(|| format!("carta sem índice: {slug}"))?;
        out.push(index);
    }
    Ok(out)
}
