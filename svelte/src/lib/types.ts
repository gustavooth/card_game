export type Side = 0 | 1;
export type Locale = 'pt-BR' | 'en';
export type MenuTab = 'cards' | 'battle' | 'chat' | 'profile';

export interface Catalog {
	cards: string[];
	troops: string[];
	buildings: string[];
	towers: string[];
	spells: string[];
	projectiles: string[];
	effects: string[];
	skills: string[];
	arenas: string[];
}

export interface VisualDef {
	archetype: string;
	scale: number;
	primary: string;
	secondary: string;
	weapon: string;
	accessory: string;
}

export interface AttackDef {
	range: number;
	damage: number;
	cooldown_ms: number;
	windup_ms: number;
	first_hit_ms: number;
	splash_radius: number;
	projectile: string | null;
	projectile_speed: number;
	pierce: boolean;
	chain: number;
	knockback: number;
	multi_shot: number;
	targets_air: boolean;
}

export interface DeathDef {
	damage: number;
	radius: number;
	effects: string[];
	spawn: { troop: string; count: number; delay_ms: number }[];
}

export interface TroopDef {
	slug: string;
	health: number;
	speed: number;
	radius: number;
	mass: number;
	flying: boolean;
	targets: string[];
	sight_range: number;
	attack: AttackDef;
	skills: { skill: string }[];
	death: DeathDef;
	shield?: number;
	visual: VisualDef;
}

export interface BuildingDef {
	slug: string;
	health: number;
	lifetime_ms: number;
	radius: number;
	targets: string[];
	sight_range: number;
	attack: AttackDef | null;
	skills: { skill: string }[];
	death: DeathDef;
	visual: VisualDef;
}

export interface TowerDef {
	slug: string;
	health: number;
	radius: number;
	attack: AttackDef;
	visual: VisualDef;
}

export interface SpellDef {
	slug: string;
	radius: number;
	delay_ms: number;
	damage: number;
	tower_damage_mult: number;
	targets: string[];
	effects: string[];
	zone: {
		duration_ms: number;
		dps: number;
		effects: string[];
		effect?: string;
		slow_mult?: number;
		affects_allies?: boolean;
	} | null;
	spawn: {
		troop: string;
		count: number;
		interval_ms: number;
		radius: number;
		first_delay_ms?: number;
	} | null;
	rolling: { distance: number; speed: number; width: number; knockback: number } | null;
	pull: { strength: number; duration_ms: number } | null;
	visual: { variant: string; color: string };
}

export interface ProjectileDef {
	slug: string;
	speed: number;
	radius: number;
	homing: boolean;
	visual: string;
}

export interface EffectDef {
	kind: string;
	duration_ms: number;
	params: Record<string, number | string | undefined>;
	visual: string;
}

export interface SkillDef {
	kind: string;
	params: Record<string, unknown>;
}

export interface CardDef {
	slug: string;
	rarity: 'common' | 'rare' | 'epic' | 'legendary';
	elixir_cost: number;
	summon_time_ms: number;
	type:
		| { kind: 'troop'; troop: string; count: number }
		| { kind: 'spell'; spell: string }
		| { kind: 'building'; building: string };
	tags: string[];
	sort: number;
}

export interface ArenaDef {
	slug: string;
	width: number;
	height: number;
	river_top: number;
	river_bottom: number;
	bridges: [number, number][];
	tower_positions: {
		top: { king: [number, number]; princess: [number, number][] };
		bottom: { king: [number, number]; princess: [number, number][] };
	};
}

export interface Definitions {
	cards: CardDef[];
	troops: Record<string, TroopDef>;
	buildings: Record<string, BuildingDef>;
	towers: Record<string, TowerDef>;
	spells: Record<string, SpellDef>;
	projectiles: Record<string, ProjectileDef>;
	effects: Record<string, EffectDef>;
	skills: Record<string, SkillDef>;
	arenas: Record<string, ArenaDef>;
}

export interface Content {
	version: number;
	catalog: Catalog;
	definitions: Definitions;
}

export interface User {
	id: string;
	username: string;
	email: string;
	locale: string;
	created_at: string;
}

export interface PlayerStats {
	games: number;
	wins: number;
	losses: number;
	draws: number;
	crowns_for: number;
	crowns_against: number;
}

export interface RecentMatch {
	id: string;
	ended_at: string;
	duration_ms: number;
	opponent: string;
	result: 'win' | 'loss' | 'draw';
	crowns: number;
	opponent_crowns: number;
}

export interface ReplayMeta {
	id: string;
	match_id: string;
	created_at: string;
}

export interface PlayerSnapshot {
	s: Side;
	e: number;
	c: number;
	d: number;
	h: [number, number][];
	n: [number, number];
}

export interface Snapshot {
	t: 's';
	k: number;
	m: number;
	ph: number;
	ot: boolean;
	p: PlayerSnapshot[];
	e: number[][];
	ev: number[][];
}

export interface MatchInfo {
	room: string;
	side: Side;
	opponent: string;
	arena: string;
	reconnect: boolean;
}

export interface EndInfo {
	result: 'win' | 'loss' | 'draw';
	winner: Side | null;
	crowns: [number, number];
	damage: [number, number];
	duration_ms: number;
	reason: string;
}

export interface ReplayHeader {
	side: Side;
	arena: string;
	blue: string;
	red: string;
	started_at: number;
	opponent: string;
	result: 'win' | 'loss' | 'draw';
	crowns: [number, number];
}

export const ENTITY = {
	ID: 0,
	KIND: 1,
	SIDE: 2,
	TYPE: 3,
	HP: 4,
	MAX_HP: 5,
	X: 6,
	Y: 7,
	RADIUS: 8,
	FLYING: 9,
	SHIELD: 10,
	ANIM: 11,
	FACING: 12,
	LIFE: 13,
	ACTIVE: 14,
	EFFECT_COUNT: 15,
	BASE: 16
} as const;

export const KIND = { TROOP: 0, BUILDING: 1, TOWER: 2, PROJECTILE: 3, ZONE: 4 } as const;
export const ANIM = { IDLE: 0, WALK: 1, ATTACK: 2, BURROW: 3 } as const;

export const EVENT = {
	DEPLOY: 1,
	HIT: 2,
	DEATH: 3,
	SPELL: 4,
	TOWER_DOWN: 5,
	KING_ACTIVE: 6,
	MATCH_END: 7,
	ELIXIR: 8
} as const;
