import { effectKind, spellByIndex } from '#lib/stores/content';
import type { Side } from '#lib/types';
import { TEAM_COLORS } from './constants';

export function drawProjectile(
	ctx: CanvasRenderingContext2D,
	visual: string,
	x: number,
	y: number,
	facing: number,
	elapsedMs: number
) {
	ctx.save();
	ctx.translate(x, y);
	if (facing < 0) {
		ctx.scale(-1, 1);
	}
	const kind = visual.toLowerCase();
	if (kind.includes('arrow') || kind.includes('dart')) {
		ctx.rotate(-0.08);
		ctx.beginPath();
		ctx.moveTo(-9, 0);
		ctx.lineTo(7, 0);
		ctx.strokeStyle = '#92400e';
		ctx.lineWidth = 2.5;
		ctx.stroke();
		ctx.beginPath();
		ctx.moveTo(8, 0);
		ctx.lineTo(3, -3);
		ctx.lineTo(3, 3);
		ctx.closePath();
		ctx.fillStyle = '#e2e8f0';
		ctx.fill();
	} else if (kind.includes('cannon')) {
		ctx.beginPath();
		ctx.arc(0, 0, 4, 0, Math.PI * 2);
		ctx.fillStyle = '#1e293b';
		ctx.fill();
		ctx.strokeStyle = '#94a3b8';
		ctx.lineWidth = 1;
		ctx.stroke();
	} else if (kind.includes('bomb') || kind.includes('mortar') || kind.includes('ball')) {
		ctx.beginPath();
		ctx.arc(0, 0, 5, 0, Math.PI * 2);
		ctx.fillStyle = '#334155';
		ctx.fill();
		ctx.strokeStyle = '#0f172a';
		ctx.lineWidth = 1.5;
		ctx.stroke();
		ctx.beginPath();
		ctx.arc(-1.5, -1.5, 1.4, 0, Math.PI * 2);
		ctx.fillStyle = 'rgba(255,255,255,0.5)';
		ctx.fill();
	} else if (kind.includes('zap') || kind.includes('bolt') || kind.includes('lightning')) {
		ctx.beginPath();
		ctx.moveTo(4, -7);
		ctx.lineTo(-3, 1);
		ctx.lineTo(1, 1);
		ctx.lineTo(-4, 8);
		ctx.lineTo(5, -1);
		ctx.lineTo(1, -1);
		ctx.closePath();
		ctx.fillStyle = '#fde047';
		ctx.fill();
		ctx.strokeStyle = '#a16207';
		ctx.lineWidth = 1.2;
		ctx.stroke();
	} else {
		ctx.beginPath();
		ctx.arc(0, 0, 3.5, 0, Math.PI * 2);
		ctx.fillStyle = '#f8fafc';
		ctx.fill();
	}
	ctx.restore();
}

export function drawZone(
	ctx: CanvasRenderingContext2D,
	spellIndex: number,
	x: number,
	y: number,
	radiusPx: number,
	elapsedMs: number
) {
	const spell = spellByIndex(spellIndex);
	const variant = `${spell?.slug ?? ''} ${spell?.visual.variant ?? ''}`.toLowerCase();
	const pulse = 0.65 + Math.sin(elapsedMs * 0.006) * 0.15;
	ctx.save();
	ctx.globalAlpha = 0.55;
	ctx.beginPath();
	ctx.ellipse(x, y, radiusPx, radiusPx * 0.55, 0, 0, Math.PI * 2);
	if (variant.includes('poison')) {
		const gradient = ctx.createRadialGradient(x, y, radiusPx * 0.1, x, y, radiusPx);
		gradient.addColorStop(0, `rgba(132, 204, 22, ${pulse})`);
		gradient.addColorStop(1, 'rgba(22, 101, 52, 0.05)');
		ctx.fillStyle = gradient;
		ctx.fill();
		for (let index = 0; index < 7; index += 1) {
			const angle = index * 1.7 + elapsedMs * 0.001;
			const distance = radiusPx * (0.2 + (index % 3) * 0.22);
			ctx.beginPath();
			ctx.arc(
				x + Math.cos(angle) * distance,
				y + Math.sin(angle) * distance * 0.55,
				3 + (index % 2),
				0,
				Math.PI * 2
			);
			ctx.fillStyle = 'rgba(190, 242, 100, 0.45)';
			ctx.fill();
		}
	} else if (variant.includes('tornado')) {
		ctx.globalAlpha = 0.7;
		for (let index = 0; index < 4; index += 1) {
			ctx.beginPath();
			ctx.ellipse(
				x,
				y - index * 6,
				radiusPx * (0.7 - index * 0.12),
				radiusPx * (0.22 - index * 0.03),
				elapsedMs * 0.002 + index,
				0,
				Math.PI * 2
			);
			ctx.strokeStyle = 'rgba(165, 243, 252, 0.8)';
			ctx.lineWidth = 2.5;
			ctx.stroke();
		}
	} else if (variant.includes('earthquake')) {
		ctx.globalAlpha = 0.75;
		for (let index = 0; index < 6; index += 1) {
			const startX = x + (Math.sin(index * 2.3) * radiusPx) / 1.6;
			const startY = y + (Math.cos(index * 1.7) * radiusPx) / 3;
			ctx.beginPath();
			ctx.moveTo(startX, startY);
			ctx.lineTo(startX + 14, startY - 6);
			ctx.lineTo(startX + 26, startY + 4);
			ctx.strokeStyle = 'rgba(120, 53, 15, 0.8)';
			ctx.lineWidth = 2;
			ctx.stroke();
		}
	} else if (variant.includes('graveyard')) {
		ctx.fillStyle = 'rgba(71, 85, 105, 0.3)';
		ctx.fill();
		for (let index = 0; index < 5; index += 1) {
			const angle = index * 1.3;
			const distance = radiusPx * 0.5;
			const gx = x + Math.cos(angle) * distance;
			const gy = y + Math.sin(angle) * distance * 0.5;
			ctx.fillStyle = 'rgba(100, 116, 139, 0.75)';
			ctx.fillRect(gx - 3, gy - 8, 6, 10);
			ctx.beginPath();
			ctx.arc(gx, gy - 8, 3, Math.PI, Math.PI * 2);
			ctx.fill();
		}
	} else if (variant.includes('rage')) {
		const gradient = ctx.createRadialGradient(x, y, 0, x, y, radiusPx);
		gradient.addColorStop(0, `rgba(239, 68, 68, ${pulse})`);
		gradient.addColorStop(1, 'rgba(127, 29, 29, 0.05)');
		ctx.fillStyle = gradient;
		ctx.fill();
	} else {
		ctx.fillStyle = `rgba(168, 85, 247, ${pulse * 0.5})`;
		ctx.fill();
	}
	ctx.restore();
}

