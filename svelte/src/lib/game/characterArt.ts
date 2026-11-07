import type { Side, VisualDef } from '#lib/types';
import { TEAM_COLORS } from './constants';
import { buildHumanoidRig, clamp, type ClipName, type Point } from './skeleton';

export const SPRITE_SIZE = 128;
export const GROUND_Y = 112;
const CENTER_X = SPRITE_SIZE / 2;

export const CLIP_FRAMES: Record<ClipName, number> = {
	idle: 6,
	walk: 8,
	attack: 5
};

interface Palette {
	team: string;
	teamLight: string;
	teamDark: string;
	secondary: string;
	skin: string;
	outline: string;
	metal: string;
}

function paletteFor(visual: VisualDef, side: Side): Palette {
	const team = side === 0 ? TEAM_COLORS.blue : TEAM_COLORS.red;
	const skin =
		visual.archetype === 'goblin'
			? '#86efac'
			: visual.archetype === 'skeleton'
				? '#e2e8f0'
				: visual.archetype === 'dragon'
					? '#fb923c'
					: '#f1c27d';
	return {
		team: team.primary,
		teamLight: team.light,
		teamDark: team.dark,
		secondary: visual.secondary || '#e2e8f0',
		skin,
		outline: '#0f172a',
		metal: '#94a3b8'
	};
}

function p(x: number, y: number): Point {
	return { x: CENTER_X + x, y: GROUND_Y + y };
}

function strokeLine(
	ctx: CanvasRenderingContext2D,
	a: Point,
	b: Point,
	width: number,
	color: string,
	outline = '#0f172a'
) {
	ctx.beginPath();
	ctx.moveTo(a.x, a.y);
	ctx.lineTo(b.x, b.y);
	ctx.strokeStyle = outline;
	ctx.lineWidth = width + 3;
	ctx.lineCap = 'round';
	ctx.stroke();
	ctx.strokeStyle = color;
	ctx.lineWidth = width;
	ctx.stroke();
}

function fillCircle(
	ctx: CanvasRenderingContext2D,
	center: Point,
	radius: number,
	fill: string,
	outline = '#0f172a',
	line = 2
) {
	ctx.beginPath();
	ctx.arc(center.x, center.y, radius, 0, Math.PI * 2);
	ctx.fillStyle = fill;
	ctx.fill();
	if (line > 0) {
		ctx.strokeStyle = outline;
		ctx.lineWidth = line;
		ctx.stroke();
	}
}

function polygon(ctx: CanvasRenderingContext2D, points: Point[], fill: string, outline = '#0f172a', line = 2) {
	ctx.beginPath();
	points.forEach((point, index) => {
		if (index === 0) {
			ctx.moveTo(point.x, point.y);
		} else {
			ctx.lineTo(point.x, point.y);
		}
	});
	ctx.closePath();
	ctx.fillStyle = fill;
	ctx.fill();
	if (line > 0) {
		ctx.strokeStyle = outline;
		ctx.lineWidth = line;
		ctx.stroke();
	}
}

