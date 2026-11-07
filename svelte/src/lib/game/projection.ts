import { ARENA_HEIGHT, ARENA_LAYOUT, ARENA_WIDTH } from './constants';
import type { Side } from '#lib/types';

export interface ScreenPoint {
	x: number;
	y: number;
}

export function worldToScreen(x: number, y: number, side: Side): ScreenPoint {
	const localX = side === 0 ? x : ARENA_WIDTH - x;
	const localY = side === 0 ? y : ARENA_HEIGHT - y;
	return {
		x: ARENA_LAYOUT.x + localX * ARENA_LAYOUT.tile,
		y: ARENA_LAYOUT.y + localY * ARENA_LAYOUT.tile * ARENA_LAYOUT.depth
	};
}

export function screenToWorld(px: number, py: number, side: Side): ScreenPoint {
	const localX = (px - ARENA_LAYOUT.x) / ARENA_LAYOUT.tile;
	const localY = (py - ARENA_LAYOUT.y) / (ARENA_LAYOUT.tile * ARENA_LAYOUT.depth);
	return {
		x: side === 0 ? localX : ARENA_WIDTH - localX,
		y: side === 0 ? localY : ARENA_HEIGHT - localY
	};
}

export function screenToLocal(px: number, py: number): ScreenPoint {
	return {
		x: (px - ARENA_LAYOUT.x) / ARENA_LAYOUT.tile,
		y: (py - ARENA_LAYOUT.y) / (ARENA_LAYOUT.tile * ARENA_LAYOUT.depth)
	};
}

export function insideArena(px: number, py: number): boolean {
	const local = screenToLocal(px, py);
	return local.x >= 0 && local.x <= ARENA_WIDTH && local.y >= 0 && local.y <= ARENA_HEIGHT;
}

export function depthOf(y: number, side: Side): number {
	const localY = side === 0 ? y : ARENA_HEIGHT - y;
	return localY;
}
