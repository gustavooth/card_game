import type {
	BuildingDef,
	CardDef,
	Content,
	MatchInfo,
	Side,
	Snapshot,
	TroopDef
} from '#lib/types';
import { ANIM, ENTITY, KIND } from '#lib/types';
import { ARENA_LAYOUT, DESIGN, HUD, INTERPOLATION_MS, TEAM_COLORS } from './constants';
import { drawArenaGround, drawDeployOverlay, drawTower } from './arena';
import { drawBaked, ensureBaked, type SpriteView } from './sprites';
import { drawCardIcon } from './cardArt';
import {
	drawProjectile,
	drawShield,
	drawUnitEffects,
	drawZone
} from './effects';
import { worldToScreen } from './projection';
import { drawHud } from './hud';

export interface RenderPair {
	prev: Snapshot;
	next: Snapshot;
	alpha: number;
	latest: Snapshot;
}

export interface PendingDeployView {
	card: CardDef;
	x: number;
	y: number;
	side: Side;
	progress: number;
}

export interface Interaction {
	selectedSlot: number | null;
	hover: { x: number; y: number } | null;
	valid: boolean;
	cardKind: 'troop' | 'spell' | 'building';
	drag: { x: number; y: number } | null;
	dragCard: CardDef | null;
	pending: PendingDeployView[];
}

export interface HudRender {
	opponent: string;
	username: string;
	selectedSlot: number | null;
}

function sideColor(entitySide: Side, localSide: Side): 0 | 1 {
	return entitySide === localSide ? 0 : 1;
}

function interpolatePosition(
	prev: number[] | undefined,
	next: number[],
	alpha: number
): { x: number; y: number } {
	if (!prev) {
		return { x: next[ENTITY.X], y: next[ENTITY.Y] };
	}
	return {
		x: prev[ENTITY.X] + (next[ENTITY.X] - prev[ENTITY.X]) * alpha,
		y: prev[ENTITY.Y] + (next[ENTITY.Y] - prev[ENTITY.Y]) * alpha
	};
}

function drawHealthBar(
	ctx: CanvasRenderingContext2D,
	x: number,
	y: number,
	width: number,
	hp: number,
	maxHp: number,
	side: Side
) {
	if (hp >= maxHp || maxHp <= 0) {
		return;
	}
	const height = 4;
	ctx.save();
	ctx.fillStyle = 'rgba(15, 23, 42, 0.85)';
	ctx.fillRect(x - width / 2 - 1, y - 1, width + 2, height + 2);
	ctx.fillStyle = side === 0 ? TEAM_COLORS.blue.primary : TEAM_COLORS.red.primary;
	ctx.fillRect(x - width / 2, y, width * Math.max(0, hp / maxHp), height);
	ctx.restore();
}

function drawShadow(
	ctx: CanvasRenderingContext2D,
	x: number,
	y: number,
	radius: number,
	alpha = 0.28
) {
	ctx.save();
	ctx.beginPath();
	ctx.ellipse(x, y + 2, radius, radius * 0.38, 0, 0, Math.PI * 2);
	ctx.fillStyle = `rgba(0, 0, 0, ${alpha})`;
	ctx.fill();
	ctx.restore();
}

function drawBuildingUnit(
	ctx: CanvasRenderingContext2D,
	building: BuildingDef,
	x: number,
	y: number,
	team: 0 | 1,
	hp: number,
	maxHp: number,
	elapsedMs: number
) {
	const size = 62;
	drawShadow(ctx, x, y, 24);
	const bob = Math.sin(elapsedMs * 0.002 + x) * 0.6;
	drawCardIcon(
		ctx,
		{
			slug: building.slug,
			rarity: 'common',
			elixir_cost: 0,
			summon_time_ms: 0,
			type: { kind: 'building', building: building.slug },
			tags: [],
			sort: 0
		},
		{
			troops: {},
			buildings: { [building.slug]: building },
			spells: {}
		},
		x - size / 2,
		y - size + 4 + bob,
		size,
		team
	);
	drawHealthBar(ctx, x, y - size - 12, 42, hp, maxHp, team);
}

