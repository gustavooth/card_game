import { writable } from 'svelte/store';
import type { Content, Side, TroopDef, VisualDef } from '#lib/types';
import {
	CLIP_FRAMES,
	SPRITE_SIZE,
	drawCharacter,
	normalizeView,
	supportsBackView
} from './characterArt';
import type { SpriteView } from './characterArt';
import type { ClipName } from './skeleton';

export type { SpriteView };

type SpriteCanvas = OffscreenCanvas | HTMLCanvasElement;

export interface BakedCharacter {
	key: string;
	visual: VisualDef;
	side: Side;
	view: SpriteView;
	clips: Record<ClipName, SpriteCanvas[]>;
}

const cache = new Map<string, BakedCharacter>();
export const spritesVersion = writable(0);

function createCanvas(width: number, height: number): SpriteCanvas {
	if (typeof OffscreenCanvas !== 'undefined') {
		return new OffscreenCanvas(width, height);
	}
	const canvas = document.createElement('canvas');
	canvas.width = width;
	canvas.height = height;
	return canvas;
}

export function visualKey(
	visual: VisualDef,
	side: Side,
	view: SpriteView = 'front',
	slug = ''
): string {
	return [
		slug,
		side,
		normalizeView(visual, view),
		visual.archetype,
		visual.scale,
		visual.primary,
		visual.secondary,
		visual.weapon,
		visual.accessory
	].join('|');
}

export function bakeCharacter(
	visual: VisualDef,
	side: Side,
	view: SpriteView = 'front',
	slug = ''
): BakedCharacter {
	const key = visualKey(visual, side, view, slug);
	const cached = cache.get(key);
	if (cached) {
		return cached;
	}
	const effectiveView = normalizeView(visual, view);
	const clips = {} as Record<ClipName, SpriteCanvas[]>;
	for (const clip of Object.keys(CLIP_FRAMES) as ClipName[]) {
		const frames = CLIP_FRAMES[clip];
		const list: SpriteCanvas[] = [];
		for (let frame = 0; frame < frames; frame += 1) {
			const canvas = createCanvas(SPRITE_SIZE, SPRITE_SIZE);
			const context = canvas.getContext('2d') as
				| OffscreenCanvasRenderingContext2D
				| CanvasRenderingContext2D
				| null;
			if (!context) {
				continue;
			}
			context.clearRect(0, 0, SPRITE_SIZE, SPRITE_SIZE);
			drawCharacter(
				context as CanvasRenderingContext2D,
				visual,
				side,
				clip,
				frame,
				frames,
				effectiveView
			);
			list.push(canvas);
		}
		clips[clip] = list;
	}
	const baked = { key, visual, side, view: effectiveView, clips };
	cache.set(key, baked);
	return baked;
}

export function bakedFor(troop: TroopDef, side: Side, view: SpriteView = 'front'): BakedCharacter {
	return bakeCharacter(troop.visual, side, view, troop.slug);
}

export function ensureBaked(
	visual: VisualDef,
	side: Side,
	view: SpriteView = 'front',
	slug = ''
): BakedCharacter {
	return getBaked(visual, side, view, slug) ?? bakeCharacter(visual, side, view, slug);
}

export function getBaked(
	visual: VisualDef,
	side: Side,
	view: SpriteView = 'front',
	slug = ''
): BakedCharacter | undefined {
	return cache.get(visualKey(visual, side, view, slug));
}

export async function bakeAll(content: Content, onProgress?: (value: number) => void) {
	const troops = Object.values(content.definitions.troops);
	const total = troops.length;
	let index = 0;
	for (const troop of troops) {
		bakeCharacter(troop.visual, 0, 'front', troop.slug);
		bakeCharacter(troop.visual, 1, 'front', troop.slug);
		if (supportsBackView(troop.visual)) {
			bakeCharacter(troop.visual, 0, 'back', troop.slug);
			bakeCharacter(troop.visual, 1, 'back', troop.slug);
		}
		index += 1;
		onProgress?.(index / total);
		if (index % 3 === 0) {
			await new Promise((resolve) => setTimeout(resolve, 0));
		}
	}
	onProgress?.(1);
	spritesVersion.update((value) => value + 1);
}

export async function ensureSprites(content: Content, onProgress?: (value: number) => void) {
	let expected = 0;
	for (const troop of Object.values(content.definitions.troops)) {
		expected += supportsBackView(troop.visual) ? 4 : 2;
	}
	if (cache.size >= expected) {
		return;
	}
	await bakeAll(content, onProgress);
}

export function frameFor(clip: ClipName, elapsedMs: number): number {
	const frames = CLIP_FRAMES[clip];
	const fps = clip === 'attack' ? 18 : clip === 'walk' ? 12 : 6;
	return Math.floor((elapsedMs / 1000) * fps) % frames;
}

export interface DrawSpriteOptions {
	x: number;
	y: number;
	size: number;
	facing: number;
	clip: ClipName;
	elapsedMs: number;
	alpha?: number;
	rotation?: number;
}

export function drawBaked(
	ctx: CanvasRenderingContext2D,
	baked: BakedCharacter,
	options: DrawSpriteOptions
) {
	const clip = baked.clips[options.clip];
	if (!clip || clip.length === 0) {
		return;
	}
	const frame = frameFor(options.clip, options.elapsedMs);
	const sprite = clip[frame % clip.length];
	const size = options.size;
	const x = options.x;
	const y = options.y;
	ctx.save();
	ctx.globalAlpha = options.alpha ?? 1;
	if (options.rotation) {
		ctx.translate(x, y);
		ctx.rotate(options.rotation);
		ctx.translate(-x, -y);
	}
	if (options.facing < 0) {
		ctx.translate(x, 0);
		ctx.scale(-1, 1);
		ctx.translate(-x, 0);
	}
	const top = y - size * 0.875;
	ctx.drawImage(sprite as CanvasImageSource, x - size / 2, top, size, size);
	ctx.restore();
}

export function drawIdleThumb(
	ctx: CanvasRenderingContext2D,
	troop: TroopDef,
	side: Side,
	x: number,
	y: number,
	size: number
) {
	const baked = bakedFor(troop, side);
	const frame = baked.clips.idle[0];
	if (!frame) {
		return;
	}
	ctx.drawImage(frame as CanvasImageSource, x, y, size, size);
}

export function clearSpriteCache() {
	cache.clear();
}
