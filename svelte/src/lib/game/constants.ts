export const DESIGN = { width: 720, height: 1280 };

export const ARENA_LAYOUT = {
	x: 45,
	y: 74,
	tile: 35,
	depth: 0.88
};

export const ARENA_WIDTH = 18;
export const ARENA_HEIGHT = 32;

export const HUD = {
	top: 8,
	bottom: 1064,
	handY: 1088,
	handHeight: 160,
	cardWidth: 148,
	cardGap: 5,
	elixirY: 1250,
	elixirHeight: 16
};

export const TEAM_COLORS = {
	blue: {
		primary: '#3b82f6',
		light: '#93c5fd',
		dark: '#1d4ed8',
		glow: 'rgba(59, 130, 246, 0.55)'
	},
	red: {
		primary: '#ef4444',
		light: '#fca5a5',
		dark: '#b91c1c',
		glow: 'rgba(239, 68, 68, 0.55)'
	}
} as const;

export const RARITY_COLORS: Record<string, [string, string]> = {
	common: ['#94a3b8', '#cbd5e1'],
	rare: ['#f59e0b', '#fde68a'],
	epic: ['#a855f7', '#e9d5ff'],
	legendary: ['#22d3ee', '#a5f3fc']
};

export const INTERPOLATION_MS = 90;