function drawWeapon(
	ctx: CanvasRenderingContext2D,
	visual: VisualDef,
	hand: Point,
	angle: number,
	palette: Palette
) {
	const length = visual.weapon === 'spear' ? 30 : 22;
	const tip = { x: hand.x + Math.cos(angle) * length, y: hand.y + Math.sin(angle) * length };
	switch (visual.weapon) {
		case 'sword':
			strokeLine(ctx, hand, tip, 5, '#e2e8f0');
			strokeLine(ctx, { x: hand.x - 3, y: hand.y + 6 }, { x: hand.x + 3, y: hand.y - 2 }, 4, '#f59e0b');
			break;
		case 'axe': {
			strokeLine(ctx, hand, tip, 4, '#92400e');
			const blade = [
				{ x: tip.x - 6, y: tip.y - 8 },
				{ x: tip.x + 10, y: tip.y - 2 },
				{ x: tip.x + 4, y: tip.y + 10 }
			];
			polygon(ctx, blade, '#cbd5e1');
			break;
		}
		case 'spear':
			strokeLine(ctx, hand, tip, 3, '#92400e');
			polygon(
				ctx,
				[
					{ x: tip.x, y: tip.y },
					{ x: tip.x - 5, y: tip.y + 9 },
					{ x: tip.x + 5, y: tip.y + 9 }
				],
				'#e2e8f0'
			);
			break;
		case 'bow':
			ctx.beginPath();
			ctx.arc(hand.x + 4, hand.y, 13, -1.1, 1.1);
			ctx.strokeStyle = '#92400e';
			ctx.lineWidth = 4;
			ctx.stroke();
			ctx.beginPath();
			ctx.moveTo(hand.x + 10, hand.y - 12);
			ctx.lineTo(hand.x + 10, hand.y + 12);
			ctx.strokeStyle = '#f8fafc';
			ctx.lineWidth = 1.5;
			ctx.stroke();
			break;
		case 'staff':
			strokeLine(ctx, hand, tip, 4, '#92400e');
			fillCircle(ctx, tip, 6, palette.teamLight);
			fillCircle(ctx, tip, 3, '#f8fafc', '#0f172a', 1);
			break;
		case 'hammer':
			strokeLine(ctx, hand, tip, 5, '#92400e');
			polygon(
				ctx,
				[
					{ x: tip.x - 8, y: tip.y - 6 },
					{ x: tip.x + 8, y: tip.y - 6 },
					{ x: tip.x + 6, y: tip.y + 8 },
					{ x: tip.x - 6, y: tip.y + 8 }
				],
				'#94a3b8'
			);
			break;
		case 'bomb':
			fillCircle(ctx, tip, 7, '#334155');
			ctx.beginPath();
			ctx.moveTo(tip.x + 4, tip.y - 6);
			ctx.lineTo(tip.x + 9, tip.y - 12);
			ctx.strokeStyle = '#f59e0b';
			ctx.lineWidth = 2;
			ctx.stroke();
			break;
		default:
			fillCircle(ctx, tip, 4.5, palette.skin);
			break;
	}
}

function drawHead(
	ctx: CanvasRenderingContext2D,
	visual: VisualDef,
	head: Point,
	angle: number,
	palette: Palette,
	radius: number
) {
	ctx.save();
	ctx.translate(head.x, head.y);
	ctx.rotate(angle);
	if (visual.archetype === 'goblin') {
		polygon(
			ctx,
			[
				{ x: -radius, y: 0 },
				{ x: -radius - 9, y: -5 },
				{ x: -radius - 5, y: 5 }
			],
			palette.skin
		);
		polygon(
			ctx,
			[
				{ x: radius, y: 0 },
				{ x: radius + 9, y: -5 },
				{ x: radius + 5, y: 5 }
			],
			palette.skin
		);
	}
	fillCircle(ctx, { x: 0, y: 0 }, radius, palette.skin);
	if (visual.archetype === 'skeleton') {
		fillCircle(ctx, { x: -radius * 0.4, y: -1 }, radius * 0.28, '#0f172a', '#0f172a', 0);
		fillCircle(ctx, { x: radius * 0.4, y: -1 }, radius * 0.28, '#0f172a', '#0f172a', 0);
		ctx.beginPath();
		ctx.moveTo(-radius * 0.3, radius * 0.45);
		ctx.lineTo(radius * 0.3, radius * 0.45);
		ctx.strokeStyle = '#0f172a';
		ctx.lineWidth = 1.5;
		ctx.stroke();
	} else {
		fillCircle(ctx, { x: -radius * 0.35, y: -1 }, 1.6, '#0f172a', '#0f172a', 0);
		fillCircle(ctx, { x: radius * 0.35, y: -1 }, 1.6, '#0f172a', '#0f172a', 0);
	}
	if (visual.accessory === 'helmet' || visual.accessory === 'armor') {
		ctx.beginPath();
		ctx.arc(0, -2, radius + 2, Math.PI, Math.PI * 2);
		ctx.fillStyle = palette.metal;
		ctx.fill();
		ctx.strokeStyle = palette.outline;
		ctx.lineWidth = 2;
		ctx.stroke();
	}
	if (visual.accessory === 'hat' || visual.accessory === 'hood') {
		polygon(
			ctx,
			[
				{ x: -radius - 4, y: -radius * 0.5 },
				{ x: radius + 4, y: -radius * 0.5 },
				{ x: 0, y: -radius - 12 }
			],
			visual.accessory === 'hood' ? palette.teamDark : '#7c3aed'
		);
	}
	if (visual.accessory === 'hair') {
		ctx.beginPath();
		ctx.arc(0, -1, radius + 1, Math.PI * 1.05, Math.PI * 1.95);
		ctx.strokeStyle = '#f59e0b';
		ctx.lineWidth = 4;
		ctx.stroke();
	}
	if (visual.accessory === 'crown') {
		polygon(
			ctx,
			[
				{ x: -7, y: -radius - 2 },
				{ x: -4, y: -radius - 9 },
				{ x: 0, y: -radius - 3 },
				{ x: 4, y: -radius - 9 },
				{ x: 7, y: -radius - 2 }
			],
			'#facc15'
		);
	}
	ctx.restore();
}

