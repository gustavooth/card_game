import type { ArenaDef, Side, TowerDef } from '#lib/types';
import { ARENA_HEIGHT, ARENA_LAYOUT, ARENA_WIDTH, DESIGN, TEAM_COLORS } from './constants';
import { worldToScreen } from './projection';

let groundCache: { arena: string; canvas: OffscreenCanvas | HTMLCanvasElement } | null = null;

function createCanvas(width: number, height: number): OffscreenCanvas | HTMLCanvasElement {
	if (typeof OffscreenCanvas !== 'undefined') {
		return new OffscreenCanvas(width, height);
	}
	const canvas = document.createElement('canvas');
	canvas.width = width;
	canvas.height = height;
	return canvas;
}

function arenaRect(arena: ArenaDef) {
	return {
		x: ARENA_LAYOUT.x,
		y: ARENA_LAYOUT.y,
		w: arena.width * ARENA_LAYOUT.tile,
		h: arena.height * ARENA_LAYOUT.tile * ARENA_LAYOUT.depth
	};
}

function drawGround(context: CanvasRenderingContext2D, arena: ArenaDef) {
	const rect = arenaRect(arena);
	const riverTop = ARENA_LAYOUT.y + arena.river_top * ARENA_LAYOUT.tile * ARENA_LAYOUT.depth;
	const riverBottom = ARENA_LAYOUT.y + arena.river_bottom * ARENA_LAYOUT.tile * ARENA_LAYOUT.depth;

	const backdrop = context.createLinearGradient(0, 0, 0, DESIGN.height);
	backdrop.addColorStop(0, '#0b1220');
	backdrop.addColorStop(0.45, '#111c33');
	backdrop.addColorStop(1, '#0a0f1c');
	context.fillStyle = backdrop;
	context.fillRect(0, 0, DESIGN.width, DESIGN.height);

	context.save();
	context.beginPath();
	context.roundRect(rect.x - 6, rect.y - 6, rect.w + 12, rect.h + 12, 18);
	context.fillStyle = '#1e293b';
	context.fill();
	context.strokeStyle = '#0f172a';
	context.lineWidth = 4;
	context.stroke();
	context.beginPath();
	context.roundRect(rect.x, rect.y, rect.w, rect.h, 12);
	context.clip();

	// grama
	const tileW = ARENA_LAYOUT.tile;
	const tileH = ARENA_LAYOUT.tile * ARENA_LAYOUT.depth;
	for (let gy = 0; gy < ARENA_HEIGHT; gy += 1) {
		for (let gx = 0; gx < ARENA_WIDTH; gx += 1) {
			const noise = Math.abs(Math.sin(gx * 12.9898 + gy * 78.233) * 43758.5453) % 1;
			const shade = (gx + gy) % 2 === 0 ? 0 : 1;
			const green = 96 + Math.floor(noise * 18) + shade * 6;
			context.fillStyle = `rgb(${34 + shade * 5}, ${green}, ${58 + shade * 6})`;
			context.fillRect(
				rect.x + gx * tileW,
				rect.y + gy * tileH,
				tileW + 1,
				tileH + 1
			);
		}
	}

	// faixas de lane
	context.fillStyle = 'rgba(15, 23, 42, 0.12)';
	context.fillRect(rect.x + rect.w * 0.18, rect.y, rect.w * 0.03, rect.h);
	context.fillRect(rect.x + rect.w * 0.79, rect.y, rect.w * 0.03, rect.h);

	// rio
	const river = context.createLinearGradient(0, riverTop, 0, riverBottom);
	river.addColorStop(0, '#0ea5e9');
	river.addColorStop(0.5, '#0284c7');
	river.addColorStop(1, '#0369a1');
	context.fillStyle = river;
	context.fillRect(rect.x, riverTop, rect.w, riverBottom - riverTop);

	// margens
	context.fillStyle = 'rgba(120, 83, 40, 0.9)';
	context.fillRect(rect.x, riverTop - 3, rect.w, 4);
	context.fillRect(rect.x, riverBottom - 1, rect.w, 4);

	// pontes
	for (const [left, right] of arena.bridges) {
		const bx = rect.x + left * tileW;
		const bw = (right - left) * tileW;
		const top = riverTop - tileH * 0.45;
		const height = riverBottom - riverTop + tileH * 0.9;
		context.fillStyle = '#8b5a2b';
		context.fillRect(bx, top, bw, height);
		context.strokeStyle = '#4a2c12';
		context.lineWidth = 2;
		context.strokeRect(bx, top, bw, height);
		context.strokeStyle = 'rgba(74, 44, 18, 0.55)';
		context.lineWidth = 1.5;
		for (let py = top + 5; py < top + height; py += 8) {
			context.beginPath();
			context.moveTo(bx + 2, py);
			context.lineTo(bx + bw - 2, py);
			context.stroke();
		}
		context.fillStyle = 'rgba(255, 255, 255, 0.06)';
		context.fillRect(bx, top, bw, 3);
	}

	// vinheta e profundidade
	const vignette = context.createRadialGradient(
		rect.x + rect.w / 2,
		rect.y + rect.h / 2,
		rect.w * 0.2,
		rect.x + rect.w / 2,
		rect.y + rect.h / 2,
		rect.w * 0.85
	);
	vignette.addColorStop(0, 'rgba(0,0,0,0)');
	vignette.addColorStop(1, 'rgba(0,0,0,0.32)');
	context.fillStyle = vignette;
	context.fillRect(rect.x, rect.y, rect.w, rect.h);

	context.restore();

	// paredes laterais
	context.fillStyle = 'rgba(226, 232, 240, 0.06)';
	context.fillRect(rect.x - 6, rect.y, 6, rect.h);
	context.fillRect(rect.x + rect.w, rect.y, 6, rect.h);
	context.fillStyle = 'rgba(15, 23, 42, 0.5)';
	context.fillRect(rect.x, rect.y - 6, rect.w, 6);
}

