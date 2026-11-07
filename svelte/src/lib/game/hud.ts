import { get } from 'svelte/store';
import type { CardDef, Content, Side, Snapshot } from '#lib/types';
import { content } from '#lib/stores/content';
import { i18n } from '#lib/i18n';
import { ARENA_LAYOUT, DESIGN, HUD, RARITY_COLORS } from './constants';
import { drawCardIcon } from './cardArt';

function roundRect(
	ctx: CanvasRenderingContext2D,
	x: number,
	y: number,
	w: number,
	h: number,
	r: number
) {
	ctx.beginPath();
	ctx.roundRect(x, y, w, h, r);
}

function drawCrown(ctx: CanvasRenderingContext2D, x: number, y: number, color: string) {
	ctx.save();
	ctx.beginPath();
	ctx.moveTo(x - 8, y + 6);
	ctx.lineTo(x - 9, y - 4);
	ctx.lineTo(x - 3, y + 1);
	ctx.lineTo(x, y - 7);
	ctx.lineTo(x + 3, y + 1);
	ctx.lineTo(x + 9, y - 4);
	ctx.lineTo(x + 8, y + 6);
	ctx.closePath();
	ctx.fillStyle = color;
	ctx.fill();
	ctx.strokeStyle = '#78350f';
	ctx.lineWidth = 1.5;
	ctx.stroke();
	ctx.restore();
}