function drawHumanoid(
	ctx: CanvasRenderingContext2D,
	visual: VisualDef,
	side: Side,
	clip: ClipName,
	frame: number,
	frames: number
) {
	const palette = paletteFor(visual, side);
	const rig = buildHumanoidRig(visual, clip, frame, frames);
	const giant = visual.archetype === 'giant';
	const ghost = visual.archetype === 'spirit';
	if (ghost) {
		ctx.globalAlpha = 0.82;
		fillCircle(ctx, p(rig.chest.x - 4, rig.chest.y + 4), giant ? 30 : 20, 'rgba(147, 197, 253, 0.18)', 'rgba(147, 197, 253, 0.3)', 1);
	}
	if (visual.accessory === 'cape') {
		polygon(
			ctx,
			[
				p(rig.chest.x - 8, rig.chest.y),
				p(rig.chest.x + 6, rig.chest.y + 2),
				p(rig.hip.x + 10, rig.hip.y + 14),
				p(rig.hip.x - 12, rig.hip.y + 16)
			],
			palette.teamDark
		);
	}
	if (visual.accessory === 'wings' || visual.accessory === 'flames') {
		const flap = Math.sin((frame / Math.max(1, frames)) * Math.PI * 2) * 8;
		ctx.globalAlpha = 0.85;
		polygon(
			ctx,
			[
				p(rig.chest.x - 4, rig.chest.y + 2),
				p(rig.chest.x - 26, rig.chest.y - 12 + flap),
				p(rig.chest.x - 12, rig.hip.y + 1)
			],
			visual.accessory === 'flames' ? '#fb923c' : palette.teamLight
		);
		polygon(
			ctx,
			[
				p(rig.chest.x + 4, rig.chest.y + 2),
				p(rig.chest.x + 28, rig.chest.y - 10 - flap),
				p(rig.hip.x + 12, rig.hip.y + 1)
			],
			visual.accessory === 'flames' ? '#f97316' : palette.team
		);
		ctx.globalAlpha = 1;
	}
	if (visual.accessory === 'quiver') {
		polygon(
			ctx,
			[
				p(rig.chest.x - 14, rig.chest.y),
				p(rig.chest.x - 9, rig.chest.y + 14),
				p(rig.chest.x - 17, rig.chest.y + 14)
			],
			'#92400e'
		);
	}
	if (visual.accessory === 'shield') {
		fillCircle(ctx, p(rig.chest.x - 12, rig.chest.y + 10), giant ? 12 : 9, palette.team, '#0f172a', 2);
		fillCircle(ctx, p(rig.chest.x - 12, rig.chest.y + 10), giant ? 5 : 3.5, palette.secondary, '#0f172a', 1);
	}

	// Braço e perna de trás
	strokeLine(ctx, p(rig.shoulderBack.x, rig.shoulderBack.y), p(rig.elbowBack.x, rig.elbowBack.y), giant ? 9 : 6, palette.teamDark);
	strokeLine(ctx, p(rig.elbowBack.x, rig.elbowBack.y), p(rig.handBack.x, rig.handBack.y), giant ? 8 : 5, palette.skin);
	strokeLine(ctx, p(rig.hipBack.x, rig.hipBack.y), p(rig.kneeBack.x, rig.kneeBack.y), giant ? 11 : 7, '#334155');
	strokeLine(ctx, p(rig.kneeBack.x, rig.kneeBack.y), p(rig.footBack.x, rig.footBack.y), giant ? 10 : 6, '#334155');

	// Torso
	const torsoWidth = giant ? 26 : visual.archetype === 'goblin' ? 12 : 16;
	polygon(
		ctx,
		[
			p(rig.chest.x - torsoWidth * 0.5, rig.chest.y),
			p(rig.chest.x + torsoWidth * 0.5, rig.chest.y),
			p(rig.hip.x + torsoWidth * 0.42, rig.hip.y + 3),
			p(rig.hip.x - torsoWidth * 0.42, rig.hip.y + 3)
		],
		palette.team
	);
	polygon(
		ctx,
		[
			p(rig.chest.x - torsoWidth * 0.3, rig.chest.y + 3),
			p(rig.chest.x + torsoWidth * 0.3, rig.chest.y + 3),
			p(rig.chest.x + torsoWidth * 0.25, rig.chest.y + 8),
			p(rig.chest.x - torsoWidth * 0.25, rig.chest.y + 8)
		],
		palette.secondary,
		palette.outline,
		1
	);
	if (visual.accessory === 'belt' || giant) {
		strokeLine(
			ctx,
			p(rig.hip.x - torsoWidth * 0.4, rig.hip.y),
			p(rig.hip.x + torsoWidth * 0.4, rig.hip.y),
			4,
			'#78350f'
		);
	}
	if (visual.accessory === 'coil') {
		ctx.beginPath();
		ctx.arc(CENTER_X, GROUND_Y + rig.chest.y + 2, 14, 0, Math.PI * 2);
		ctx.strokeStyle = palette.teamLight;
		ctx.lineWidth = 3;
		ctx.stroke();
		ctx.beginPath();
		ctx.arc(CENTER_X, GROUND_Y + rig.chest.y + 2, 9, 0, Math.PI * 2);
		ctx.stroke();
	}

	// Perna da frente
	strokeLine(ctx, p(rig.hipFront.x, rig.hipFront.y), p(rig.kneeFront.x, rig.kneeFront.y), giant ? 11 : 7, '#475569');
	strokeLine(ctx, p(rig.kneeFront.x, rig.kneeFront.y), p(rig.footFront.x, rig.footFront.y), giant ? 10 : 6, '#475569');
	fillCircle(ctx, p(rig.footFront.x, rig.footFront.y), giant ? 6 : 4, '#1e293b', '#0f172a', 1);

	// Braço da frente
	strokeLine(ctx, p(rig.shoulderFront.x, rig.shoulderFront.y), p(rig.elbowFront.x, rig.elbowFront.y), giant ? 9 : 6, palette.team);
	strokeLine(ctx, p(rig.elbowFront.x, rig.elbowFront.y), p(rig.handFront.x, rig.handFront.y), giant ? 8 : 5, palette.skin);

	// Cabeça
	const headRadius = giant ? 12 : visual.archetype === 'goblin' ? 10 : 8.5;
	drawHead(ctx, visual, p(rig.head.x, rig.head.y), rig.headAngle, palette, headRadius);
	if (visual.accessory === 'mask') {
		strokeLine(ctx, p(rig.head.x - 8, rig.head.y), p(rig.head.x + 8, rig.head.y), 5, '#1e293b');
	}
	if (visual.accessory === 'beard') {
		polygon(
			ctx,
			[
				p(rig.head.x - 6, rig.head.y + 5),
				p(rig.head.x + 6, rig.head.y + 5),
				p(rig.head.x, rig.head.y + 16)
			],
			'#e2e8f0'
		);
	}

	drawWeapon(ctx, visual, p(rig.weaponHand.x, rig.weaponHand.y), rig.weaponAngle, palette);
	if (ghost) {
		ctx.globalAlpha = 1;
	}
}

