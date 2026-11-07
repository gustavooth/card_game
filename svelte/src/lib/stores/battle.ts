import { derived, get, writable } from 'svelte/store';
import { gameSocket } from '#lib/net/socket';
import type { EndInfo, MatchInfo, Snapshot } from '#lib/types';
import { appendChat, chatConnected } from './chat';

export interface SnapshotFrame {
	snapshot: Snapshot;
	at: number;
}

export interface BattleError {
	code: string;
	message: string;
	at: number;
}

export const match = writable<MatchInfo | null>(null);
export const frames = writable<SnapshotFrame[]>([]);
export const result = writable<EndInfo | null>(null);
export const queuePosition = writable<number | null>(null);
export const battleError = writable<BattleError | null>(null);
export const socketConnected = writable(false);
export const matchActive = derived([match, result], ([$match, $result]) => !!$match && !$result);

let initialized = false;

export function initBattleSocket() {
	if (initialized || typeof window === 'undefined') {
		return;
	}
	initialized = true;
	gameSocket.onStatus((connected) => {
		socketConnected.set(connected);
		chatConnected.set(connected);
	});
	gameSocket.onMessage((message) => {
		const type = message.t;
		switch (type) {
			case 's': {
				const snapshot = message as unknown as Snapshot;
				const now = performance.now();
				frames.update((current) => [...current.slice(-5), { snapshot, at: now }]);
				break;
			}
			case 'match': {
				const info: MatchInfo = {
					room: String(message.r ?? ''),
					side: (Number(message.s ?? 0) as 0 | 1),
					opponent: String(message.o ?? ''),
					arena: String(message.a ?? 'classic'),
					reconnect: Boolean(message.rc)
				};
				match.set(info);
				result.set(null);
				frames.set([]);
				queuePosition.set(null);
				battleError.set(null);
				break;
			}
			case 'end': {
				const info: EndInfo = {
					result: (message.res as EndInfo['result']) ?? 'draw',
					winner: message.w === null || message.w === undefined ? null : (Number(message.w) as 0 | 1),
					crowns: [
						Number((message.c as number[])?.[0] ?? 0),
						Number((message.c as number[])?.[1] ?? 0)
					],
					damage: [
						Number((message.d as number[])?.[0] ?? 0),
						Number((message.d as number[])?.[1] ?? 0)
					],
					duration_ms: Number(message.m ?? 0),
					reason: String(message.rs ?? '')
				};
				result.set(info);
				break;
			}
			case 'queue':
				queuePosition.set(Number(message.p ?? 1));
				break;
			case 'chat': {
				appendChat({
					user: String(message.u ?? '?'),
					text: String(message.m ?? ''),
					ts: Number(message.ts ?? Date.now())
				});
				break;
			}
			case 'err': {
				battleError.set({
					code: String(message.c ?? 'error'),
					message: String(message.m ?? ''),
					at: Date.now()
				});
				break;
			}
			default:
				break;
		}
	});
	gameSocket.connect();
}

export function ensureSocket() {
	initBattleSocket();
	gameSocket.connect();
}

export function joinQueue() {
	ensureSocket();
	gameSocket.joinQueue();
}

export function leaveQueue() {
	queuePosition.set(null);
	gameSocket.leaveQueue();
}

export function deploy(slot: number, x: number, y: number) {
	return gameSocket.deploy(slot, x, y);
}

export function sendChat(text: string) {
	return gameSocket.chat(text);
}

export function resetBattle() {
	match.set(null);
	result.set(null);
	frames.set([]);
	queuePosition.set(null);
	battleError.set(null);
}

export function latestFrame(): SnapshotFrame | undefined {
	const list = get(frames);
	return list[list.length - 1];
}

export function clearBattleError() {
	battleError.set(null);
}
