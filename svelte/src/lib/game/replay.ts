import { decode } from '@msgpack/msgpack';
import type { Content, ReplayHeader, Snapshot } from '#lib/types';
import { DESIGN } from './constants';
import { renderFrame, type RenderPair } from './renderer';
import { audio } from '#lib/audio/synth';

export interface ReplayData {
	interval: number;
	meta: ReplayHeader;
	frames: Snapshot[];
}

async function maybeDecompress(buffer: ArrayBuffer): Promise<ArrayBuffer> {
	const bytes = new Uint8Array(buffer);
	if (bytes.length > 2 && bytes[0] === 0x1f && bytes[1] === 0x8b) {
		const stream = new Blob([buffer]).stream().pipeThrough(new DecompressionStream('gzip'));
		return new Response(stream).arrayBuffer();
	}
	return buffer;
}

export async function parseReplay(buffer: ArrayBuffer): Promise<ReplayData> {
	const raw = await maybeDecompress(buffer);
	const bytes = new Uint8Array(raw);
	const view = new DataView(raw);
	const magic = String.fromCharCode(bytes[0], bytes[1], bytes[2], bytes[3]);
	if (magic !== 'RYL2' && magic !== 'RYL1') {
		throw new Error('Formato de replay inválido');
	}
	const interval = view.getUint16(4, true);
	let offset = 6;
	let meta: ReplayHeader = {
		side: 0,
		arena: 'classic',
		blue: 'Blue',
		red: 'Red',
		started_at: 0,
		opponent: 'Red',
		result: 'draw',
		crowns: [0, 0]
	};
	if (magic === 'RYL2') {
		const metaLength = view.getUint32(offset, true);
		offset += 4;
		const metaBytes = new Uint8Array(raw, offset, metaLength);
		meta = JSON.parse(new TextDecoder().decode(metaBytes)) as ReplayHeader;
		offset += metaLength;
	}
	const frameCount = view.getUint32(offset, true);
	offset += 4;
	const frames: Snapshot[] = [];
	for (let index = 0; index < frameCount; index += 1) {
		if (offset + 4 > raw.byteLength) {
			break;
		}
		const length = view.getUint32(offset, true);
		offset += 4;
		if (offset + length > raw.byteLength) {
			break;
		}
		const payload = new Uint8Array(raw, offset, length);
		offset += length;
		try {
			frames.push(decode(payload) as Snapshot);
		} catch {
			// frame corrompido: ignora
		}
	}
	return { interval, meta, frames };
}

function pairAt(frames: Snapshot[], timeMs: number): RenderPair | null {
	if (frames.length === 0) {
		return null;
	}
	let prev = frames[0];
	let next = frames[0];
	for (let index = 0; index < frames.length; index += 1) {
		if (frames[index].m <= timeMs) {
			prev = frames[index];
			next = frames[index + 1] ?? frames[index];
		}
	}
	if (next === prev) {
		return { prev, next, alpha: 0, latest: next };
	}
	const alpha = Math.max(
		0,
		Math.min(1, (timeMs - prev.m) / Math.max(1, next.m - prev.m))
	);
	return { prev, next, alpha, latest: next };
}

export type ReplayProgress = (time: number, duration: number, playing: boolean) => void;

export class ReplayView {
	private context: CanvasRenderingContext2D;
	private running = false;
	private raf = 0;
	private playing = true;
	private speed = 1;
	private time = 0;
	private duration = 0;
	private lastNow = 0;
	private lastTick = -1;
	private username: string;
	private opponent: string;

	constructor(
		private canvas: HTMLCanvasElement,
		private content: Content,
		private data: ReplayData,
		private onProgress: ReplayProgress
	) {
		const context = canvas.getContext('2d');
		if (!context) {
			throw new Error('Canvas 2D indisponível');
		}
		this.context = context;
		const dpr = Math.min(2, window.devicePixelRatio || 1);
		canvas.width = Math.floor(DESIGN.width * dpr);
		canvas.height = Math.floor(DESIGN.height * dpr);
		context.setTransform(dpr, 0, 0, dpr, 0, 0);
		this.username = data.meta.side === 0 ? data.meta.blue : data.meta.red;
		this.opponent = data.meta.opponent;
		this.duration = data.frames.length
			? data.frames[data.frames.length - 1].m / 1000
			: 0;
	}

	start() {
		if (this.running) {
			return;
		}
		this.running = true;
		this.lastNow = performance.now();
		this.tick(this.lastNow);
		const loop = (now: number) => {
			if (!this.running) {
				return;
			}
			this.tick(now);
			this.raf = requestAnimationFrame(loop);
		};
		this.raf = requestAnimationFrame(loop);
	}

	private tick(now: number) {
		const delta = Math.min(0.1, (now - this.lastNow) / 1000);
		this.lastNow = now;
		if (this.playing) {
			this.time = Math.min(this.duration, this.time + delta * this.speed);
			if (this.time >= this.duration) {
				this.playing = false;
			}
		}
		const pair = pairAt(this.data.frames, this.time * 1000);
		if (pair) {
			if (pair.latest.k !== this.lastTick) {
				this.lastTick = pair.latest.k;
				audio.handleEvents(pair.latest.ev, this.data.meta.side);
			}
			renderFrame(
				this.context,
				this.content,
				{
					room: '',
					side: this.data.meta.side,
					opponent: this.opponent,
					arena: this.data.meta.arena,
					reconnect: false
				},
				pair,
				{ selectedSlot: null, hover: null, valid: false, cardKind: 'troop', drag: null, dragCard: null },
				{ opponent: this.opponent, username: this.username, selectedSlot: null },
				now
			);
		}
		this.onProgress(this.time, this.duration, this.playing);
	}

	stop() {
		this.running = false;
		cancelAnimationFrame(this.raf);
	}

	setPlaying(playing: boolean) {
		if (playing && this.time >= this.duration) {
			this.time = 0;
		}
		this.playing = playing;
	}

	toggle() {
		this.setPlaying(!this.playing);
	}

	setSpeed(speed: number) {
		this.speed = speed;
	}

	seek(time: number) {
		this.time = Math.max(0, Math.min(this.duration, time));
	}

	get isPlaying() {
		return this.playing;
	}
}