function drawQuadruped(
	ctx: CanvasRenderingContext2D,
	visual: VisualDef,
	side: Side,
	clip: ClipName,
	frame: number,
	frames: number
) {
	const palette = paletteFor(visual, side);
	const t = frame / Math.max(1, frames);
	const phase = t * Math.PI * 2;
	const flying = visual.accessory === 'wings';
	const bob = flying ? Math.sin(phase) * 4 : 0;
	const bodyY = GROUND_Y - 34 - bob;
	ctx.save();
	if (flying) {
		const flap = Math.sin(phase) * 12;
		ctx.globalAlpha = 0.9;
		polygon(
			ctx,
			[
				{ x: CENTER_X - 6, y: bodyY - 2 },
				{ x: CENTER_X - 42, y: bodyY - 20 + flap },
				{ x: CENTER_X - 20, y: bodyY + 18 }
			],
			palette.teamDark
		);
		polygon(
			ctx,
			[
				{ x: CENTER_X + 6, y: bodyY - 2 },
				{ x: CENTER_X + 42, y: bodyY - 20 - flap },
				{ x: CENTER_X + 20, y: bodyY + 18 }
			],
			palette.team
		);
		ctx.globalAlpha = 1;
	}
	const legSwing = clip === 'walk' ? Math.sin(phase) * 5 : 0;
	if (!flying) {
		strokeLine(ctx, { x: CENTER_X - 18, y: bodyY + 10 }, { x: CENTER_X - 20 + legSwing, y: GROUND_Y - 2 }, 7, palette.teamDark);
		strokeLine(ctx, { x: CENTER_X + 18, y: bodyY + 10 }, { x: CENTER_X + 20 - legSwing, y: GROUND_Y - 2 }, 7, palette.team);
	}
	ctx.beginPath();
	ctx.ellipse(CENTER_X, bodyY, 30, 19, 0, 0, Math.PI * 2);
	ctx.fillStyle = palette.team;
	ctx.fill();
	ctx.strokeStyle = palette.outline;
	ctx.lineWidth = 2.5;
	ctx.stroke();
	ctx.beginPath();
	ctx.ellipse(CENTER_X - 4, bodyY + 4, 20, 11, 0, 0, Math.PI * 2);
	ctx.fillStyle = palette.teamLight;
	ctx.globalAlpha = 0.5;
	ctx.fill();
	ctx.globalAlpha = 1;
	// cabeça
	const head = { x: CENTER_X + 34, y: bodyY - 10 };
	fillCircle(ctx, head, 12, palette.team);
	fillCircle(ctx, { x: head.x + 7, y: head.y - 3 }, 2, '#0f172a', '#0f172a', 0);
	polygon(
		ctx,
		[
			{ x: head.x + 4, y: head.y + 6 },
			{ x: head.x + 2, y: head.y - 2 },
			{ x: head.x + 12, y: head.y + 1 }
		],
		'#f8fafc'
	);
	if (visual.accessory === 'saddle') {
		const riderY = bodyY - 24;
		fillCircle(ctx, { x: CENTER_X - 2, y: riderY - 14 }, 8, palette.skin);
		polygon(
			ctx,
			[
				{ x: CENTER_X - 10, y: riderY - 6 },
				{ x: CENTER_X + 8, y: riderY - 6 },
				{ x: CENTER_X + 6, y: riderY + 8 },
				{ x: CENTER_X - 8, y: riderY + 8 }
			],
			palette.teamDark
		);
		strokeLine(ctx, { x: CENTER_X + 6, y: riderY }, { x: CENTER_X + 18, y: riderY - 12 }, 5, '#92400e');
		polygon(
			ctx,
			[
				{ x: CENTER_X + 16, y: riderY - 14 },
				{ x: CENTER_X + 24, y: riderY - 18 },
				{ x: CENTER_X + 24, y: riderY - 6 }
			],
			'#94a3b8'
		);
	}
	if (visual.accessory === 'flames') {
		for (let index = 0; index < 4; index += 1) {
			const flameX = CENTER_X - 24 + index * 14;
			polygon(
				ctx,
				[
					{ x: flameX, y: bodyY - 14 },
					{ x: flameX + 4, y: bodyY - 28 },
					{ x: flameX + 8, y: bodyY - 14 }
				],
				index % 2 === 0 ? '#fb923c' : '#f97316',
				'#7c2d12',
				1
			);
		}
	}
	ctx.restore();
}

