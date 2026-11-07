use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definitions {
    pub cards: Vec<CardDef>,
    pub troops: BTreeMap<String, TroopDef>,
    pub buildings: BTreeMap<String, BuildingDef>,
    pub towers: BTreeMap<String, TowerDef>,
    pub spells: BTreeMap<String, SpellDef>,
    pub projectiles: BTreeMap<String, ProjectileDef>,
    pub effects: BTreeMap<String, EffectDef>,
    pub skills: BTreeMap<String, SkillDef>,
    pub arenas: BTreeMap<String, ArenaDef>,
}

impl Definitions {
    pub fn card(&self, slug: &str) -> Option<&CardDef> {
        self.cards.iter().find(|card| card.slug == slug)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDef {
    pub slug: String,
    pub rarity: Rarity,
    pub elixir_cost: u8,
    #[serde(default)]
    pub summon_time_ms: u32,
    #[serde(rename = "type")]
    pub card_type: CardType,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub sort: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CardType {
    Troop { troop: String, count: u32 },
    Spell { spell: String },
    Building { building: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    Ground,
    Air,
    Buildings,
    Towers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TroopDef {
    pub slug: String,
    pub health: i32,
    pub speed: f32,
    pub radius: f32,
    #[serde(default = "default_mass")]
    pub mass: f32,
    #[serde(default)]
    pub flying: bool,
    #[serde(default)]
    pub targets: Vec<TargetKind>,
    #[serde(default = "default_sight")]
    pub sight_range: f32,
    pub attack: AttackDef,
    #[serde(default)]
    pub skills: Vec<SkillRef>,
    #[serde(default)]
    pub death: DeathDef,
    #[serde(default)]
    pub shield: i32,
    pub visual: VisualDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingDef {
    pub slug: String,
    pub health: i32,
    #[serde(default = "default_lifetime")]
    pub lifetime_ms: u32,
    pub radius: f32,
    #[serde(default)]
    pub targets: Vec<TargetKind>,
    #[serde(default = "default_sight")]
    pub sight_range: f32,
    #[serde(default)]
    pub attack: Option<AttackDef>,
    #[serde(default)]
    pub skills: Vec<SkillRef>,
    #[serde(default)]
    pub death: DeathDef,
    pub visual: VisualDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerDef {
    pub slug: String,
    pub health: i32,
    pub radius: f32,
    pub attack: AttackDef,
    pub visual: VisualDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackDef {
    pub range: f32,
    pub damage: i32,
    #[serde(default = "default_cooldown")]
    pub cooldown_ms: u32,
    #[serde(default)]
    pub windup_ms: u32,
    #[serde(default)]
    pub first_hit_ms: u32,
    #[serde(default)]
    pub splash_radius: f32,
    #[serde(default)]
    pub projectile: Option<String>,
    #[serde(default)]
    pub projectile_speed: f32,
    #[serde(default)]
    pub pierce: bool,
    #[serde(default)]
    pub chain: u32,
    #[serde(default)]
    pub knockback: f32,
    #[serde(default)]
    pub multi_shot: u32,
    #[serde(default)]
    pub targets_air: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathDef {
    #[serde(default)]
    pub damage: i32,
    #[serde(default)]
    pub radius: f32,
    #[serde(default)]
    pub effects: Vec<String>,
    #[serde(default)]
    pub spawn: Vec<DeathSpawn>,
}

impl Default for DeathDef {
    fn default() -> Self {
        Self {
            damage: 0,
            radius: 0.0,
            effects: Vec::new(),
            spawn: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathSpawn {
    pub troop: String,
    #[serde(default = "default_one")]
    pub count: u32,
    #[serde(default)]
    pub delay_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRef {
    pub skill: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualDef {
    #[serde(default = "default_archetype")]
    pub archetype: String,
    #[serde(default = "default_scale")]
    pub scale: f32,
    #[serde(default = "default_primary")]
    pub primary: String,
    #[serde(default = "default_secondary")]
    pub secondary: String,
    #[serde(default = "default_weapon")]
    pub weapon: String,
    #[serde(default)]
    pub accessory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellDef {
    pub slug: String,
    pub radius: f32,
    #[serde(default)]
    pub delay_ms: u32,
    #[serde(default)]
    pub damage: i32,
    #[serde(default = "default_tower_mult")]
    pub tower_damage_mult: f32,
    #[serde(default)]
    pub targets: Vec<TargetKind>,
    #[serde(default)]
    pub effects: Vec<String>,
    #[serde(default)]
    pub zone: Option<ZoneDef>,
    #[serde(default)]
    pub spawn: Option<SpellSpawnDef>,
    #[serde(default)]
    pub rolling: Option<RollingDef>,
    #[serde(default)]
    pub pull: Option<PullDef>,
    pub visual: SpellVisual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneDef {
    pub duration_ms: u32,
    #[serde(default)]
    pub dps: f32,
    #[serde(default)]
    pub effects: Vec<String>,
    #[serde(default)]
    pub effect: Option<String>,
    #[serde(default)]
    pub slow_mult: Option<f32>,
    #[serde(default)]
    pub affects_allies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellSpawnDef {
    pub troop: String,
    #[serde(default = "default_one")]
    pub count: u32,
    #[serde(default)]
    pub interval_ms: u32,
    #[serde(default = "default_spawn_radius")]
    pub radius: f32,
    #[serde(default)]
    pub first_delay_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingDef {
    pub distance: f32,
    pub speed: f32,
    #[serde(default = "default_width")]
    pub width: f32,
    #[serde(default)]
    pub knockback: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullDef {
    #[serde(default = "default_pull_strength")]
    pub strength: f32,
    #[serde(default)]
    pub duration_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellVisual {
    #[serde(default)]
    pub variant: String,
    #[serde(default)]
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectileDef {
    pub slug: String,
    #[serde(default = "default_projectile_speed")]
    pub speed: f32,
    #[serde(default)]
    pub radius: f32,
    #[serde(default)]
    pub homing: bool,
    #[serde(default)]
    pub visual: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectDef {
    pub kind: EffectKind,
    #[serde(default)]
    pub duration_ms: u32,
    #[serde(default)]
    pub params: EffectParams,
    #[serde(default)]
    pub visual: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectKind {
    Freeze,
    Stun,
    Slow,
    Snare,
    Poison,
    Burn,
    Shock,
    Rage,
    Heal,
    Invisibility,
    DamageAmp,
    SpeedBoost,
    AttackBoost,
    Knockback,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectParams {
    #[serde(default)]
    pub speed_mult: Option<f32>,
    #[serde(default)]
    pub attack_mult: Option<f32>,
    #[serde(default)]
    pub dps: Option<f32>,
    #[serde(default)]
    pub tick_ms: Option<u32>,
    #[serde(default)]
    pub amount: Option<f32>,
    #[serde(default)]
    pub heal_per_sec: Option<f32>,
    #[serde(default)]
    pub damage_mult: Option<f32>,
    #[serde(default)]
    pub element: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub kind: SkillKind,
    #[serde(default)]
    pub params: SkillParams,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillKind {
    Charge,
    Dash,
    Jump,
    OnHitEffects,
    DeployDamage,
    Invisibility,
    Burrow,
    Hook,
    Rebirth,
    SpawnPeriodic,
    Aura,
    Reflect,
    Ramp,
    Shield,
    DeployAnywhere,
    ElixirGenerate,
    ElixirOnDeath,
    SelfDestruct,
    TransformOnDeath,
    HealOnHit,
    ChargedShot,
    DeathEffects,
    Tornado,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillParams {
    #[serde(default)]
    pub min_distance: Option<f32>,
    #[serde(default)]
    pub speed_mult: Option<f32>,
    #[serde(default)]
    pub bonus_damage: Option<i32>,
    #[serde(default)]
    pub reset_ms: Option<u32>,
    #[serde(default)]
    pub trigger_range: Option<f32>,
    #[serde(default)]
    pub duration_ms: Option<u32>,
    #[serde(default)]
    pub windup_ms: Option<u32>,
    #[serde(default)]
    pub damage: Option<i32>,
    #[serde(default)]
    pub radius: Option<f32>,
    #[serde(default)]
    pub effects: Vec<String>,
    #[serde(default)]
    pub period_ms: Option<u32>,
    #[serde(default)]
    pub visible_ms: Option<u32>,
    #[serde(default)]
    pub travel_ms: Option<u32>,
    #[serde(default)]
    pub deploy_anywhere: Option<bool>,
    #[serde(default)]
    pub range: Option<f32>,
    #[serde(default)]
    pub pull_distance: Option<f32>,
    #[serde(default)]
    pub cooldown_ms: Option<u32>,
    #[serde(default)]
    pub health: Option<i32>,
    #[serde(default)]
    pub troop: Option<String>,
    #[serde(default)]
    pub building: Option<String>,
    #[serde(default)]
    pub count: Option<u32>,
    #[serde(default)]
    pub interval_ms: Option<u32>,
    #[serde(default)]
    pub first_ms: Option<u32>,
    #[serde(default)]
    pub max_alive: Option<u32>,
    #[serde(default)]
    pub dps: Option<f32>,
    #[serde(default)]
    pub heal_per_sec: Option<f32>,
    #[serde(default)]
    pub affects_allies: Option<bool>,
    #[serde(default)]
    pub affects_enemies: Option<bool>,
    #[serde(default)]
    pub percent: Option<f32>,
    #[serde(default)]
    pub max_range: Option<f32>,
    #[serde(default)]
    pub max_mult: Option<f32>,
    #[serde(default)]
    pub amount: Option<f32>,
    #[serde(default)]
    pub charge_ms: Option<u32>,
    #[serde(default)]
    pub splash_radius: Option<f32>,
    #[serde(default)]
    pub projectile: Option<String>,
    #[serde(default)]
    pub strength: Option<f32>,
    #[serde(default)]
    pub knockback: Option<f32>,
    #[serde(default)]
    pub delay_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaDef {
    pub slug: String,
    pub width: f32,
    pub height: f32,
    pub river_top: f32,
    pub river_bottom: f32,
    pub bridges: Vec<[f32; 2]>,
    pub tower_positions: ArenaTowers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaTowers {
    pub top: TowerPositions,
    pub bottom: TowerPositions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerPositions {
    pub king: [f32; 2],
    pub princess: Vec<[f32; 2]>,
}

fn default_one() -> u32 {
    1
}

fn default_mass() -> f32 {
    1.0
}

fn default_sight() -> f32 {
    5.5
}

fn default_cooldown() -> u32 {
    1000
}

fn default_lifetime() -> u32 {
    30_000
}

fn default_scale() -> f32 {
    1.0
}

fn default_archetype() -> String {
    "humanoid".into()
}

fn default_primary() -> String {
    "#94a3b8".into()
}

fn default_secondary() -> String {
    "#e2e8f0".into()
}

fn default_weapon() -> String {
    "sword".into()
}

fn default_tower_mult() -> f32 {
    1.0
}

fn default_spawn_radius() -> f32 {
    2.5
}

fn default_width() -> f32 {
    1.0
}

fn default_pull_strength() -> f32 {
    2.0
}

fn default_projectile_speed() -> f32 {
    8.0
}