export function formatTime(remainingMs: number): string {
	const total = Math.max(0, Math.ceil(remainingMs / 1000));
	const minutes = Math.floor(total / 60);
	const seconds = total % 60;
	return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

function truncate(ctx: CanvasRenderingContext2D, text: string, maxWidth: number): string {
	if (ctx.measureText(text).width <= maxWidth) {
		return text;
	}
	let value = text;
	while (value.length > 3 && ctx.measureText(`${value}…`).width > maxWidth) {
		value = value.slice(0, -1);
	}
	return `${value}…`;
}

export interface HudOptions {
	snapshot: Snapshot;
	localSide: Side;
	opponent: string;
	username: string;
	selectedSlot: number | null;
	elapsedMs: number;
}

export function handLayout() {
	const cardWidth = HUD.cardWidth;
	const gap = HUD.cardGap;
	const startX = 78;
	return {
		cardWidth,
		cardHeight: HUD.handHeight,
		gap,
		startX,
		y: HUD.handY
	};
}

export function drawHud(ctx: CanvasRenderingContext2D, options: HudOptions) {
	const literal = get(i18n);
	const loaded: Content | null = get(content);
	if (!loaded) {
		return;
	}
	const snapshot = options.snapshot;
	const local = snapshot.p.find((player) => player.s === options.localSide) ?? snapshot.p[0];
	const enemy =
		snapshot.p.find((player) => player.s !== options.localSide) ??
		snapshot.p[1] ??
		snapshot.p[0];

	// barra superior
	ctx.save();
	const gradient = ctx.createLinearGradient(0, 0, 0, 72);
	gradient.addColorStop(0, 'rgba(2, 6, 23, 0.92)');
	gradient.addColorStop(1, 'rgba(2, 6, 23, 0.15)');
	ctx.fillStyle = gradient;
	ctx.fillRect(0, 0, DESIGN.width, 84);

	ctx.font = 'bold 17px system-ui, sans-serif';
	ctx.textAlign = 'left';
	ctx.fillStyle = '#fca5a5';
	ctx.fillText(truncate(ctx, options.opponent, 180), 58, 26);
	ctx.textAlign = 'right';
	ctx.fillStyle = '#93c5fd';
	ctx.fillText(truncate(ctx, options.username, 180), DESIGN.width - 58, 26);

	// coroas
	ctx.textAlign = 'left';
	drawCrown(ctx, 64, 48, enemy.c >= 3 ? '#facc15' : enemy.c >= 2 ? '#facc15' : '#94a3b8');
	ctx.fillStyle = '#f8fafc';
	ctx.font = 'bold 16px system-ui, sans-serif';
	ctx.fillText(String(enemy.c), 78, 54);

	ctx.textAlign = 'right';
	drawCrown(ctx, DESIGN.width - 64, 48, local.c >= 3 ? '#facc15' : local.c >= 2 ? '#facc15' : '#94a3b8');
	ctx.fillText(String(local.c), DESIGN.width - 78, 54);

	// tempo
	const remaining =
		snapshot.ph === 0
			? 180_000 - snapshot.m
			: snapshot.ph === 1
				? 300_000 - snapshot.m
				: 0;
	const overtime = snapshot.ph === 1;
	ctx.textAlign = 'center';
	ctx.font = 'bold 30px system-ui, sans-serif';
	ctx.fillStyle = overtime ? '#fb923c' : '#f8fafc';
	ctx.fillText(formatTime(remaining), DESIGN.width / 2, 44);
	ctx.font = 'bold 11px system-ui, sans-serif';
	ctx.fillStyle = overtime ? '#fdba74' : '#94a3b8';
	ctx.fillText(
		overtime ? literal.t('battle.overtime') : literal.t('battle.timeLeft').toUpperCase(),
		DESIGN.width / 2,
		61
	);

	// elixir
	const barX = 10;
	const barWidth = DESIGN.width - 20;
	const barY = HUD.elixirY;
	const segmentGap = 3;
	const segmentWidth = (barWidth - segmentGap * 9) / 10;
	for (let index = 0; index < 10; index += 1) {
		const filled = Math.min(1, Math.max(0, local.e - index));
		ctx.fillStyle = 'rgba(30, 27, 75, 0.85)';
		roundRect(ctx, barX + index * (segmentWidth + segmentGap), barY, segmentWidth, HUD.elixirHeight, 4);
		ctx.fill();
		if (filled > 0) {
			ctx.fillStyle = '#c084fc';
			roundRect(
				ctx,
				barX + index * (segmentWidth + segmentGap) + 1,
				barY + 2,
				(segmentWidth - 2) * filled,
				HUD.elixirHeight - 4,
				3
			);
			ctx.fill();
		}
		ctx.strokeStyle = 'rgba(15, 23, 42, 0.8)';
		ctx.lineWidth = 1;
		roundRect(ctx, barX + index * (segmentWidth + segmentGap), barY, segmentWidth, HUD.elixirHeight, 4);
		ctx.stroke();
	}
	ctx.textAlign = 'right';
	ctx.font = 'bold 12px system-ui, sans-serif';
	ctx.fillStyle = '#e9d5ff';
	ctx.fillText(local.e.toFixed(1), DESIGN.width - 46, barY - 4);

	// cartas
	const layout = handLayout();
	for (let slot = 0; slot < 4; slot += 1) {
		const entry = local.h[slot];
		if (!entry) {
			continue;
		}
		const card = loaded.definitions.cards[entry[0]];
		if (!card) {
			continue;
		}
		const cost = entry[1];
		const affordable = local.e + 0.01 >= cost;
		const selected = options.selectedSlot === slot;
		const x = layout.startX + slot * (layout.cardWidth + layout.gap);
		const y = layout.y - (selected ? 10 : 0);
		const height = layout.cardHeight;
		const rarity = RARITY_COLORS[card.rarity] ?? RARITY_COLORS.common;

		ctx.save();
		if (selected) {
			ctx.shadowColor = '#facc15';
			ctx.shadowBlur = 18;
		}
		ctx.fillStyle = 'rgba(15, 23, 42, 0.96)';
		roundRect(ctx, x, y, layout.cardWidth, height, 12);
		ctx.fill();
		const cardGradient = ctx.createLinearGradient(x, y, x, y + height);
		cardGradient.addColorStop(0, `${rarity[0]}cc`);
		cardGradient.addColorStop(0.45, 'rgba(15, 23, 42, 0.35)');
		cardGradient.addColorStop(1, 'rgba(15, 23, 42, 0.95)');
		ctx.fillStyle = cardGradient;
		roundRect(ctx, x + 2, y + 2, layout.cardWidth - 4, height - 4, 10);
		ctx.fill();
		ctx.restore();

		// ícone
		drawCardIcon(
			ctx,
			card,
			loaded.definitions,
			x + layout.cardWidth / 2 - 44,
			y + 16,
			88,
			0
		);

		// custo
		ctx.beginPath();
		ctx.arc(x + 17, y + 18, 13, 0, Math.PI * 2);
		ctx.fillStyle = affordable ? '#a855f7' : '#4c1d95';
		ctx.fill();
		ctx.strokeStyle = '#0f172a';
		ctx.lineWidth = 2;
		ctx.stroke();
		ctx.fillStyle = '#f8fafc';
		ctx.font = 'bold 16px system-ui, sans-serif';
		ctx.textAlign = 'center';
		ctx.textBaseline = 'middle';
		ctx.fillText(String(Math.round(cost)), x + 17, y + 19);
		ctx.textBaseline = 'alphabetic';

		// nome
		const text = literal.card(card.slug).name;
		ctx.font = 'bold 12px system-ui, sans-serif';
		ctx.textAlign = 'center';
		ctx.fillStyle = affordable ? '#f8fafc' : '#cbd5e1';
		let label = text;
		while (ctx.measureText(label).width > layout.cardWidth - 16 && label.length > 4) {
			label = `${label.slice(0, -2)}…`;
		}
		ctx.fillText(label, x + layout.cardWidth / 2, y + height - 16);

		if (!affordable) {
			ctx.fillStyle = 'rgba(2, 6, 23, 0.5)';
			roundRect(ctx, x, y, layout.cardWidth, height, 12);
			ctx.fill();
			ctx.fillStyle = 'rgba(148, 163, 184, 0.85)';
			ctx.font = 'bold 10px system-ui, sans-serif';
			ctx.fillText(
				literal.t('battle.notEnoughElixir'),
				x + layout.cardWidth / 2,
				y + height / 2
			);
		}
		ctx.strokeStyle = selected ? '#facc15' : `${rarity[1]}88`;
		ctx.lineWidth = selected ? 3 : 1.5;
		roundRect(ctx, x, y, layout.cardWidth, height, 12);
		ctx.stroke();
	}

	// próxima carta (mini, à esquerda como na referência)
	const nextEntry = local.n;
	if (nextEntry) {
		const next = loaded.definitions.cards[nextEntry[0]];
		if (next) {
			const x = 8;
			const y = HUD.handY + 6;
			ctx.save();
			ctx.globalAlpha = 0.92;
			ctx.fillStyle = 'rgba(15, 23, 42, 0.9)';
			roundRect(ctx, x, y, 58, 112, 10);
			ctx.fill();
			ctx.strokeStyle = '#475569';
			ctx.lineWidth = 1.5;
			ctx.stroke();
			drawCardIcon(ctx, next, loaded.definitions, x + 4, y + 6, 50, 0);
			ctx.fillStyle = '#a855f7';
			ctx.beginPath();
			ctx.arc(x + 11, y + 11, 9, 0, Math.PI * 2);
			ctx.fill();
			ctx.fillStyle = '#f8fafc';
			ctx.font = 'bold 11px system-ui, sans-serif';
			ctx.textAlign = 'center';
			ctx.fillText(String(Math.round(nextEntry[1])), x + 11, y + 15);
			ctx.font = 'bold 9px system-ui, sans-serif';
			ctx.fillStyle = '#94a3b8';
			ctx.fillText(literal.t('battle.next').toUpperCase(), x + 29, y + 126);
			ctx.restore();
		}
	}

	// faixa de elixir (abaixo das cartas)
	ctx.save();
	ctx.fillStyle = 'rgba(2, 6, 23, 0.55)';
	roundRect(ctx, 0, HUD.elixirY - 3, DESIGN.width, 22, 8);
	ctx.fill();
	ctx.restore();

	ctx.restore();
}

export function handCardAt(x: number, y: number): number | null {
	const layout = handLayout();
	if (y < layout.y - 10 || y > layout.y + layout.cardHeight) {
		return null;
	}
	for (let slot = 0; slot < 4; slot += 1) {
		const start = layout.startX + slot * (layout.cardWidth + layout.gap);
		if (x >= start && x <= start + layout.cardWidth) {
			return slot;
		}
	}
	return null;
}

export function drawCrownCount(
	ctx: CanvasRenderingContext2D,
	x: number,
	y: number,
	count: number
) {
	for (let index = 0; index < count; index += 1) {
		drawCrown(ctx, x + index * 18, y, '#facc15');
	}
}

export const ARENA_TOP = ARENA_LAYOUT.y;