function drawDragon(
	ctx: CanvasRenderingContext2D,
	visual: VisualDef,
	side: Side,
	clip: ClipName,
	frame: number,
	frames: number
) {
	const palette = paletteFor(visual, side);
	const t = frame / Math.max(1, frames);
	const phase = t * Math.PI * 2;
	const bob = Math.sin(phase) * 4;
	const bodyY = GROUND_Y - 48 - bob;
	const flap = Math.sin(phase) * 14;
	// asas
	ctx.globalAlpha = 0.9;
	polygon(
		ctx,
		[
			{ x: CENTER_X - 4, y: bodyY },
			{ x: CENTER_X - 46, y: bodyY - 26 + flap },
			{ x: CENTER_X - 54, y: bodyY + 14 + flap * 0.4 },
			{ x: CENTER_X - 16, y: bodyY + 16 }
		],
		palette.teamDark
	);
	polygon(
		ctx,
		[
			{ x: CENTER_X + 4, y: bodyY },
			{ x: CENTER_X + 46, y: bodyY - 26 - flap },
			{ x: CENTER_X + 54, y: bodyY + 14 - flap * 0.4 },
			{ x: CENTER_X + 16, y: bodyY + 16 }
		],
		palette.team
	);
	ctx.globalAlpha = 1;
	// cauda
	ctx.beginPath();
	ctx.moveTo(CENTER_X - 28, bodyY + 4);
	ctx.quadraticCurveTo(CENTER_X - 50, bodyY + 24, CENTER_X - 58, bodyY + 2);
	ctx.strokeStyle = palette.team;
	ctx.lineWidth = 8;
	ctx.lineCap = 'round';
	ctx.stroke();
	// corpo
	ctx.beginPath();
	ctx.ellipse(CENTER_X - 2, bodyY, 27, 17, 0, 0, Math.PI * 2);
	ctx.fillStyle = palette.team;
	ctx.fill();
	ctx.strokeStyle = palette.outline;
	ctx.lineWidth = 2.5;
	ctx.stroke();
	// barriga
	ctx.beginPath();
	ctx.ellipse(CENTER_X - 2, bodyY + 5, 18, 9, 0, 0, Math.PI * 2);
	ctx.fillStyle = '#fde68a';
	ctx.globalAlpha = 0.65;
	ctx.fill();
	ctx.globalAlpha = 1;
	// pescoço e cabeça
	strokeLine(ctx, { x: CENTER_X + 18, y: bodyY - 4 }, { x: CENTER_X + 38, y: bodyY - 26 }, 10, palette.team);
	fillCircle(ctx, { x: CENTER_X + 42, y: bodyY - 30 }, 11, palette.team);
	polygon(
		ctx,
		[
			{ x: CENTER_X + 46, y: bodyY - 34 },
			{ x: CENTER_X + 60, y: bodyY - 30 },
			{ x: CENTER_X + 46, y: bodyY - 26 }
		],
		palette.secondary
	);
	fillCircle(ctx, { x: CENTER_X + 46, y: bodyY - 33 }, 2, '#0f172a', '#0f172a', 0);
	// pernas com IK
	const legTargets = [
		{ x: CENTER_X - 16, y: GROUND_Y - 2 },
		{ x: CENTER_X + 14, y: GROUND_Y - 2 }
	];
	for (const target of legTargets) {
		const reach = { x: clamp(target.x, CENTER_X - 26, CENTER_X + 26), y: target.y };
		strokeLine(ctx, { x: CENTER_X - 4, y: bodyY + 8 }, reach, 7, palette.teamDark);
		fillCircle(ctx, reach, 4, '#0f172a', '#0f172a', 1);
	}
	if (visual.accessory === 'beam') {
		ctx.beginPath();
		ctx.moveTo(CENTER_X + 56, bodyY - 30);
		ctx.lineTo(CENTER_X + 76, bodyY - 24);
		ctx.strokeStyle = '#f87171';
		ctx.lineWidth = 3;
		ctx.stroke();
	}
}

