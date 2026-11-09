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
			const green = 102 + Math.floor(noise * 10) + shade * 3;
			context.fillStyle = `rgb(${36 + shade * 3}, ${green}, ${58 + shade * 4})`;
			context.fillRect(
				rect.x + gx * tileW,
				rect.y + gy * tileH,
				tileW + 1,
				tileH + 1
			);
		}
	}

	// caminhos de pedra ligando pontes, princesas e rei
	const path = 'rgba(146, 110, 64, 0.38)';
	const pathEdge = 'rgba(90, 62, 30, 0.35)';
	const bridgeCenters = arena.bridges.map(([a, b]) => (a + b) * 0.5);
	const yOf = (worldY: number) => rect.y + worldY * tileH;
	for (const center of bridgeCenters) {
		const px = rect.x + center * tileW;
		context.fillStyle = path;
		context.fillRect(px - 13, yOf(6.5), 26, yOf(15.5) - yOf(6.5));
		context.fillRect(px - 13, yOf(16.5), 26, yOf(25.5) - yOf(16.5));
		context.strokeStyle = pathEdge;
		context.lineWidth = 2;
		context.strokeRect(px - 13, yOf(6.5), 26, yOf(15.5) - yOf(6.5));
		context.strokeRect(px - 13, yOf(16.5), 26, yOf(25.5) - yOf(16.5));
	}
	for (const [worldY, towerY] of [
		[15.5, 6.5],
		[16.5, 25.5]
	] as const) {
		const x1 = rect.x + 3.5 * tileW;
		const x2 = rect.x + 14.5 * tileW;
		context.fillStyle = path;
		context.fillRect(x1, yOf(towerY) - 11, x2 - x1, 22);
		context.strokeStyle = pathEdge;
		context.lineWidth = 2;
		context.strokeRect(x1, yOf(towerY) - 11, x2 - x1, 22);
	}

	// bordas de pedra
	context.fillStyle = 'rgba(30, 41, 59, 0.85)';
	for (let bx = rect.x + 6; bx < rect.x + rect.w; bx += 34) {
		context.fillRect(bx, rect.y, 28, 8);
		context.fillRect(bx, rect.y + rect.h - 8, 28, 8);
		context.strokeStyle = 'rgba(15, 23, 42, 0.7)';
		context.lineWidth = 1;
		context.strokeRect(bx, rect.y, 28, 8);
		context.strokeRect(bx, rect.y + rect.h - 8, 28, 8);
	}
	for (let by = rect.y + 20; by < rect.y + rect.h - 20; by += 34) {
		context.fillRect(rect.x, by, 7, 28);
		context.fillRect(rect.x + rect.w - 7, by, 7, 28);
	}

	// vegetação e pedras decorativas
	const tree = (wx: number, wy: number, size: number) => {
		const tx = rect.x + wx * tileW;
		const ty = rect.y + wy * tileH;
		context.fillStyle = 'rgba(0,0,0,0.18)';
		context.beginPath();
		context.ellipse(tx, ty + 6, size * 0.8, size * 0.3, 0, 0, Math.PI * 2);
		context.fill();
		context.fillStyle = '#3f2d1c';
		context.fillRect(tx - 2, ty - size * 0.2, 4, size * 0.45);
		for (let layer = 0; layer < 3; layer += 1) {
			context.beginPath();
			context.arc(tx, ty - size * (0.4 + layer * 0.22), size * (0.62 - layer * 0.12), 0, Math.PI * 2);
			context.fillStyle = layer % 2 === 0 ? '#14532d' : '#166534';
			context.fill();
			context.strokeStyle = 'rgba(6, 78, 59, 0.6)';
			context.lineWidth = 1.5;
			context.stroke();
		}
	};
	tree(0.9, 1.0, 22);
	tree(17.1, 1.0, 22);
	tree(0.9, 31.0, 24);
	tree(17.1, 31.0, 24);
	tree(0.9, 12.4, 17);
	tree(17.1, 19.6, 17);

	for (let index = 0; index < 26; index += 1) {
		const wx = 1.4 + ((index * 7.31) % 15.2);
		const wy = 2.0 + ((index * 11.73) % 27.5);
		if (wy > 14.8 && wy < 17.2) {
			continue;
		}
		const gx = rect.x + wx * tileW;
		const gy = rect.y + wy * tileH;
		context.strokeStyle = 'rgba(22, 101, 52, 0.55)';
		context.lineWidth = 1.4;
		for (let blade = -1; blade <= 1; blade += 1) {
			context.beginPath();
			context.moveTo(gx + blade * 3, gy);
			context.lineTo(gx + blade * 4, gy - 6 - Math.abs(blade) * 2);
			context.stroke();
		}
		if (index % 5 === 0) {
			context.beginPath();
			context.arc(gx + 2, gy - 3, 2.2, 0, Math.PI * 2);
			context.fillStyle = index % 10 === 0 ? '#fde047' : '#fda4af';
			context.fill();
		}
	}

	// faixas de lane
	context.fillStyle = 'rgba(15, 23, 42, 0.12)';
	context.fillRect(rect.x + rect.w * 0.18, rect.y, rect.w * 0.03, rect.h);
	context.fillRect(rect.x + rect.w * 0.79, rect.y, rect.w * 0.03, rect.h);

	// marcas d'água reais com coroas no centro de cada metade
	context.save();
	context.globalAlpha = 0.09;
	const crest = goldGradient(context, rect.x, rect.y, rect.h);
	drawCrownShape(context, rect.x + rect.w * 0.5, rect.y + rect.h * 0.3, rect.w * 0.2, rect.h * 0.1, crest);
	drawCrownShape(context, rect.x + rect.w * 0.5, rect.y + rect.h * 0.7, rect.w * 0.2, rect.h * 0.1, crest);
	context.restore();

	// torches e estandartes nas laterais
	const torch = (tx: number, ty: number) => {
		context.fillStyle = '#5b3b12';
		context.fillRect(tx - 2, ty - 18, 4, 20);
		context.fillStyle = '#f7d45c';
		context.fillRect(tx - 4, ty - 22, 8, 4);
		const glow = context.createRadialGradient(tx, ty - 28, 1, tx, ty - 28, 18);
		glow.addColorStop(0, 'rgba(255, 222, 120, 0.95)');
		glow.addColorStop(0.45, 'rgba(247, 160, 30, 0.5)');
		glow.addColorStop(1, 'rgba(247, 160, 30, 0)');
		context.fillStyle = glow;
		context.fillRect(tx - 18, ty - 48, 36, 34);
		context.beginPath();
		context.moveTo(tx, ty - 34);
		context.quadraticCurveTo(tx + 6, ty - 26, tx, ty - 18);
		context.quadraticCurveTo(tx - 6, ty - 26, tx, ty - 34);
		context.fillStyle = '#ffbe3d';
		context.fill();
	};
	torch(rect.x + 14, rect.y + 100);
	torch(rect.x + rect.w - 14, rect.y + 100);
	torch(rect.x + 14, rect.y + rect.h - 26);
	torch(rect.x + rect.w - 14, rect.y + rect.h - 26);

	// faixa dourada nas bordas
	context.strokeStyle = 'rgba(247, 212, 92, 0.4)';
	context.lineWidth = 1.5;
	context.strokeRect(rect.x + 10, rect.y + 10, rect.w - 20, rect.h - 20);

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
	// ondas animadas (sem cobrir as pontes)
	context.save();
	context.beginPath();
	context.rect(rect.x, riverTop, rect.w, riverBottom - riverTop);
	const tileWidth = ARENA_LAYOUT.tile;
	const tileHeight = ARENA_LAYOUT.tile * ARENA_LAYOUT.depth;
	for (const [left, right] of arena.bridges) {
		const bx = rect.x + left * tileWidth;
		const bw = (right - left) * tileWidth;
		const top = riverTop - tileHeight * 0.45;
		const height = riverBottom - riverTop + tileHeight * 0.9;
		context.rect(bx, top, bw, height);
	}
	context.clip('evenodd');
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

