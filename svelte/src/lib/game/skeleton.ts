import type { VisualDef } from '#lib/types';

export type ClipName = 'idle' | 'walk' | 'attack';

export interface Point {
	x: number;
	y: number;
}

export interface HumanoidRig {
	hip: Point;
	chest: Point;
	head: Point;
	shoulderBack: Point;
	elbowBack: Point;
	handBack: Point;
	shoulderFront: Point;
	elbowFront: Point;
	handFront: Point;
	hipBack: Point;
	kneeBack: Point;
	footBack: Point;
	hipFront: Point;
	kneeFront: Point;
	footFront: Point;
	weaponAngle: number;
	weaponHand: Point;
	headAngle: number;
}

export interface Metrics {
	upperLeg: number;
	lowerLeg: number;
	torso: number;
	headRadius: number;
	upperArm: number;
	lowerArm: number;
	shoulder: number;
}

export function metricsFor(archetype: string): Metrics {
	switch (archetype) {
		case 'goblin':
			return { upperLeg: 11, lowerLeg: 11, torso: 15, headRadius: 10, upperArm: 9, lowerArm: 9, shoulder: 7 };
		case 'skeleton':
			return { upperLeg: 15, lowerLeg: 15, torso: 17, headRadius: 8, upperArm: 10, lowerArm: 10, shoulder: 8 };
		case 'giant':
			return { upperLeg: 23, lowerLeg: 23, torso: 30, headRadius: 11, upperArm: 16, lowerArm: 16, shoulder: 14 };
		default:
			return { upperLeg: 16, lowerLeg: 16, torso: 20, headRadius: 8.5, upperArm: 11, lowerArm: 11, shoulder: 9 };
	}
}

export function clamp(value: number, min: number, max: number): number {
	return Math.max(min, Math.min(max, value));
}

export function twoBoneIK(
	root: Point,
	target: Point,
	upper: number,
	lower: number,
	bend: 1 | -1
): { joint: Point; end: Point } {
	const dx = target.x - root.x;
	const dy = target.y - root.y;
	const raw = Math.hypot(dx, dy) || 0.0001;
	const distance = clamp(raw, Math.abs(upper - lower) + 0.05, upper + lower - 0.05);
	const baseAngle = Math.atan2(dy, dx);
	const cos = clamp((upper * upper + distance * distance - lower * lower) / (2 * upper * distance), -1, 1);
	const angle = Math.acos(cos);
	const jointAngle = baseAngle + bend * angle;
	const joint = {
		x: root.x + Math.cos(jointAngle) * upper,
		y: root.y + Math.sin(jointAngle) * upper
	};
	const end = {
		x: root.x + Math.cos(baseAngle) * distance,
		y: root.y + Math.sin(baseAngle) * distance
	};
	return { joint, end };
}

function lerp(a: number, b: number, t: number): number {
	return a + (b - a) * t;
}

export function buildHumanoidRig(
	visual: VisualDef,
	clip: ClipName,
	frame: number,
	frames: number
): HumanoidRig {
	const metrics = metricsFor(visual.archetype);
	const t = frames <= 1 ? 0 : frame / frames;
	const phase = t * Math.PI * 2;

	let bob = Math.sin(phase) * 0.8;
	if (clip === 'walk') {
		bob = Math.abs(Math.sin(phase)) * 2.0;
	} else if (clip === 'attack') {
		bob = -Math.sin(t * Math.PI) * 1.6;
	}

	const hip: Point = {
		x: clip === 'walk' ? Math.sin(phase) * 1.2 : 0,
		y: -(metrics.upperLeg + metrics.lowerLeg) + bob
	};

	const chest: Point = {
		x: hip.x + (clip === 'attack' ? -2 + t * 4 : 0),
		y: hip.y - metrics.torso
	};
	const lean = clip === 'attack' ? lerp(-0.15, 0.22, Math.sin(t * Math.PI)) : 0;
	const head: Point = {
		x: chest.x + Math.sin(lean) * metrics.headRadius,
		y: chest.y - metrics.headRadius - 3
	};

	const footBackTarget: Point = { x: -metrics.shoulder * 0.55, y: 0 };
	const footFrontTarget: Point = { x: metrics.shoulder * 0.55, y: 0 };
	if (clip === 'walk') {
		footBackTarget.x += Math.sin(phase) * 9;
		footBackTarget.y = -Math.max(0, Math.sin(phase)) * 6;
		footFrontTarget.x += Math.sin(phase + Math.PI) * 9;
		footFrontTarget.y = -Math.max(0, Math.sin(phase + Math.PI)) * 6;
	} else if (clip === 'attack') {
		footBackTarget.x -= 4;
		footFrontTarget.x += 3;
	}

	const legBack = twoBoneIK(
		hip,
		footBackTarget,
		metrics.upperLeg,
		metrics.lowerLeg,
		1
	);
	const legFront = twoBoneIK(
		hip,
		footFrontTarget,
		metrics.upperLeg,
		metrics.lowerLeg,
		-1
	);

	const shoulderBack: Point = { x: chest.x - metrics.shoulder * 0.35, y: chest.y + 2 };
	const shoulderFront: Point = { x: chest.x + metrics.shoulder * 0.55, y: chest.y + 2 };

	const idleSwing = Math.sin(phase) * 2.5;
	let handBackTarget: Point = {
		x: shoulderBack.x - 6,
		y: shoulderBack.y + metrics.upperArm + metrics.lowerArm - 2 + idleSwing
	};
	let handFrontTarget: Point = {
		x: shoulderFront.x + 6,
		y: shoulderFront.y + metrics.upperArm + metrics.lowerArm - 2 - idleSwing
	};

	if (clip === 'walk') {
		handBackTarget.x += Math.sin(phase + Math.PI) * 5;
		handFrontTarget.x += Math.sin(phase) * 5;
	}

	let weaponAngle = -0.55;
	if (clip === 'attack') {
		const swing = Math.sin(clamp(t, 0, 1) * Math.PI);
		handFrontTarget = {
			x: shoulderFront.x + Math.cos(-1.9 + swing * 2.6) * (metrics.upperArm + metrics.lowerArm) * 0.7,
			y: shoulderFront.y + Math.sin(-1.9 + swing * 2.6) * (metrics.upperArm + metrics.lowerArm) * 0.7
		};
		handBackTarget = {
			x: shoulderBack.x - 2,
			y: shoulderBack.y + metrics.upperArm + metrics.lowerArm - 4
		};
		weaponAngle = -1.9 + swing * 2.9;
	}

	const armBack = twoBoneIK(
		shoulderBack,
		handBackTarget,
		metrics.upperArm,
		metrics.lowerArm,
		1
	);
	const armFront = twoBoneIK(
		shoulderFront,
		handFrontTarget,
		metrics.upperArm,
		metrics.lowerArm,
		-1
	);

	return {
		hip,
		chest,
		head,
		shoulderBack,
		elbowBack: armBack.joint,
		handBack: armBack.end,
		shoulderFront,
		elbowFront: armFront.joint,
		handFront: armFront.end,
		hipBack: hip,
		kneeBack: legBack.joint,
		footBack: legBack.end,
		hipFront: hip,
		kneeFront: legFront.joint,
		footFront: legFront.end,
		weaponAngle,
		weaponHand: armFront.end,
		headAngle: lean * 0.6 + Math.sin(phase) * 0.02
	};
}