function getGround(arena: ArenaDef) {
	if (groundCache && groundCache.arena === arena.slug) {
		return groundCache.canvas;
	}
	const canvas = createCanvas(DESIGN.width, DESIGN.height);
	const context = canvas.getContext('2d') as
		| OffscreenCanvasRenderingContext2D
		| CanvasRenderingContext2D
		| null;
	if (!context) {
		return canvas;
	}
	drawGround(context as CanvasRenderingContext2D, arena);
	groundCache = { arena: arena.slug, canvas };
	return canvas;
}

export function drawArenaGround(
	context: CanvasRenderingContext2D,
	arena: ArenaDef,
	elapsedMs: number
) {
	context.drawImage(getGround(arena) as CanvasImageSource, 0, 0);
	const rect = arenaRect(arena);
	const riverTop = ARENA_LAYOUT.y + arena.river_top * ARENA_LAYOUT.tile * ARENA_LAYOUT.depth;
	const riverBottom =
		ARENA_LAYOUT.y + arena.river_bottom * ARENA_LAYOUT.tile * ARENA_LAYOUT.depth;
	// ondas animadas
	context.save();
	context.beginPath();
	context.rect(rect.x, riverTop, rect.w, riverBottom - riverTop);
	context.clip();
	context.globalAlpha = 0.22;
	for (let line = 0; line < 3; line += 1) {
		context.beginPath();
		const offset = ((elapsedMs * 0.02 + line * 40) % (rect.w + 80)) - 40;
		for (let x = rect.x - 40; x <= rect.x + rect.w + 40; x += 8) {
			const y =
				riverTop +
				(riverBottom - riverTop) * (0.3 + line * 0.2) +
				Math.sin((x + elapsedMs * 0.05) * 0.08 + line) * 2;
			if (x === rect.x - 40) {
				context.moveTo(x + offset * 0.15, y);
			} else {
				context.lineTo(x + offset * 0.15, y);
			}
		}
		context.strokeStyle = '#bae6fd';
		context.lineWidth = 2;
		context.stroke();
	}
	context.restore();
}

export function drawDeployOverlay(
	context: CanvasRenderingContext2D,
	arena: ArenaDef,
	side: Side,
	cardKind: 'troop' | 'spell' | 'building',
	hover: { x: number; y: number } | null,
	valid: boolean
) {
	const rect = arenaRect(arena);
	context.save();
	context.beginPath();
	context.rect(rect.x, rect.y, rect.w, rect.h);
	context.clip();
	const riverTop = ARENA_LAYOUT.y + arena.river_top * ARENA_LAYOUT.tile * ARENA_LAYOUT.depth;
	const riverBottom =
		ARENA_LAYOUT.y + arena.river_bottom * ARENA_LAYOUT.tile * ARENA_LAYOUT.depth;
	if (cardKind === 'spell') {
		context.fillStyle = 'rgba(168, 85, 247, 0.08)';
		context.fillRect(rect.x, rect.y, rect.w, rect.h);
	} else {
		context.fillStyle = 'rgba(59, 130, 246, 0.1)';
		if (side === 0) {
			context.fillRect(rect.x, riverBottom, rect.w, rect.y + rect.h - riverBottom);
		} else {
			context.fillRect(rect.x, rect.y, rect.w, riverTop - rect.y);
		}
	}
	context.restore();
	if (hover) {
		context.save();
		context.beginPath();
		context.arc(hover.x, hover.y, ARENA_LAYOUT.tile * 0.75, 0, Math.PI * 2);
		context.fillStyle = valid ? 'rgba(34, 197, 94, 0.25)' : 'rgba(239, 68, 68, 0.25)';
		context.fill();
		context.strokeStyle = valid ? '#22c55e' : '#ef4444';
		context.lineWidth = 2;
		context.stroke();
		context.restore();
	}
}

