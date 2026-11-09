import type { BuildingDef, CardDef, SpellDef } from '#lib/types';
import { ensureBaked } from './sprites';

function roundRect(
	ctx: CanvasRenderingContext2D,
	x: number,
	y: number,
	w: number,
	h: number,
	r: number
) {
	ctx.beginPath();
	ctx.moveTo(x + r, y);
	ctx.arcTo(x + w, y, x + w, y + h, r);
	ctx.arcTo(x + w, y + h, x, y + h, r);
	ctx.arcTo(x, y + h, x, y, r);
	ctx.arcTo(x, y, x + w, y, r);
	ctx.closePath();
}

export function drawBuildingIcon(
	ctx: CanvasRenderingContext2D,
	building: BuildingDef,
	x: number,
	y: number,
	size: number,
	color = '#3b82f6'
) {
	const cx = x + size / 2;
	const base = y + size * 0.82;
	const accessory = building.visual.accessory ?? '';
	const variant = `${building.slug} ${accessory}`.toLowerCase();
	ctx.save();
	ctx.lineJoin = 'round';
	// base
	roundRect(ctx, cx - size * 0.3, base - size * 0.18, size * 0.6, size * 0.16, 4);
	ctx.fillStyle = '#475569';
	ctx.fill();
	ctx.strokeStyle = '#0f172a';
	ctx.lineWidth = 2;
	ctx.stroke();

	if (variant.includes('cannon') || variant.includes('tesla') || variant.includes('bomb')) {
		roundRect(ctx, cx - size * 0.2, base - size * 0.42, size * 0.4, size * 0.26, 5);
		ctx.fillStyle = '#64748b';
		ctx.fill();
		ctx.stroke();
		ctx.beginPath();
		ctx.moveTo(cx, base - size * 0.34);
		ctx.lineTo(cx + size * 0.34, base - size * 0.4);
		ctx.lineWidth = size * 0.11;
		ctx.strokeStyle = '#1e293b';
		ctx.stroke();
		ctx.beginPath();
		ctx.arc(cx - size * 0.06, base - size * 0.28, size * 0.08, 0, Math.PI * 2);
		ctx.fillStyle = color;
		ctx.fill();
	} else if (variant.includes('inferno') || variant.includes('mortar')) {
		ctx.beginPath();
		ctx.moveTo(cx - size * 0.18, base - size * 0.12);
		ctx.lineTo(cx - size * 0.1, base - size * 0.62);
		ctx.lineTo(cx + size * 0.12, base - size * 0.62);
		ctx.lineTo(cx + size * 0.18, base - size * 0.12);
		ctx.closePath();
		ctx.fillStyle = '#475569';
		ctx.fill();
		ctx.stroke();
		ctx.beginPath();
		ctx.arc(cx + size * 0.03, base - size * 0.44, size * 0.12, 0, Math.PI * 2);
		ctx.fillStyle = variant.includes('inferno') ? '#f97316' : '#1e293b';
		ctx.fill();
		ctx.stroke();
	} else if (variant.includes('collector')) {
		roundRect(ctx, cx - size * 0.18, base - size * 0.55, size * 0.36, size * 0.4, 6);
		ctx.fillStyle = '#7c3aed';
		ctx.fill();
		ctx.stroke();
		ctx.beginPath();
		ctx.arc(cx, base - size * 0.36, size * 0.08, 0, Math.PI * 2);
		ctx.fillStyle = '#e9d5ff';
		ctx.fill();
	} else {
		// casinha/spawner
		ctx.beginPath();
		ctx.moveTo(cx - size * 0.26, base - size * 0.2);
		ctx.lineTo(cx - size * 0.26, base - size * 0.5);
		ctx.lineTo(cx, base - size * 0.68);
		ctx.lineTo(cx + size * 0.26, base - size * 0.5);
		ctx.lineTo(cx + size * 0.26, base - size * 0.2);
		ctx.closePath();
		ctx.fillStyle = '#92400e';
		ctx.fill();
		ctx.stroke();
		roundRect(ctx, cx - size * 0.1, base - size * 0.36, size * 0.2, size * 0.16, 3);
		ctx.fillStyle = '#fef3c7';
		ctx.fill();
		ctx.stroke();
	}
	ctx.restore();
}