function drawTroop(
	ctx: CanvasRenderingContext2D,
	troop: TroopDef,
	x: number,
	y: number,
	team: 0 | 1,
	hp: number,
	maxHp: number,
	anim: number,
	shield: number,
	effects: number[],
	elapsedMs: number,
	view: SpriteView
) {
	const visual = troop.visual;
	const archetypeScale =
		{
			giant: 1.38,
			dragon: 1.28,
			beast: 1.16,
			machine: 1.1,
			flyer: 1.06,
			goblin: 0.9,
			skeleton: 0.94,
			spirit: 0.98
		}[visual.archetype] ?? 1;
	const baseSize = ARENA_LAYOUT.tile * 2.05 * (visual.scale || 1) * archetypeScale;
	const size = Math.min(150, Math.max(38, baseSize));
	const flying = troop.flying;
	const lift = flying ? 22 + Math.sin(elapsedMs * 0.004 + x) * 3 : 0;
	drawShadow(ctx, x, y, size * 0.22, flying ? 0.18 : 0.3);
	if (anim === ANIM.BURROW) {
		ctx.save();
		ctx.beginPath();
		ctx.ellipse(x, y, 18, 7, 0, 0, Math.PI * 2);
		ctx.fillStyle = '#78350f';
		ctx.fill();
		ctx.strokeStyle = '#451a03';
		ctx.lineWidth = 2;
		ctx.stroke();
		ctx.restore();
		return;
	}
	const baked = ensureBaked(visual, team, view, troop.slug);
	if (baked) {
		let clip: 'idle' | 'walk' | 'attack' = 'idle';
		if (anim === ANIM.WALK) {
			clip = 'walk';
		} else if (anim === ANIM.ATTACK) {
			clip = 'attack';
		}
		drawBaked(ctx, baked, {
			x,
			y: y - lift,
			size,
			facing: 1,
			clip,
			elapsedMs: elapsedMs + x * 13
		});
	}
	if (shield > 0) {
		drawShield(ctx, x, y - lift, size * 0.28, elapsedMs);
	}
	const barWidth = Math.max(26, size * 0.5);
	drawHealthBar(ctx, x, y - lift - size * 0.88, barWidth, hp, maxHp, team);
	if (effects[0] > 0) {
		drawUnitEffects(ctx, x, y - lift, size * 0.3, effects, elapsedMs);
	}
}

// A vista depende do sentido do movimento no ponto de vista local:
// subindo (para longe do jogador) mostra as costas, descendo mostra a frente.
function resolveSpriteView(
	id: number,
	prev: number[] | undefined,
	next: number[],
	localSide: Side,
	views: Map<number, SpriteView>
): SpriteView {
	if (prev) {
		const dy = next[ENTITY.Y] - prev[ENTITY.Y];
		const localDy = localSide === 0 ? dy : -dy;
		if (localDy < -0.008) {
			views.set(id, 'back');
		} else if (localDy > 0.008) {
			views.set(id, 'front');
		}
	}
	return views.get(id) ?? 'front';
}