function drawFlyer(
	ctx: CanvasRenderingContext2D,
	visual: VisualDef,
	side: Side,
	clip: ClipName,
	frame: number,
	frames: number
) {
	const palette = paletteFor(visual, side);
	const t = frame / Math.max(1, frames);
	const phase = t * Math.PI * 2;
	const bob = Math.sin(phase) * 5;
	if (visual.weapon === 'bomb' || visual.accessory === 'balloon') {
		const balloonY = GROUND_Y - 92 + bob;
		ctx.beginPath();
		ctx.ellipse(CENTER_X, balloonY, 25, 29, 0, 0, Math.PI * 2);
		ctx.fillStyle = palette.team;
		ctx.fill();
		ctx.strokeStyle = palette.outline;
		ctx.lineWidth = 2.5;
		ctx.stroke();
		strokeLine(ctx, { x: CENTER_X - 12, y: balloonY + 24 }, { x: CENTER_X - 10, y: GROUND_Y - 26 }, 1.5, '#475569');
		strokeLine(ctx, { x: CENTER_X + 12, y: balloonY + 24 }, { x: CENTER_X + 10, y: GROUND_Y - 26 }, 1.5, '#475569');
		polygon(
			ctx,
			[
				{ x: CENTER_X - 16, y: GROUND_Y - 26 },
				{ x: CENTER_X + 16, y: GROUND_Y - 26 },
				{ x: CENTER_X + 13, y: GROUND_Y - 4 },
				{ x: CENTER_X - 13, y: GROUND_Y - 4 }
			],
			'#92400e'
		);
		fillCircle(ctx, { x: CENTER_X, y: GROUND_Y - 16 }, 6, '#334155');
		return;
	}
	const bodyY = GROUND_Y - 56 + bob;
	const flap = Math.sin(phase) * 16;
	ctx.globalAlpha = 0.92;
	polygon(
		ctx,
		[
			{ x: CENTER_X - 4, y: bodyY },
			{ x: CENTER_X - 44, y: bodyY - 20 + flap },
			{ x: CENTER_X - 18, y: bodyY + 18 }
		],
		visual.accessory === 'flames' ? '#f97316' : palette.teamDark
	);
	polygon(
		ctx,
		[
			{ x: CENTER_X + 4, y: bodyY },
			{ x: CENTER_X + 44, y: bodyY - 20 - flap },
			{ x: CENTER_X + 18, y: bodyY + 18 }
		],
		visual.accessory === 'flames' ? '#fb923c' : palette.team
	);
	ctx.globalAlpha = 1;
	ctx.beginPath();
	ctx.ellipse(CENTER_X, bodyY, 18, 14, 0, 0, Math.PI * 2);
	ctx.fillStyle = palette.team;
	ctx.fill();
	ctx.strokeStyle = palette.outline;
	ctx.lineWidth = 2.5;
	ctx.stroke();
	fillCircle(ctx, { x: CENTER_X + 12, y: bodyY - 8 }, 8, palette.team);
	fillCircle(ctx, { x: CENTER_X + 15, y: bodyY - 10 }, 1.8, '#0f172a', '#0f172a', 0);
	polygon(
		ctx,
		[
			{ x: CENTER_X + 18, y: bodyY - 8 },
			{ x: CENTER_X + 27, y: bodyY - 5 },
			{ x: CENTER_X + 18, y: bodyY - 3 }
		],
		'#f59e0b'
	);
	if (visual.accessory === 'flames') {
		for (let index = 0; index < 3; index += 1) {
			polygon(
				ctx,
				[
					{ x: CENTER_X - 20 + index * 16, y: bodyY - 16 },
					{ x: CENTER_X - 15 + index * 16, y: bodyY - 32 },
					{ x: CENTER_X - 10 + index * 16, y: bodyY - 16 }
				],
				index % 2 === 0 ? '#fbbf24' : '#f97316',
				'#7c2d12',
				1
			);
		}
	}
}

