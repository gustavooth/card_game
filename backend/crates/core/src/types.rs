use royale_defs::{AttackDef, DeathDef, Rarity, TargetKind};
use royale_shared::{ANIM_IDLE, Side};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Troop,
    Building,
    Tower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TowerRole {
    None,
    Princess,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Regulation,
    Overtime,
    Ended,
}

impl Phase {
    pub fn code(self) -> u8 {
        match self {
            Phase::Regulation => royale_shared::PHASE_REGULATION,
            Phase::Overtime => royale_shared::PHASE_OVERTIME,
            Phase::Ended => royale_shared::PHASE_ENDED,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndReason {
    KingDestroyed,
    SuddenDeath,
    RegulationCrowns,
    OvertimeTiebreak,
    Draw,
}

impl EndReason {
    pub fn as_str(self) -> &'static str {
        match self {
            EndReason::KingDestroyed => "king_destroyed",
            EndReason::SuddenDeath => "sudden_death",
            EndReason::RegulationCrowns => "regulation_crowns",
            EndReason::OvertimeTiebreak => "overtime_tiebreak",
            EndReason::Draw => "draw",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TargetMask(pub u8);

impl TargetMask {
    pub const GROUND: u8 = 1 << 0;
    pub const AIR: u8 = 1 << 1;
    pub const BUILDINGS: u8 = 1 << 2;
    pub const TOWERS: u8 = 1 << 3;

    pub fn from_kinds(kinds: &[TargetKind]) -> Self {
        let mut bits = 0u8;
        for kind in kinds {
            bits |= match kind {
                TargetKind::Ground => Self::GROUND,
                TargetKind::Air => Self::AIR,
                TargetKind::Buildings => Self::BUILDINGS,
                TargetKind::Towers => Self::TOWERS,
            };
        }
        Self(bits)
    }

    pub fn contains(self, bit: u8) -> bool {
        self.0 & bit != 0
    }

    pub fn can_hit(self, kind: EntityKind, flying: bool) -> bool {
        match kind {
            EntityKind::Troop => {
                if flying {
                    self.contains(Self::AIR)
                } else {
                    self.contains(Self::GROUND)
                }
            }
            EntityKind::Building => self.contains(Self::BUILDINGS),
            EntityKind::Tower => self.contains(Self::TOWERS),
        }
    }

    pub fn hits_any_unit(self) -> bool {
        self.contains(Self::GROUND) || self.contains(Self::AIR)
    }
}

#[derive(Debug, Clone)]
pub struct ProjectileSpec {
    pub speed: f32,
    pub radius: f32,
    pub homing: bool,
    pub visual_idx: u16,
}

#[derive(Debug, Clone)]
pub struct AttackRuntime {
    pub range: f32,
    pub damage: i32,
    pub cooldown_ms: u32,
    pub windup_ms: u32,
    pub first_hit_ms: u32,
    pub splash_radius: f32,
    pub projectile: Option<ProjectileSpec>,
    pub pierce: bool,
    pub chain: u32,
    pub knockback: f32,
    pub multi_shot: u32,
}

impl AttackRuntime {
    pub fn from_def(def: &AttackDef, projectile: Option<ProjectileSpec>) -> Self {
        Self {
            range: def.range,
            damage: def.damage,
            cooldown_ms: def.cooldown_ms,
            windup_ms: def.windup_ms,
            first_hit_ms: def.first_hit_ms.max(100),
            splash_radius: def.splash_radius,
            projectile,
            pierce: def.pierce,
            chain: def.chain,
            knockback: def.knockback,
            multi_shot: def.multi_shot.max(1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillCharge {
    pub min_distance: f32,
    pub speed_mult: f32,
    pub bonus_damage: i32,
    pub reset_ms: u32,
    pub knockback: f32,
}

#[derive(Debug, Clone)]
pub struct SkillDash {
    pub trigger_range: f32,
    pub speed_mult: f32,
    pub duration_ms: u32,
    pub bonus_damage: i32,
}

#[derive(Debug, Clone)]
pub struct SkillJump {
    pub trigger_range: f32,
    pub windup_ms: u32,
    pub damage: i32,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct SkillBurrow {
    pub travel_ms: u32,
    pub speed_mult: f32,
    pub deploy_anywhere: bool,
}

#[derive(Debug, Clone)]
pub struct SkillHook {
    pub range: f32,
    pub pull_distance: f32,
    pub damage: i32,
    pub cooldown_ms: u32,
}

#[derive(Debug, Clone)]
pub struct SkillRebirth {
    pub health: i32,
    pub troop: String,
    pub delay_ms: u32,
}

#[derive(Debug, Clone)]
pub struct SkillSpawnPeriodic {
    pub troop: String,
    pub count: u32,
    pub interval_ms: u32,
    pub first_ms: u32,
    pub max_alive: u32,
}

#[derive(Debug, Clone)]
pub struct SkillAura {
    pub radius: f32,
    pub dps: f32,
    pub heal_per_sec: f32,
    pub effects: Vec<String>,
    pub affects_allies: bool,
    pub affects_enemies: bool,
}

#[derive(Debug, Clone)]
pub struct SkillRamp {
    pub max_mult: f32,
    pub interval_ms: u32,
    pub reset_ms: u32,
}

#[derive(Debug, Clone)]
pub struct SkillElixirGenerate {
    pub amount: f32,
    pub interval_ms: u32,
}

#[derive(Debug, Clone)]
pub struct SkillSelfDestruct {
    pub damage: i32,
    pub radius: f32,
    pub effects: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SkillChargedShot {
    pub charge_ms: u32,
    pub damage: i32,
    pub splash_radius: f32,
    pub projectile: Option<ProjectileSpec>,
}

#[derive(Debug, Clone)]
pub enum SkillRuntime {
    Charge(SkillCharge),
    Dash(SkillDash),
    Jump(SkillJump),
    OnHitEffects(Vec<String>),
    DeployDamage {
        damage: i32,
        radius: f32,
        effects: Vec<String>,
    },
    Invisibility {
        period_ms: u32,
        visible_ms: u32,
    },
    Burrow(SkillBurrow),
    Hook(SkillHook),
    Rebirth(SkillRebirth),
    SpawnPeriodic(SkillSpawnPeriodic),
    Aura(SkillAura),
    Reflect {
        percent: f32,
        max_range: f32,
    },
    Ramp(SkillRamp),
    DeployAnywhere,
    ElixirGenerate(SkillElixirGenerate),
    ElixirOnDeath {
        amount: f32,
    },
    SelfDestruct(SkillSelfDestruct),
    TransformOnDeath {
        building: String,
        health: i32,
    },
    HealOnHit {
        amount: i32,
    },
    ChargedShot(SkillChargedShot),
    DeathEffects {
        effects: Vec<String>,
        radius: f32,
    },
}

#[derive(Debug, Clone)]
pub struct EffectInstance {
    pub preset: String,
    pub kind: royale_defs::EffectKind,
    pub remaining_ms: i32,
    pub duration_ms: i32,
    pub speed_mult: f32,
    pub attack_mult: f32,
    pub dps: f32,
    pub tick_ms: u32,
    pub tick_acc: i32,
    pub amount: f32,
    pub heal_per_sec: f32,
    pub damage_mult: f32,
    pub source_side: Side,
}

#[derive(Debug, Clone)]
pub struct DeathSpawnRuntime {
    pub troop: String,
    pub count: u32,
    pub delay_ms: u32,
}

#[derive(Debug, Clone)]
pub struct DeathSpec {
    pub damage: i32,
    pub radius: f32,
    pub effects: Vec<String>,
    pub spawn: Vec<DeathSpawnRuntime>,
}

impl DeathSpec {
    pub fn from_def(def: &DeathDef) -> Self {
        Self {
            damage: def.damage,
            radius: def.radius,
            effects: def.effects.clone(),
            spawn: def
                .spawn
                .iter()
                .map(|spawn| DeathSpawnRuntime {
                    troop: spawn.troop.clone(),
                    count: spawn.count,
                    delay_ms: spawn.delay_ms,
                })
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.damage <= 0 && self.effects.is_empty() && self.spawn.is_empty()
    }
}

impl Default for DeathSpec {
    fn default() -> Self {
        Self {
            damage: 0,
            radius: 0.0,
            effects: Vec::new(),
            spawn: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct JumpState {
    pub start_tick: u32,
    pub end_tick: u32,
    pub sx: f32,
    pub sy: f32,
    pub tx: f32,
    pub ty: f32,
    pub damage: i32,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct InvisState {
    pub invisible: bool,
    pub left_ms: i32,
    pub period_ms: u32,
    pub visible_ms: u32,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: u32,
    pub side: Side,
    pub kind: EntityKind,
    pub slug: String,
    pub card_slug: String,
    pub rarity: Option<Rarity>,
    pub hp: i32,
    pub max_hp: i32,
    pub shield: i32,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub mass: f32,
    pub flying: bool,
    pub base_speed: f32,
    pub targets: TargetMask,
    pub sight_range: f32,
    pub attack: Option<AttackRuntime>,
    pub skills: Vec<SkillRuntime>,
    pub effects: Vec<EffectInstance>,
    pub target: Option<u32>,
    pub attack_cd: i32,
    pub windup: i32,
    pub windup_target: Option<u32>,
    pub anim: u8,
    pub facing: i8,
    pub life_ms: i32,
    pub active: bool,
    pub is_tower: bool,
    pub tower_role: TowerRole,
    pub lane: u8,
    pub moving: bool,
    pub charge_moved: f32,
    pub charge_ready: bool,
    pub idle_ms: i32,
    pub dash_timer: i32,
    pub dash_end_tick: u32,
    pub dash_target: Option<u32>,
    pub jump_cd: i32,
    pub jump: Option<JumpState>,
    pub invis: Option<InvisState>,
    pub burrow_left: i32,
    pub burrowed: bool,
    pub ramp_mult: f32,
    pub ramp_target: Option<u32>,
    pub ramp_progress: u32,
    pub ramp_idle: i32,
    pub spawn_timer: i32,
    pub hook_timer: i32,
    pub elixir_timer: i32,
    pub charged_timer: i32,
    pub heal_on_hit: i32,
    pub self_destruct: Option<SkillSelfDestruct>,
    pub transform_on_death: Option<(String, i32)>,
    pub rebirth: Option<SkillRebirth>,
    pub death_effects: Option<(Vec<String>, f32)>,
    pub death_spec: DeathSpec,
    pub deploy_anywhere: bool,
    pub no_rebirth: bool,
    pub last_damage_tick: u32,
    pub last_move_tick: u32,
    pub spawn_serial: u32,
}

impl Entity {
    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    pub fn is_stunned(&self) -> bool {
        self.effects.iter().any(|effect| {
            matches!(
                effect.kind,
                royale_defs::EffectKind::Freeze | royale_defs::EffectKind::Stun
            )
        })
    }

    pub fn is_snared(&self) -> bool {
        self.effects
            .iter()
            .any(|effect| matches!(effect.kind, royale_defs::EffectKind::Snare))
    }

    pub fn speed_mult(&self) -> f32 {
        let mut mult = 1.0f32;
        for effect in &self.effects {
            match effect.kind {
                royale_defs::EffectKind::Freeze | royale_defs::EffectKind::Stun => return 0.0,
                royale_defs::EffectKind::Slow
                | royale_defs::EffectKind::Snare
                | royale_defs::EffectKind::Poison
                | royale_defs::EffectKind::Burn
                | royale_defs::EffectKind::Shock => {
                    mult *= effect.speed_mult.clamp(0.0, 1.0);
                }
                royale_defs::EffectKind::Rage | royale_defs::EffectKind::SpeedBoost => {
                    mult *= effect.speed_mult.max(0.0)
                }
                _ => {}
            }
        }
        mult
    }

    pub fn attack_mult(&self) -> f32 {
        let mut mult = 1.0f32;
        for effect in &self.effects {
            match effect.kind {
                royale_defs::EffectKind::Freeze | royale_defs::EffectKind::Stun => return 0.0,
                royale_defs::EffectKind::Rage
                | royale_defs::EffectKind::AttackBoost
                | royale_defs::EffectKind::Slow => {
                    mult *= effect.attack_mult.max(0.0);
                }
                _ => {}
            }
        }
        mult
    }

    pub fn damage_taken_mult(&self) -> f32 {
        self.effects
            .iter()
            .filter(|effect| matches!(effect.kind, royale_defs::EffectKind::DamageAmp))
            .map(|effect| effect.damage_mult.max(1.0))
            .product::<f32>()
            .max(1.0)
    }

    pub fn is_invisible(&self) -> bool {
        self.invis.as_ref().is_some_and(|state| state.invisible)
    }

    pub fn untargetable(&self) -> bool {
        self.burrowed || self.is_invisible()
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub side: Side,
    pub hand: Vec<u16>,
    pub next: u16,
    pub queue: std::collections::VecDeque<u16>,
    pub elixir: f32,
    pub crowns: u8,
    pub tower_damage: f32,
}

impl Player {
    pub fn new(user_id: uuid::Uuid, username: String, side: Side, deck: Vec<u16>) -> Option<Self> {
        if deck.len() != royale_shared::DECK_SIZE {
            return None;
        }
        let mut hand: Vec<u16> = deck.iter().take(4).copied().collect();
        let next = *deck.get(4)?;
        let queue: std::collections::VecDeque<u16> = deck.iter().skip(5).copied().collect();
        if hand.len() != 4 {
            return None;
        }
        Some(Self {
            user_id,
            username,
            side,
            hand: std::mem::take(&mut hand),
            next,
            queue,
            elixir: 5.0,
            crowns: 0,
            tower_damage: 0.0,
        })
    }

    pub fn cycle_card(&mut self, slot: usize, played: u16) {
        if slot >= self.hand.len() {
            return;
        }
        self.hand[slot] = self.next;
        self.next = self.queue.pop_front().unwrap_or(played);
        self.queue.push_back(played);
    }
}

#[derive(Debug, Clone)]
pub struct Projectile {
    pub id: u32,
    pub side: Side,
    pub x: f32,
    pub y: f32,
    pub dir_x: f32,
    pub dir_y: f32,
    pub target: Option<u32>,
    pub tx: f32,
    pub ty: f32,
    pub speed: f32,
    pub radius: f32,
    pub homing: bool,
    pub visual_idx: u16,
    pub damage: i32,
    pub splash_radius: f32,
    pub pierce: bool,
    pub chain: u32,
    pub knockback: f32,
    pub effects: Vec<String>,
    pub source_id: Option<u32>,
    pub hit_ids: Vec<u32>,
    pub distance_left: f32,
    pub traveled: f32,
    pub mask: TargetMask,
}

#[derive(Debug, Clone)]
pub struct ZoneSpawn {
    pub troop: String,
    pub remaining: u32,
    pub interval_ms: u32,
    pub timer: i32,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct Zone {
    pub id: u32,
    pub side: Side,
    pub spell_idx: u16,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub remaining_ms: i32,
    pub dps: f32,
    pub effects: Vec<String>,
    pub affects_allies: bool,
    pub pull: Option<f32>,
    pub spawn: Option<ZoneSpawn>,
    pub slow_mult: Option<f32>,
    pub tick_acc: i32,
    pub tower_damage_mult: f32,
    pub mask: TargetMask,
}

#[derive(Debug, Clone)]
pub struct RollingSpell {
    pub id: u32,
    pub side: Side,
    pub spell_idx: u16,
    pub x: f32,
    pub y: f32,
    pub dir_x: f32,
    pub dir_y: f32,
    pub speed: f32,
    pub distance_left: f32,
    pub width: f32,
    pub damage: i32,
    pub tower_damage_mult: f32,
    pub effects: Vec<String>,
    pub knockback: f32,
    pub hit_ids: Vec<u32>,
    pub mask: TargetMask,
}

#[derive(Debug, Clone)]
pub enum PendingKind {
    SpawnTroop {
        card_slug: String,
        troop_slug: String,
        x: f32,
        y: f32,
        health: Option<i32>,
        no_rebirth: bool,
    },
    SpawnBuilding {
        card_slug: String,
        building_slug: String,
        x: f32,
        y: f32,
        health: Option<i32>,
    },
    CastSpell {
        card_slug: String,
        spell_slug: String,
        x: f32,
        y: f32,
    },
}

#[derive(Debug, Clone)]
pub struct PendingAction {
    pub at_tick: u32,
    pub side: Side,
    pub kind: PendingKind,
}

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub kind: u8,
    pub x: f32,
    pub y: f32,
    pub side: u8,
    pub index: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct MatchResult {
    pub winner: Option<Side>,
    pub end_reason: EndReason,
    pub crowns: [u8; 2],
    pub damage: [f32; 2],
    pub duration_ms: u32,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            id: 0,
            side: Side::Blue,
            kind: EntityKind::Troop,
            slug: String::new(),
            card_slug: String::new(),
            rarity: None,
            hp: 1,
            max_hp: 1,
            shield: 0,
            x: 0.0,
            y: 0.0,
            radius: 0.3,
            mass: 1.0,
            flying: false,
            base_speed: 1.0,
            targets: TargetMask::default(),
            sight_range: 5.5,
            attack: None,
            skills: Vec::new(),
            effects: Vec::new(),
            target: None,
            attack_cd: 0,
            windup: 0,
            windup_target: None,
            anim: ANIM_IDLE,
            facing: 1,
            life_ms: -1,
            active: false,
            is_tower: false,
            tower_role: TowerRole::None,
            lane: 0,
            moving: false,
            charge_moved: 0.0,
            charge_ready: false,
            idle_ms: 0,
            dash_timer: 0,
            dash_end_tick: 0,
            dash_target: None,
            jump_cd: 0,
            jump: None,
            invis: None,
            burrow_left: 0,
            burrowed: false,
            ramp_mult: 1.0,
            ramp_target: None,
            ramp_progress: 0,
            ramp_idle: 0,
            spawn_timer: 0,
            hook_timer: 0,
            elixir_timer: 0,
            charged_timer: 0,
            heal_on_hit: 0,
            self_destruct: None,
            transform_on_death: None,
            rebirth: None,
            death_effects: None,
            death_spec: DeathSpec::default(),
            deploy_anywhere: false,
            no_rebirth: false,
            last_damage_tick: 0,
            last_move_tick: 0,
            spawn_serial: 0,
        }
    }
}

/// Visão das tropas exigida pela estrutura de sala: card_slug, troop_slug,
/// health, pos_x, pos_y e effects.
#[derive(Debug, Clone)]
pub struct TroopState {
    pub id: u32,
    pub card_slug: String,
    pub troop_slug: String,
    pub health: i32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub effects: Vec<EffectInstance>,
}