// import tardio para evitar ciclo de módulos na inicialização
function drawTowersAndEntities(
	ctx: CanvasRenderingContext2D,
	content: Content,
	pair: RenderPair,
	localSide: Side,
	elapsedMs: number,
	views: Map<number, SpriteView>
) {
	const prevMap = new Map<number, number[]>();
	for (const entity of pair.prev.e) {
		prevMap.set(entity[ENTITY.ID], entity);
	}
	const sorted = [...pair.next.e]
		.filter((entity) => entity[ENTITY.KIND] <= KIND.TOWER)
		.sort((a, b) => {
			const ay = localSide === 0 ? a[ENTITY.Y] : 32 - a[ENTITY.Y];
			const by = localSide === 0 ? b[ENTITY.Y] : 32 - b[ENTITY.Y];
			return ay - by;
		});

	for (const entity of sorted) {
		const id = entity[ENTITY.ID];
		const position = interpolatePosition(prevMap.get(id), entity, pair.alpha);
		const screen = worldToScreen(position.x, position.y, localSide);
		const kind = entity[ENTITY.KIND];
		const actualSide = entity[ENTITY.SIDE] as Side;
		const team: 0 | 1 = sideColor(actualSide, localSide);
		const hp = entity[ENTITY.HP];
		const maxHp = entity[ENTITY.MAX_HP];
		if (kind === KIND.TOWER) {
			const tower = content.definitions.towers[content.catalog.towers[entity[ENTITY.TYPE]]];
			if (tower) {
				const role = tower.slug === 'king_tower' ? 'King' : 'Princess';
				drawTower(
					ctx,
					tower,
					team,
					screen.x,
					screen.y + 6,
					hp,
					maxHp,
					entity[ENTITY.ACTIVE] > 0,
					role
				);
			}
			continue;
		}
		if (kind === KIND.BUILDING) {
			const building = content.definitions.buildings[content.catalog.buildings[entity[ENTITY.TYPE]]];
			if (building) {
				drawBuildingUnit(ctx, building, screen.x, screen.y, team, hp, maxHp, elapsedMs);
			}
			continue;
		}
		if (kind === KIND.TROOP) {
			const troop = content.definitions.troops[content.catalog.troops[entity[ENTITY.TYPE]]];
			if (!troop) {
				continue;
			}
			const effects: number[] = [];
			const count = Math.floor(entity[ENTITY.EFFECT_COUNT] ?? 0);
			effects.push(count);
			for (let index = 0; index < count; index += 1) {
				const base = ENTITY.BASE + index * 3;
				effects.push(entity[base] ?? 0, entity[base + 1] ?? 0, entity[base + 2] ?? 0);
			}
			const anim = entity[ENTITY.ANIM];
			const view = resolveSpriteView(id, prevMap.get(id), entity, localSide, views);
			drawTroop(
				ctx,
				troop,
				screen.x,
				screen.y,
				team,
				hp,
				maxHp,
				anim,
				entity[ENTITY.SHIELD],
				effects,
				elapsedMs,
				view
			);
		}
	}
}

function drawProjectiles(
	ctx: CanvasRenderingContext2D,
	content: Content,
	pair: RenderPair,
	localSide: Side,
	elapsedMs: number
) {
	const prevMap = new Map<number, number[]>();
	for (const entity of pair.prev.e) {
		prevMap.set(entity[ENTITY.ID], entity);
	}
	for (const entity of pair.next.e) {
		if (entity[ENTITY.KIND] !== KIND.PROJECTILE) {
			continue;
		}
		const position = interpolatePosition(prevMap.get(entity[ENTITY.ID]), entity, pair.alpha);
		const screen = worldToScreen(position.x, position.y, localSide);
		const def = content.definitions.projectiles[content.catalog.projectiles[entity[ENTITY.TYPE]]];
		drawProjectile(
			ctx,
			def?.visual ?? 'ball',
			screen.x,
			screen.y,
			entity[ENTITY.FACING] >= 0 ? 1 : -1,
			elapsedMs
		);
	}
}