function drawVehicle(
	ctx: CanvasRenderingContext2D,
	visual: VisualDef,
	side: Side,
	clip: ClipName,
	frame: number,
	frames: number
) {
	const palette = paletteFor(visual, side);
	const t = frame / Math.max(1, frames);
	const phase = t * Math.PI * 2;
	const bodyY = GROUND_Y - 30;
	const wheelSpin = clip === 'walk' ? phase : 0;
	// rodas
	for (const x of [CENTER_X - 18, CENTER_X + 18]) {
		fillCircle(ctx, { x, y: GROUND_Y - 9 }, 9, '#334155');
		fillCircle(ctx, { x, y: GROUND_Y - 9 }, 4, '#94a3b8');
		ctx.save();
		ctx.translate(x, GROUND_Y - 9);
		ctx.rotate(wheelSpin);
		strokeLine(ctx, { x: -6, y: 0 }, { x: 6, y: 0 }, 2, '#cbd5e1', '#334155');
		strokeLine(ctx, { x: 0, y: -6 }, { x: 0, y: 6 }, 2, '#cbd5e1', '#334155');
		ctx.restore();
	}
	polygon(
		ctx,
		[
			{ x: CENTER_X - 28, y: bodyY - 14 },
			{ x: CENTER_X + 28, y: bodyY - 14 },
			{ x: CENTER_X + 22, y: bodyY + 14 },
			{ x: CENTER_X - 22, y: bodyY + 14 }
		],
		palette.team
	);
	polygon(
		ctx,
		[
			{ x: CENTER_X - 18, y: bodyY - 8 },
			{ x: CENTER_X + 18, y: bodyY - 8 },
			{ x: CENTER_X + 14, y: bodyY + 2 },
			{ x: CENTER_X - 14, y: bodyY + 2 }
		],
		palette.secondary,
		palette.outline,
		1
	);
	if (visual.accessory === 'ram') {
		polygon(
			ctx,
			[
				{ x: CENTER_X + 28, y: bodyY - 6 },
				{ x: CENTER_X + 46, y: bodyY },
				{ x: CENTER_X + 28, y: bodyY + 8 }
			],
			'#92400e'
		);
	}
	if (visual.accessory === 'antenna' || visual.accessory === 'coil') {
		strokeLine(ctx, { x: CENTER_X, y: bodyY - 14 }, { x: CENTER_X, y: bodyY - 32 }, 3, '#94a3b8');
		fillCircle(ctx, { x: CENTER_X, y: bodyY - 34 }, 4, palette.teamLight);
	}
	// canhão
	polygon(
		ctx,
		[
			{ x: CENTER_X + 6, y: bodyY - 6 },
			{ x: CENTER_X + 34, y: bodyY - 10 },
			{ x: CENTER_X + 34, y: bodyY - 2 },
			{ x: CENTER_X + 6, y: bodyY + 2 }
		],
		'#475569'
	);
	if (visual.accessory === 'wheels' && visual.archetype === 'machine' && visual.weapon === 'sword') {
		// Mini P.E.K.K.A: usa o rig humanoide em vez do veículo.
	}
}

export function drawCharacter(
	ctx: CanvasRenderingContext2D,
	visual: VisualDef,
	side: Side,
	clip: ClipName,
	frame: number,
	frames: number
) {
	ctx.save();
	ctx.lineJoin = 'round';
	switch (visual.archetype) {
		case 'beast':
			drawQuadruped(ctx, visual, side, clip, frame, frames);
			break;
		case 'dragon':
			drawDragon(ctx, visual, side, clip, frame, frames);
			break;
		case 'flyer':
			drawFlyer(ctx, visual, side, clip, frame, frames);
			break;
		case 'machine':
			if (visual.weapon === 'sword') {
				drawHumanoid(ctx, visual, side, clip, frame, frames);
			} else {
				drawVehicle(ctx, visual, side, clip, frame, frames);
			}
			break;
		case 'spirit':
		case 'skeleton':
		case 'goblin':
		case 'giant':
		case 'humanoid':
		default:
			drawHumanoid(ctx, visual, side, clip, frame, frames);
			break;
	}
	ctx.restore();
}