export function drawTower(
	context: CanvasRenderingContext2D,
	tower: TowerDef,
	side: Side,
	x: number,
	y: number,
	hp: number,
	maxHp: number,
	active: boolean,
	role: string
) {
	const team = side === 0 ? TEAM_COLORS.blue : TEAM_COLORS.red;
	const king = role === 'King';
	const width = king ? 68 : 54;
	const height = king ? 96 : 72;
	const baseY = y;
	const topY = baseY - height;
	context.save();
	context.globalAlpha = active || role !== 'King' ? 1 : 0.72;

	// sombra
	context.beginPath();
	context.ellipse(x, baseY + 3, width * 0.7, width * 0.3, 0, 0, Math.PI * 2);
	context.fillStyle = 'rgba(0, 0, 0, 0.3)';
	context.fill();

	// base
	context.beginPath();
	context.moveTo(x - width * 0.62, baseY);
	context.lineTo(x - width * 0.5, topY + height * 0.24);
	context.lineTo(x + width * 0.5, topY + height * 0.24);
	context.lineTo(x + width * 0.62, baseY);
	context.closePath();
	context.fillStyle = '#475569';
	context.fill();
	context.strokeStyle = '#0f172a';
	context.lineWidth = 3;
	context.stroke();

	// corpo
	context.beginPath();
	context.roundRect(x - width * 0.42, topY + height * 0.2, width * 0.84, height * 0.62, 8);
	context.fillStyle = '#64748b';
	context.fill();
	context.stroke();

	// faixa de time
	context.fillStyle = team.primary;
	context.fillRect(x - width * 0.42, topY + height * 0.34, width * 0.84, height * 0.2);
	context.strokeStyle = team.dark;
	context.lineWidth = 2;
	context.strokeRect(x - width * 0.42, topY + height * 0.34, width * 0.84, height * 0.2);

	// ameias
	for (let index = -1; index <= 1; index += 1) {
		context.fillStyle = '#94a3b8';
		context.fillRect(x + index * width * 0.28 - width * 0.09, topY + height * 0.08, width * 0.18, height * 0.14);
		context.strokeStyle = '#0f172a';
		context.lineWidth = 2;
		context.strokeRect(x + index * width * 0.28 - width * 0.09, topY + height * 0.08, width * 0.18, height * 0.14);
	}

	// telhado / coroa
	if (king) {
		context.beginPath();
		context.moveTo(x - width * 0.34, topY + height * 0.08);
		context.lineTo(x, topY - height * 0.12);
		context.lineTo(x + width * 0.34, topY + height * 0.08);
		context.closePath();
		context.fillStyle = team.dark;
		context.fill();
		context.stroke();
		context.fillStyle = '#facc15';
		context.beginPath();
		context.moveTo(x - 10, topY - height * 0.1);
		context.lineTo(x - 7, topY - height * 0.2);
		context.lineTo(x, topY - height * 0.12);
		context.lineTo(x + 7, topY - height * 0.2);
		context.lineTo(x + 10, topY - height * 0.1);
		context.closePath();
		context.fill();
		context.strokeStyle = '#a16207';
		context.lineWidth = 1.5;
		context.stroke();
	} else {
		context.beginPath();
		context.moveTo(x - width * 0.5, topY + height * 0.1);
		context.lineTo(x, topY - height * 0.05);
		context.lineTo(x + width * 0.5, topY + height * 0.1);
		context.closePath();
		context.fillStyle = team.dark;
		context.fill();
		context.stroke();
	}

	// porta
	context.beginPath();
	context.arc(x, baseY - height * 0.12, width * 0.13, Math.PI, Math.PI * 2);
	context.lineTo(x + width * 0.13, baseY);
	context.lineTo(x - width * 0.13, baseY);
	context.closePath();
	context.fillStyle = '#1e293b';
	context.fill();
	context.stroke();

	// barra de vida
	if (hp < maxHp) {
		const barWidth = width * 0.95;
		const barY = topY - height * 0.28;
		context.fillStyle = 'rgba(15, 23, 42, 0.8)';
		context.fillRect(x - barWidth / 2 - 2, barY - 2, barWidth + 4, 10);
		context.fillStyle = side === 0 ? '#3b82f6' : '#ef4444';
		context.fillRect(x - barWidth / 2, barY, barWidth * (hp / maxHp), 6);
		context.strokeStyle = '#0f172a';
		context.lineWidth = 1;
		context.strokeRect(x - barWidth / 2 - 2, barY - 2, barWidth + 4, 10);
	}
	context.restore();
}

export function towerScreenPosition(
	x: number,
	y: number,
	side: Side
): { x: number; y: number } {
	return worldToScreen(x, y, side);
}

export function arenaBounds() {
	return arenaRect({ width: 18, height: 32 } as ArenaDef);
}