export function drawUnitEffects(
	ctx: CanvasRenderingContext2D,
	x: number,
	y: number,
	radiusPx: number,
	effects: number[],
	elapsedMs: number
) {
	const count = Math.floor(effects[0] ?? 0);
	for (let index = 0; index < count; index += 1) {
		const base = 1 + index * 3;
		const effectIndex = effects[base] ?? 0;
		const kind = effectKind(effectIndex);
		if (kind === 'freeze' || kind === 'slow') {
			ctx.save();
			ctx.globalAlpha = 0.35 + Math.sin(elapsedMs * 0.004) * 0.08;
			ctx.beginPath();
			ctx.arc(x, y - radiusPx * 0.9, radiusPx * 1.05, 0, Math.PI * 2);
			ctx.fillStyle = '#7dd3fc';
			ctx.fill();
			ctx.strokeStyle = '#e0f2fe';
			ctx.lineWidth = 2;
			ctx.stroke();
			ctx.restore();
		} else if (kind === 'poison') {
			for (let bubble = 0; bubble < 3; bubble += 1) {
				const angle = elapsedMs * 0.002 + bubble * 2.1;
				ctx.beginPath();
				ctx.arc(
					x + Math.cos(angle) * radiusPx * 0.7,
					y - radiusPx * 1.1 + Math.sin(angle) * 5,
					2.5,
					0,
					Math.PI * 2
				);
				ctx.fillStyle = 'rgba(163, 230, 53, 0.75)';
				ctx.fill();
			}
		} else if (kind === 'burn') {
			for (let flame = 0; flame < 3; flame += 1) {
				const fx = x + (flame - 1) * radiusPx * 0.5;
				const fy = y - radiusPx * (1.2 + Math.sin(elapsedMs * 0.01 + flame) * 0.12);
				ctx.beginPath();
				ctx.moveTo(fx - 3, fy + 6);
				ctx.quadraticCurveTo(fx, fy - 6, fx + 3, fy + 6);
				ctx.fillStyle = 'rgba(249, 115, 22, 0.8)';
				ctx.fill();
			}
		} else if (kind === 'stun') {
			const angle = elapsedMs * 0.005;
			for (let star = 0; star < 3; star += 1) {
				const a = angle + (star / 3) * Math.PI * 2;
				ctx.save();
				ctx.translate(x + Math.cos(a) * radiusPx * 0.8, y - radiusPx * 1.7 + Math.sin(a) * 5);
				ctx.rotate(a);
				ctx.fillStyle = '#fde047';
				ctx.beginPath();
				for (let point = 0; point < 5; point += 1) {
					const r = point % 2 === 0 ? 5 : 2;
					const pAngle = (point / 5) * Math.PI * 2 - Math.PI / 2;
					if (point === 0) {
						ctx.moveTo(Math.cos(pAngle) * r, Math.sin(pAngle) * r);
					} else {
						ctx.lineTo(Math.cos(pAngle) * r, Math.sin(pAngle) * r);
					}
				}
				ctx.closePath();
				ctx.fill();
				ctx.restore();
			}
		} else if (kind === 'rage') {
			ctx.beginPath();
			ctx.arc(x, y - radiusPx, radiusPx * 1.1, 0, Math.PI * 2);
			ctx.strokeStyle = 'rgba(239, 68, 68, 0.7)';
			ctx.lineWidth = 3;
			ctx.stroke();
		}
	}
}

export function drawShield(
	ctx: CanvasRenderingContext2D,
	x: number,
	y: number,
	radiusPx: number,
	elapsedMs: number
) {
	ctx.save();
	ctx.globalAlpha = 0.55 + Math.sin(elapsedMs * 0.005) * 0.1;
	ctx.beginPath();
	ctx.arc(x, y - radiusPx * 1.1, radiusPx * 1.25, 0, Math.PI * 2);
	ctx.strokeStyle = '#e2e8f0';
	ctx.lineWidth = 2.5;
	ctx.stroke();
	ctx.restore();
}

export function drawFloatingText(
	ctx: CanvasRenderingContext2D,
	text: string,
	x: number,
	y: number,
	color: string,
	alpha: number
) {
	ctx.save();
	ctx.globalAlpha = alpha;
	ctx.font = 'bold 16px system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.lineWidth = 3;
	ctx.strokeStyle = 'rgba(15, 23, 42, 0.85)';
	ctx.strokeText(text, x, y);
	ctx.fillStyle = color;
	ctx.fillText(text, x, y);
	ctx.restore();
}

export function teamGlow(side: Side): string {
	return side === 0 ? TEAM_COLORS.blue.glow : TEAM_COLORS.red.glow;
}
