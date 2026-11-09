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
	const gradient = ctx.createLinearGradient(0, 0, 0, 84);
	gradient.addColorStop(0, 'rgba(6, 12, 24, 0.98)');
	gradient.addColorStop(1, 'rgba(8, 16, 32, 0.2)');
	ctx.fillStyle = gradient;
	ctx.fillRect(0, 0, DESIGN.width, 84);
	ctx.fillStyle = 'rgba(247, 212, 92, 0.65)';
	ctx.fillRect(0, 82, DESIGN.width, 1.5);

	ctx.font = 'bold 17px Georgia, serif';
	ctx.textAlign = 'left';
	ctx.fillStyle = '#ffb3b3';
	ctx.fillText(truncate(ctx, options.opponent, 180), 58, 26);
	ctx.textAlign = 'right';
	ctx.fillStyle = '#9dc4ff';
	ctx.fillText(truncate(ctx, options.username, 180), DESIGN.width - 58, 26);

	// coroas
	ctx.textAlign = 'left';
	drawCrown(ctx, 64, 48, enemy.c >= 2 ? '#f7d45c' : '#cbd5e1');
	ctx.fillStyle = '#f8fafc';
	ctx.font = 'bold 16px system-ui, sans-serif';
	ctx.fillText(String(enemy.c), 78, 54);

	ctx.textAlign = 'right';
	drawCrown(ctx, DESIGN.width - 64, 48, local.c >= 2 ? '#f7d45c' : '#cbd5e1');
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
	ctx.font = 'bold 31px Georgia, serif';
	ctx.fillStyle = overtime ? '#ff9d4d' : '#ffe28a';
	ctx.shadowColor = 'rgba(247, 212, 92, 0.35)';
	ctx.shadowBlur = 10;
	ctx.fillText(formatTime(remaining), DESIGN.width / 2, 44);
	ctx.shadowBlur = 0;
	ctx.font = 'bold 11px Georgia, serif';
	ctx.fillStyle = overtime ? '#fdba74' : '#d9b85c';
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
	// moldura dourada da barra de elixir
	ctx.save();
	ctx.strokeStyle = 'rgba(247, 212, 92, 0.55)';
	ctx.lineWidth = 1.2;
	roundRect(ctx, barX - 2, barY - 2, barWidth + 4, HUD.elixirHeight + 4, 6);
	ctx.stroke();
	ctx.restore();

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
		const costFill = ctx.createLinearGradient(x + 5, y + 5, x + 29, y + 31);
		costFill.addColorStop(0, '#fff2a8');
		costFill.addColorStop(0.55, '#e9b521');
		costFill.addColorStop(1, '#8a5206');
		ctx.fillStyle = affordable ? costFill : '#4a3410';
		ctx.fill();
		ctx.strokeStyle = '#3a2102';
		ctx.lineWidth = 2;
		ctx.stroke();
		ctx.fillStyle = '#2a1a02';
		ctx.font = 'bold 16px Georgia, serif';
		ctx.textAlign = 'center';
		ctx.textBaseline = 'middle';
		ctx.fillText(String(Math.round(cost)), x + 17, y + 19);
		ctx.textBaseline = 'alphabetic';

		// nome
		const text = literal.card(card.slug).name;
		ctx.font = 'bold 12px Georgia, serif';
		ctx.textAlign = 'center';
		ctx.fillStyle = affordable ? '#fff3c4' : '#b9c1cf';
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
		ctx.strokeStyle = selected ? '#ffe789' : affordable ? 'rgba(217, 165, 33, 0.72)' : '#475569';
		ctx.lineWidth = selected ? 3 : 1.4;
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
			// mini carta com a mesma proporção das cartas da mão (148x160)
			const width = 58;
			const height = Math.round(width * (HUD.handHeight / HUD.cardWidth));
			const iconSize = width - 8;
			ctx.save();
			ctx.globalAlpha = 0.92;
			ctx.fillStyle = 'rgba(15, 23, 42, 0.9)';
			roundRect(ctx, x, y, width, height, 10);
			ctx.fill();
			ctx.strokeStyle = 'rgba(217, 165, 33, 0.7)';
			ctx.lineWidth = 1.5;
			ctx.stroke();
			drawCardIcon(ctx, next, loaded.definitions, x + 4, y + 4, iconSize, 0);
			ctx.fillStyle = '#e9b521';
			ctx.beginPath();
			ctx.arc(x + 11, y + 11, 9, 0, Math.PI * 2);
			ctx.fill();
			ctx.fillStyle = '#2a1a02';
			ctx.font = 'bold 11px Georgia, serif';
			ctx.textAlign = 'center';
			ctx.fillText(String(Math.round(nextEntry[1])), x + 11, y + 15);
			ctx.font = 'bold 9px system-ui, sans-serif';
			ctx.fillStyle = '#94a3b8';
			ctx.fillText(literal.t('battle.next').toUpperCase(), x + width / 2, y + height + 14);
			// elixir disponível (abaixo de "próxima")
			ctx.font = 'bold 13px system-ui, sans-serif';
			ctx.fillStyle = '#e9d5ff';
			ctx.fillText(local.e.toFixed(1), x + width / 2, y + height + 31);
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