function goldGradient(context: CanvasRenderingContext2D, x: number, y: number, height: number) {
	const gradient = context.createLinearGradient(x, y, x, y + height);
	gradient.addColorStop(0, '#fff6c4');
	gradient.addColorStop(0.3, '#f7d45c');
	gradient.addColorStop(0.62, '#d9a521');
	gradient.addColorStop(1, '#8a5206');
	return gradient;
}

function drawCrownShape(
	context: CanvasRenderingContext2D,
	x: number,
	y: number,
	width: number,
	height: number,
	fill: string | CanvasGradient
) {
	context.beginPath();
	context.moveTo(x - width / 2, y + height * 0.6);
	context.lineTo(x - width * 0.56, y - height * 0.15);
	context.lineTo(x - width * 0.2, y + height * 0.18);
	context.lineTo(x, y - height * 0.6);
	context.lineTo(x + width * 0.2, y + height * 0.18);
	context.lineTo(x + width * 0.56, y - height * 0.15);
	context.lineTo(x + width / 2, y + height * 0.6);
	context.closePath();
	context.fillStyle = fill;
	context.fill();
	context.strokeStyle = '#5d3603';
	context.lineWidth = 1.2;
	context.stroke();
}

function drawRoyalBust(
	context: CanvasRenderingContext2D,
	x: number,
	y: number,
	scale: number,
	queen: boolean
) {
	context.save();
	context.translate(x, y);
	context.scale(scale, scale);
	const gold = context.createLinearGradient(0, -16, 0, 12);
	gold.addColorStop(0, '#fff2a8');
	gold.addColorStop(0.45, '#f1c63e');
	gold.addColorStop(1, '#8c5606');
	// manto
	context.beginPath();
	context.moveTo(-10, 12);
	context.lineTo(-7, -2);
	context.lineTo(7, -2);
	context.lineTo(10, 12);
	context.closePath();
	context.fillStyle = queen ? '#8f1d2c' : '#1d4ed8';
	context.fill();
	context.strokeStyle = '#2a070c';
	context.lineWidth = 1;
	context.stroke();
	// ombros/armadura dourada
	context.beginPath();
	context.moveTo(-8, 6);
	context.quadraticCurveTo(0, 0, 8, 6);
	context.lineTo(6, 11);
	context.lineTo(-6, 11);
	context.closePath();
	context.fillStyle = gold;
	context.fill();
	context.strokeStyle = '#5d3603';
	context.stroke();
	// cabeça
	context.beginPath();
	context.arc(0, -7, 5.2, 0, Math.PI * 2);
	context.fillStyle = '#f8dc9a';
	context.fill();
	context.strokeStyle = '#5d3603';
	context.stroke();
	// cabelo/veu
	if (queen) {
		context.beginPath();
		context.arc(0, -8, 7, Math.PI * 0.95, Math.PI * 2.05);
		context.strokeStyle = '#f7d45c';
		context.lineWidth = 2;
		context.stroke();
	}
	// coroa
	drawCrownShape(context, 0, -15, 12, 8, gold);
	// cetro
	context.beginPath();
	context.moveTo(8, 2);
	context.lineTo(12, -12);
	context.strokeStyle = '#f7d45c';
	context.lineWidth = 1.6;
	context.stroke();
	context.beginPath();
	context.arc(12, -13, 2, 0, Math.PI * 2);
	context.fillStyle = queen ? '#ff9aa7' : '#7dd3fc';
	context.fill();
	context.restore();
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
	const width = king ? 76 : 60;
	const height = king ? 106 : 82;
	const baseY = y;
	const topY = baseY - height;
	context.save();
	context.globalAlpha = active || role !== 'King' ? 1 : 0.78;

	// sombra
	context.beginPath();
	context.ellipse(x, baseY + 4, width * 0.78, width * 0.32, 0, 0, Math.PI * 2);
	context.fillStyle = 'rgba(0, 0, 0, 0.42)';
	context.fill();

	// plinto de pedra com filete dourado
	context.beginPath();
	context.moveTo(x - width * 0.72, baseY);
	context.lineTo(x - width * 0.56, baseY - height * 0.14);
	context.lineTo(x + width * 0.56, baseY - height * 0.14);
	context.lineTo(x + width * 0.72, baseY);
	context.closePath();
	const plinth = context.createLinearGradient(x, baseY - height * 0.14, x, baseY);
	plinth.addColorStop(0, '#8c98a8');
	plinth.addColorStop(1, '#39424f');
	context.fillStyle = plinth;
	context.fill();
	context.strokeStyle = '#0d1420';
	context.lineWidth = 2.5;
	context.stroke();
	context.fillStyle = goldGradient(context, x, baseY - height * 0.15, 6);
	context.fillRect(x - width * 0.58, baseY - height * 0.16, width * 1.16, 3);

	// corpo de pedra
	const bodyTop = topY + height * 0.23;
	const bodyHeight = height * 0.57;
	const bodyLeft = x - width * 0.43;
	const bodyWidth = width * 0.86;
	const stone = context.createLinearGradient(x, bodyTop, x, bodyTop + bodyHeight);
	stone.addColorStop(0, '#aab4c2');
	stone.addColorStop(0.4, '#7b8899');
	stone.addColorStop(1, '#3d4958');
	context.beginPath();
	context.roundRect(bodyLeft, bodyTop, bodyWidth, bodyHeight, king ? 10 : 7);
	context.fillStyle = stone;
	context.fill();
	context.strokeStyle = '#0d1420';
	context.lineWidth = 2.5;
	context.stroke();

	// blocos de pedra
	context.save();
	context.beginPath();
	context.roundRect(bodyLeft, bodyTop, bodyWidth, bodyHeight, king ? 10 : 7);
	context.clip();
	context.strokeStyle = 'rgba(24, 33, 46, 0.32)';
	context.lineWidth = 1;
	for (let row = 1; row < 5; row += 1) {
		const lineY = bodyTop + (bodyHeight / 5) * row;
		context.beginPath();
		context.moveTo(bodyLeft, lineY);
		context.lineTo(bodyLeft + bodyWidth, lineY);
		context.stroke();
	}
	for (let row = 0; row < 5; row += 1) {
		const lineY = bodyTop + (bodyHeight / 5) * row;
		const offset = row % 2 === 0 ? bodyWidth * 0.33 : bodyWidth * 0.66;
		context.beginPath();
		context.moveTo(bodyLeft + offset, lineY);
		context.lineTo(bodyLeft + offset, lineY + bodyHeight / 5);
		context.stroke();
	}
	context.restore();

	// pilastras douradas
	context.fillStyle = goldGradient(context, x, bodyTop, bodyHeight);
	context.fillRect(bodyLeft + 2, bodyTop + 3, 3.5, bodyHeight - 6);
	context.fillRect(bodyLeft + bodyWidth - 5.5, bodyTop + 3, 3.5, bodyHeight - 6);

	// janelas com brilho
	for (const offset of [-width * 0.17, width * 0.17]) {
		const wx = x + offset;
		const wy = bodyTop + bodyHeight * 0.18;
		context.beginPath();
		context.moveTo(wx, wy);
		context.lineTo(wx, wy + 10);
		context.arc(wx, wy + 10, 4.2, Math.PI, 0, true);
		context.closePath();
		context.fillStyle = '#ffd76a';
		context.shadowColor = 'rgba(247, 212, 92, 0.75)';
		context.shadowBlur = 8;
		context.fill();
		context.shadowBlur = 0;
		context.strokeStyle = '#3a2704';
		context.lineWidth = 1;
		context.stroke();
	}

	// estandarte do time com brasão
	const bannerTop = bodyTop + bodyHeight * 0.34;
	const bannerHeight = bodyHeight * 0.38;
	context.beginPath();
	context.moveTo(x - width * 0.27, bannerTop);
	context.lineTo(x + width * 0.27, bannerTop);
	context.lineTo(x + width * 0.27, bannerTop + bannerHeight);
	context.lineTo(x, bannerTop + bannerHeight + 7);
	context.lineTo(x - width * 0.27, bannerTop + bannerHeight);
	context.closePath();
	const banner = context.createLinearGradient(x, bannerTop, x, bannerTop + bannerHeight);
	banner.addColorStop(0, team.light);
	banner.addColorStop(0.5, team.primary);
	banner.addColorStop(1, team.dark);
	context.fillStyle = banner;
	context.fill();
	context.strokeStyle = '#e6bb4c';
	context.lineWidth = 1.5;
	context.stroke();
	drawCrownShape(context, x, bannerTop + bannerHeight * 0.42, width * 0.2, width * 0.13, goldGradient(context, x, bannerTop, bannerHeight));

	// estátua dourada do rei/rainha
	drawRoyalBust(context, x, bodyTop + bodyHeight - 7, king ? 0.95 : 0.68, !king);

	// ameias
	for (let index = -1; index <= 1; index += 1) {
		const bx = x + index * width * 0.28 - width * 0.085;
		const by = topY + height * 0.12;
		context.fillStyle = '#9aa6b5';
		context.fillRect(bx, by, width * 0.17, height * 0.11);
		context.strokeStyle = '#0d1420';
		context.lineWidth = 1.8;
		context.strokeRect(bx, by, width * 0.17, height * 0.11);
		context.fillStyle = goldGradient(context, bx, by, 3);
		context.fillRect(bx, by, width * 0.17, 2.5);
	}

	// telhado e coroa
	if (king) {
		context.beginPath();
		context.moveTo(x - width * 0.44, topY + height * 0.13);
		context.lineTo(x, topY - height * 0.08);
		context.lineTo(x + width * 0.44, topY + height * 0.13);
		context.closePath();
		const roof = context.createLinearGradient(x, topY - height * 0.08, x, topY + height * 0.13);
		roof.addColorStop(0, '#f7d45c');
		roof.addColorStop(0.55, '#c98b0d');
		roof.addColorStop(1, '#6b3c02');
		context.fillStyle = roof;
		context.fill();
		context.strokeStyle = '#4a2c02';
		context.lineWidth = 2;
		context.stroke();
		drawCrownShape(context, x, topY - height * 0.19, width * 0.38, height * 0.14, goldGradient(context, x, topY - height * 0.28, height * 0.2));
		context.beginPath();
		context.arc(x, topY - height * 0.3, 2.4, 0, Math.PI * 2);
		context.fillStyle = '#ff5a6f';
		context.fill();
	} else {
		context.beginPath();
		context.moveTo(x - width * 0.52, topY + height * 0.14);
		context.lineTo(x, topY - height * 0.05);
		context.lineTo(x + width * 0.52, topY + height * 0.14);
		context.closePath();
		const roof = context.createLinearGradient(x, topY - height * 0.05, x, topY + height * 0.14);
		roof.addColorStop(0, team.light);
		roof.addColorStop(0.55, team.primary);
		roof.addColorStop(1, team.dark);
		context.fillStyle = roof;
		context.fill();
		context.strokeStyle = '#0d1420';
		context.lineWidth = 2;
		context.stroke();
		context.beginPath();
		context.moveTo(x - width * 0.52, topY + height * 0.14);
		context.lineTo(x + width * 0.52, topY + height * 0.14);
		context.strokeStyle = 'rgba(255, 230, 122, 0.75)';
		context.lineWidth = 2;
		context.stroke();
		drawCrownShape(context, x, topY + height * 0.02, width * 0.22, height * 0.09, goldGradient(context, x, topY - height * 0.05, height * 0.12));
	}

	// porta
	context.beginPath();
	context.arc(x, baseY - height * 0.1, width * 0.15, Math.PI, Math.PI * 2);
	context.lineTo(x + width * 0.15, baseY);
	context.lineTo(x - width * 0.15, baseY);
	context.closePath();
	context.fillStyle = '#161f2c';
	context.fill();
	context.strokeStyle = '#c9962e';
	context.lineWidth = 1.5;
	context.stroke();

	// barra de vida em moldura dourada
	if (hp < maxHp) {
		const barWidth = width * 1.02;
		const barY = topY - height * 0.36;
		context.fillStyle = 'rgba(8, 13, 24, 0.88)';
		context.fillRect(x - barWidth / 2 - 3, barY - 3, barWidth + 6, 12);
		context.strokeStyle = 'rgba(247, 212, 92, 0.85)';
		context.lineWidth = 1.2;
		context.strokeRect(x - barWidth / 2 - 3, barY - 3, barWidth + 6, 12);
		const fill = context.createLinearGradient(x - barWidth / 2, barY, x - barWidth / 2, barY + 6);
		fill.addColorStop(0, team.light);
		fill.addColorStop(1, team.dark);
		context.fillStyle = fill;
		context.fillRect(x - barWidth / 2, barY, barWidth * (hp / maxHp), 6);
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