export function drawSpellIcon(
	ctx: CanvasRenderingContext2D,
	spell: SpellDef,
	x: number,
	y: number,
	size: number
) {
	const cx = x + size / 2;
	const cy = y + size / 2;
	const variant = `${spell.slug} ${spell.visual.variant}`.toLowerCase();
	ctx.save();
	ctx.lineJoin = 'round';
	if (variant.includes('fire') || variant.includes('rocket')) {
		ctx.beginPath();
		ctx.moveTo(cx, cy - size * 0.34);
		ctx.bezierCurveTo(cx + size * 0.34, cy - size * 0.08, cx + size * 0.2, cy + size * 0.3, cx, cy + size * 0.3);
		ctx.bezierCurveTo(cx - size * 0.2, cy + size * 0.3, cx - size * 0.34, cy - size * 0.08, cx, cy - size * 0.34);
		ctx.fillStyle = '#f97316';
		ctx.fill();
		ctx.strokeStyle = '#7c2d12';
		ctx.lineWidth = 2.5;
		ctx.stroke();
		ctx.beginPath();
		ctx.moveTo(cx, cy - size * 0.12);
		ctx.bezierCurveTo(cx + size * 0.14, cy + size * 0.02, cx + size * 0.1, cy + size * 0.2, cx, cy + size * 0.2);
		ctx.bezierCurveTo(cx - size * 0.1, cy + size * 0.2, cx - size * 0.14, cy + size * 0.02, cx, cy - size * 0.12);
		ctx.fillStyle = '#fde047';
		ctx.fill();
	} else if (variant.includes('arrow')) {
		for (let index = -1; index <= 1; index += 1) {
			ctx.save();
			ctx.translate(cx + index * size * 0.16, cy);
			ctx.rotate(-0.5);
			ctx.beginPath();
			ctx.moveTo(0, -size * 0.28);
			ctx.lineTo(0, size * 0.26);
			ctx.strokeStyle = '#92400e';
			ctx.lineWidth = 3;
			ctx.stroke();
			ctx.beginPath();
			ctx.moveTo(0, -size * 0.32);
			ctx.lineTo(-size * 0.05, -size * 0.2);
			ctx.lineTo(size * 0.05, -size * 0.2);
			ctx.closePath();
			ctx.fillStyle = '#e2e8f0';
			ctx.fill();
			ctx.restore();
		}
	} else if (variant.includes('zap') || variant.includes('lightning')) {
		ctx.beginPath();
		ctx.moveTo(cx + size * 0.08, cy - size * 0.36);
		ctx.lineTo(cx - size * 0.18, cy - size * 0.02);
		ctx.lineTo(cx - size * 0.02, cy - size * 0.02);
		ctx.lineTo(cx - size * 0.1, cy + size * 0.34);
		ctx.lineTo(cx + size * 0.22, cy - size * 0.08);
		ctx.lineTo(cx + size * 0.04, cy - size * 0.08);
		ctx.closePath();
		ctx.fillStyle = '#fde047';
		ctx.fill();
		ctx.strokeStyle = '#a16207';
		ctx.lineWidth = 2.5;
		ctx.stroke();
	} else if (variant.includes('poison')) {
		ctx.beginPath();
		ctx.arc(cx, cy + size * 0.08, size * 0.24, 0, Math.PI * 2);
		ctx.fillStyle = '#65a30d';
		ctx.fill();
		ctx.strokeStyle = '#365314';
		ctx.lineWidth = 2.5;
		ctx.stroke();
		for (let index = 0; index < 4; index += 1) {
			const angle = (index / 4) * Math.PI * 2;
			ctx.beginPath();
			ctx.arc(cx + Math.cos(angle) * size * 0.22, cy + Math.sin(angle) * size * 0.22 - size * 0.12, size * 0.07, 0, Math.PI * 2);
			ctx.fillStyle = '#a3e635';
			ctx.fill();
		}
	} else if (variant.includes('freeze')) {
		ctx.translate(cx, cy);
		for (let index = 0; index < 6; index += 1) {
			ctx.rotate(Math.PI / 3);
			ctx.beginPath();
			ctx.moveTo(0, 0);
			ctx.lineTo(0, -size * 0.3);
			ctx.strokeStyle = '#7dd3fc';
			ctx.lineWidth = 4;
			ctx.stroke();
			ctx.beginPath();
			ctx.moveTo(0, -size * 0.2);
			ctx.lineTo(size * 0.07, -size * 0.27);
			ctx.moveTo(0, -size * 0.2);
			ctx.lineTo(-size * 0.07, -size * 0.27);
			ctx.stroke();
		}
	} else if (variant.includes('tornado')) {
		for (let index = 0; index < 3; index += 1) {
			ctx.beginPath();
			ctx.ellipse(cx, cy + index * size * 0.1, size * 0.3 - index * size * 0.06, size * 0.1, 0, 0, Math.PI * 2);
			ctx.strokeStyle = '#a5f3fc';
			ctx.lineWidth = 3;
			ctx.stroke();
		}
	} else if (variant.includes('log')) {
		roundRect(ctx, cx - size * 0.32, cy - size * 0.14, size * 0.64, size * 0.28, 8);
		ctx.fillStyle = '#92400e';
		ctx.fill();
		ctx.strokeStyle = '#451a03';
		ctx.lineWidth = 2.5;
		ctx.stroke();
		ctx.beginPath();
		ctx.arc(cx + size * 0.24, cy, size * 0.14, -Math.PI / 2, Math.PI / 2);
		ctx.fillStyle = '#c2410c';
		ctx.fill();
		ctx.stroke();
	} else if (variant.includes('graveyard')) {
		roundRect(ctx, cx - size * 0.22, cy - size * 0.12, size * 0.44, size * 0.3, 4);
		ctx.fillStyle = '#64748b';
		ctx.fill();
		ctx.stroke();
		ctx.beginPath();
		ctx.arc(cx, cy - size * 0.1, size * 0.14, Math.PI, Math.PI * 2);
		ctx.fill();
		ctx.stroke();
	} else if (variant.includes('rage')) {
		ctx.beginPath();
		ctx.arc(cx, cy, size * 0.26, 0, Math.PI * 2);
		ctx.fillStyle = '#dc2626';
		ctx.fill();
		ctx.strokeStyle = '#7f1d1d';
		ctx.lineWidth = 2.5;
		ctx.stroke();
		for (let index = 0; index < 5; index += 1) {
			const angle = (index / 5) * Math.PI * 2;
			ctx.beginPath();
			ctx.moveTo(cx + Math.cos(angle) * size * 0.28, cy + Math.sin(angle) * size * 0.28);
			ctx.lineTo(cx + Math.cos(angle) * size * 0.38, cy + Math.sin(angle) * size * 0.38);
			ctx.strokeStyle = '#ef4444';
			ctx.lineWidth = 3;
			ctx.stroke();
		}
	} else if (variant.includes('earthquake')) {
		ctx.beginPath();
		ctx.moveTo(cx - size * 0.3, cy);
		ctx.lineTo(cx - size * 0.12, cy - size * 0.12);
		ctx.lineTo(cx - size * 0.02, cy + size * 0.06);
		ctx.lineTo(cx + size * 0.1, cy - size * 0.16);
		ctx.lineTo(cx + size * 0.3, cy + size * 0.06);
		ctx.strokeStyle = '#a16207';
		ctx.lineWidth = 5;
		ctx.stroke();
		ctx.beginPath();
		ctx.moveTo(cx - size * 0.18, cy + size * 0.24);
		ctx.lineTo(cx + size * 0.2, cy + size * 0.24);
		ctx.strokeStyle = '#78350f';
		ctx.lineWidth = 3;
		ctx.stroke();
	} else {
		ctx.beginPath();
		ctx.arc(cx, cy, size * 0.26, 0, Math.PI * 2);
		ctx.fillStyle = spell.visual.color || '#a855f7';
		ctx.fill();
		ctx.strokeStyle = '#0f172a';
		ctx.lineWidth = 2.5;
		ctx.stroke();
	}
	ctx.restore();
}

export function drawCardIcon(
	ctx: CanvasRenderingContext2D,
	card: CardDef,
	content: {
		troops: Record<string, import('#lib/types').TroopDef>;
		buildings: Record<string, BuildingDef>;
		spells: Record<string, SpellDef>;
	},
	x: number,
	y: number,
	size: number,
	side: 0 | 1 = 0
) {
	if (card.type.kind === 'troop') {
		const troop = content.troops[card.type.troop];
		if (!troop) {
			return;
		}
		const baked = ensureBaked(troop.visual, side, 'front', troop.slug);
		if (baked) {
			const frame = baked.clips.idle[0];
			if (frame) {
				ctx.drawImage(frame as CanvasImageSource, x, y - size * 0.04, size, size);
				return;
			}
		}
	} else if (card.type.kind === 'building') {
		const building = content.buildings[card.type.building];
		if (building) {
			drawBuildingIcon(ctx, building, x, y, size);
		}
	} else {
		const spell = content.spells[card.type.spell];
		if (spell) {
			drawSpellIcon(ctx, spell, x, y, size);
		}
	}
}

export { roundRect };