function drawZones(
	ctx: CanvasRenderingContext2D,
	content: Content,
	pair: RenderPair,
	localSide: Side,
	elapsedMs: number
) {
	const prevMap = new Map<number, number[]>();
	for (const entity of pair.prev.e) {
		prevMap.set(entity[ENTITY.ID], entity);
	}
	for (const entity of pair.next.e) {
		if (entity[ENTITY.KIND] !== KIND.ZONE) {
			continue;
		}
		const position = interpolatePosition(prevMap.get(entity[ENTITY.ID]), entity, pair.alpha);
		const screen = worldToScreen(position.x, position.y, localSide);
		const spellIndex = entity[ENTITY.TYPE];
		const spell = content.definitions.spells[content.catalog.spells[spellIndex]];
		if (!spell) {
			continue;
		}
		if (spell.rolling) {
			ctx.save();
			drawShadow(ctx, screen.x, screen.y, 12);
			ctx.translate(screen.x, screen.y - 8);
			ctx.rotate((elapsedMs * 0.01) % (Math.PI * 2));
			ctx.fillStyle = '#92400e';
			ctx.beginPath();
			ctx.roundRect(-20, -8, 40, 16, 7);
			ctx.fill();
			ctx.strokeStyle = '#451a03';
			ctx.lineWidth = 2;
			ctx.stroke();
			ctx.fillStyle = '#c2410c';
			ctx.beginPath();
			ctx.arc(17, 0, 7, 0, Math.PI * 2);
			ctx.fill();
			ctx.stroke();
			ctx.restore();
			continue;
		}
		drawZone(
			ctx,
			spellIndex,
			screen.x,
			screen.y,
			spell.radius * ARENA_LAYOUT.tile,
			elapsedMs
		);
	}
}

function drawDraggedCard(
	ctx: CanvasRenderingContext2D,
	content: Content,
	card: CardDef,
	pointer: { x: number; y: number }
) {
	const width = 64;
	const height = Math.round(width * (HUD.handHeight / HUD.cardWidth));
	const x = Math.max(4, Math.min(DESIGN.width - width - 4, pointer.x - width / 2));
	const y = Math.max(4, Math.min(DESIGN.height - height - 4, pointer.y - height - 28));
	ctx.save();
	ctx.globalAlpha = 0.9;
	ctx.shadowColor = 'rgba(0, 0, 0, 0.45)';
	ctx.shadowBlur = 12;
	ctx.fillStyle = 'rgba(15, 23, 42, 0.95)';
	ctx.beginPath();
	ctx.roundRect(x, y, width, height, 10);
	ctx.fill();
	ctx.shadowBlur = 0;
	ctx.strokeStyle = '#64748b';
	ctx.lineWidth = 1.5;
	ctx.stroke();
	drawCardIcon(ctx, card, content.definitions, x + 4, y + 3, width - 8, 0);
	ctx.restore();
}

function drawPendingDeploy(
	ctx: CanvasRenderingContext2D,
	content: Content,
	localSide: Side,
	pending: PendingDeployView
) {
	const point = worldToScreen(pending.x, pending.y, localSide);
	const width = 46;
	const height = Math.round(width * (HUD.handHeight / HUD.cardWidth));
	const x = point.x - width / 2;
	const y = point.y - height - 8;
	const progress = Math.max(0, Math.min(1, pending.progress));

	// carta translúcida indicando o local onde a tropa vai surgir
	ctx.save();
	ctx.globalAlpha = 0.42;
	ctx.fillStyle = 'rgba(15, 23, 42, 0.9)';
	ctx.beginPath();
	ctx.roundRect(x, y, width, height, 8);
	ctx.fill();
	ctx.strokeStyle = pending.side === localSide ? '#93c5fd' : '#fca5a5';
	ctx.lineWidth = 1.5;
	ctx.stroke();
	drawCardIcon(ctx, pending.card, content.definitions, x + 3, y + 2, width - 6, 0);
	ctx.restore();

	// barra de carregamento acima da carta
	const barHeight = 5;
	const barX = x;
	const barY = y - 10;
	ctx.save();
	ctx.fillStyle = 'rgba(2, 6, 23, 0.85)';
	ctx.beginPath();
	ctx.roundRect(barX, barY, width, barHeight, barHeight / 2);
	ctx.fill();
	if (progress > 0) {
		ctx.fillStyle = pending.side === localSide ? '#60a5fa' : '#f87171';
		ctx.beginPath();
		ctx.roundRect(barX, barY, width * progress, barHeight, barHeight / 2);
		ctx.fill();
	}
	ctx.strokeStyle = 'rgba(226, 232, 240, 0.45)';
	ctx.lineWidth = 1;
	ctx.beginPath();
	ctx.roundRect(barX, barY, width, barHeight, barHeight / 2);
	ctx.stroke();
	ctx.restore();
}

function drawGhost(
	ctx: CanvasRenderingContext2D,
	content: Content,
	interaction: Interaction,
	hover: { x: number; y: number }
) {
	if (interaction.selectedSlot === null) {
		return;
	}
	ctx.save();
	ctx.globalAlpha = 0.55;
	ctx.beginPath();
	ctx.arc(hover.x, hover.y, ARENA_LAYOUT.tile * 0.6, 0, Math.PI * 2);
	ctx.fillStyle = interaction.valid ? 'rgba(34, 197, 94, 0.25)' : 'rgba(239, 68, 68, 0.25)';
	ctx.fill();
	ctx.strokeStyle = interaction.valid ? '#22c55e' : '#ef4444';
	ctx.lineWidth = 2;
	ctx.stroke();
	ctx.restore();
}

export function renderFrame(
	ctx: CanvasRenderingContext2D,
	content: Content,
	match: MatchInfo,
	pair: RenderPair,
	interaction: Interaction,
	hud: HudRender,
	elapsedMs: number,
	views: Map<number, SpriteView>
) {
	ctx.clearRect(0, 0, DESIGN.width, DESIGN.height);
	const arena = content.definitions.arenas[match.arena] ?? Object.values(content.definitions.arenas)[0];
	if (!arena) {
		return;
	}
	drawArenaGround(ctx, arena, elapsedMs);

	// zonas no chão
	drawZones(ctx, content, pair, match.side, elapsedMs);
	if (interaction.selectedSlot !== null) {
		drawDeployOverlay(
			ctx,
			arena,
			match.side,
			interaction.cardKind,
			interaction.hover,
			interaction.valid
		);
	}

	drawTowersAndEntities(ctx, content, pair, match.side, elapsedMs, views);
	drawProjectiles(ctx, content, pair, match.side, elapsedMs);

	for (const pending of interaction.pending) {
		drawPendingDeploy(ctx, content, match.side, pending);
	}

	if (interaction.hover) {
		drawGhost(ctx, content, interaction, interaction.hover);
	}
	if (interaction.drag && interaction.dragCard) {
		drawDraggedCard(ctx, content, interaction.dragCard, interaction.drag);
	}

	drawHud(ctx, {
		snapshot: pair.latest,
		localSide: match.side,
		opponent: hud.opponent,
		username: hud.username,
		selectedSlot: hud.selectedSlot,
		elapsedMs
	});
}

export function battleFramePair(
	frames: { snapshot: Snapshot; at: number }[],
	now: number
): RenderPair | null {
	if (frames.length === 0) {
		return null;
	}
	const target = now - INTERPOLATION_MS;
	let prev = frames[0];
	let next = frames[frames.length - 1];
	for (let index = 0; index < frames.length; index += 1) {
		if (frames[index].at <= target) {
			prev = frames[index];
			next = frames[index + 1] ?? frames[index];
		}
	}
	if (frames.length === 1) {
		return {
			prev: frames[0].snapshot,
			next: frames[0].snapshot,
			alpha: 0,
			latest: frames[0].snapshot
		};
	}
	const span = Math.max(1, next.at - prev.at);
	const alpha = Math.max(0, Math.min(1, (target - prev.at) / span));
	return {
		prev: prev.snapshot,
		next: next.snapshot,
		alpha,
		latest: frames[frames.length - 1].snapshot
	};
}
