import { get } from 'svelte/store';
import type { CardDef, Content, MatchInfo, Snapshot } from '#lib/types';
import { frames } from '#lib/stores/battle';
import { audio } from '#lib/audio/synth';
import { DESIGN } from './constants';
import { battleFramePair, renderFrame, type RenderPair } from './renderer';
import { insideArena, screenToWorld } from './projection';
import { handCardAt } from './hud';
import { validatePlacement } from './placement';
import { drawArenaGround } from './arena';

export type DeployHandler = (slot: number, x: number, y: number) => void;
export type ErrorHandler = (code: string) => void;

export class GameView {
	private context: CanvasRenderingContext2D;
	private running = false;
	private raf = 0;
	private selectedSlot: number | null = null;
	private hover: { x: number; y: number } | null = null;
	private pointerDown = false;
	private lastTick = -1;
	private latest: Snapshot | null = null;

	constructor(
		private canvas: HTMLCanvasElement,
		private content: Content,
		private match: MatchInfo,
		private username: string,
		private onDeploy: DeployHandler,
		private onError: ErrorHandler
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
		if (import.meta.env.DEV) {
			(window as unknown as { __royaleGameView?: GameView }).__royaleGameView = this;
		}
		this.attach();
	}

	start() {
		if (this.running) {
			return;
		}
		this.running = true;
		this.render(performance.now());
		const loop = (now: number) => {
			if (!this.running) {
				return;
			}
			this.render(now);
			this.raf = requestAnimationFrame(loop);
		};
		this.raf = requestAnimationFrame(loop);
	}

	stop() {
		this.running = false;
		cancelAnimationFrame(this.raf);
		this.detach();
	}

	clearSelection() {
		this.selectedSlot = null;
		this.hover = null;
	}

	private localSnapshot(pair: RenderPair | null): Snapshot | null {
		return pair?.latest ?? null;
	}

	private currentSnapshot(): Snapshot | null {
		const list = get(frames);
		return list[list.length - 1]?.snapshot ?? this.latest;
	}

	private cardAt(slot: number, snapshot: Snapshot | null): CardDef | undefined {
		if (!snapshot) {
			return undefined;
		}
		const local = snapshot.p.find((player) => player.s === this.match.side);
		const entry = local?.h[slot];
		if (!entry) {
			return undefined;
		}
		return this.content.definitions.cards[entry[0]];
	}

	private localElixir(snapshot: Snapshot | null): number {
		if (!snapshot) {
			return 0;
		}
		return snapshot.p.find((player) => player.s === this.match.side)?.e ?? 0;
	}

	private render(now: number) {
		const pair = battleFramePair(get(frames), now);
		if (!pair) {
			const arena =
				this.content.definitions.arenas[this.match.arena] ??
				Object.values(this.content.definitions.arenas)[0];
			if (arena) {
				drawArenaGround(this.context, arena, now);
			}
			return;
		}
		if (pair.latest.k !== this.lastTick) {
			audio.handleEvents(pair.latest.ev, this.match.side);
			this.lastTick = pair.latest.k;
		}
		this.latest = pair.latest;
		const card = this.selectedSlot !== null ? this.cardAt(this.selectedSlot, pair.latest) : undefined;
		let valid = false;
		if (card && this.hover) {
			const world = screenToWorld(this.hover.x, this.hover.y, this.match.side);
			valid = validatePlacement(
				this.content,
				card,
				world.x,
				world.y,
				pair.latest,
				this.match.side
			);
		}
		renderFrame(
			this.context,
			this.content,
			this.match,
			pair,
			{
				selectedSlot: this.selectedSlot,
				hover: this.hover,
				valid,
				cardKind: card?.type.kind ?? 'troop'
			},
			{
				opponent: this.match.opponent,
				username: this.username,
				selectedSlot: this.selectedSlot
			},
			now
		);
		this.selectedCardIsValid = valid;
		this.selectedCard = card;
	}

	private selectedCard: CardDef | undefined;
	private selectedCardIsValid = false;

	private toDesign(event: PointerEvent) {
		const rect = this.canvas.getBoundingClientRect();
		return {
			x: ((event.clientX - rect.left) / rect.width) * DESIGN.width,
			y: ((event.clientY - rect.top) / rect.height) * DESIGN.height
		};
	}

	private onPointerDown = (event: PointerEvent) => {
		event.preventDefault();
		audio.ensure();
		const point = this.toDesign(event);
		const slot = handCardAt(point.x, point.y);
		if (slot !== null) {
			const snapshot = this.currentSnapshot();
			const card = this.cardAt(slot, snapshot);
			if (!card) {
				return;
			}
			this.selectedSlot = this.selectedSlot === slot ? null : slot;
			this.hover = null;
			this.pointerDown = true;
			try {
				this.canvas.setPointerCapture(event.pointerId);
			} catch {
				// ponteiro sintético ou já liberado
			}
			return;
		}
		if (this.selectedSlot !== null && insideArena(point.x, point.y)) {
			this.pointerDown = true;
			this.hover = point;
			try {
				this.canvas.setPointerCapture(event.pointerId);
			} catch {
				// ponteiro sintético ou já liberado
			}
			return;
		}
		this.clearSelection();
	};

	private onPointerMove = (event: PointerEvent) => {
		if (this.selectedSlot === null) {
			return;
		}
		event.preventDefault();
		const point = this.toDesign(event);
		this.hover = insideArena(point.x, point.y) ? point : null;
	};

	private onPointerUp = (event: PointerEvent) => {
		if (!this.pointerDown) {
			return;
		}
		this.pointerDown = false;
		if (this.selectedSlot === null) {
			return;
		}
		const point = this.toDesign(event);
		if (!insideArena(point.x, point.y)) {
			if (handCardAt(point.x, point.y) === null) {
				this.hover = null;
			}
			return;
		}
		const snapshot = this.currentSnapshot();
		const card = this.cardAt(this.selectedSlot, snapshot);
		if (!card) {
			return;
		}
		const world = screenToWorld(point.x, point.y, this.match.side);
		const valid = validatePlacement(
			this.content,
			card,
			world.x,
			world.y,
			snapshot,
			this.match.side
		);
		if (!valid) {
			this.onError('invalid_placement');
			audio.play('error');
			return;
		}
		const cost = card.elixir_cost;
		if (this.localElixir(snapshot) + 0.01 < cost) {
			this.onError('not_enough_elixir');
			audio.play('error');
			return;
		}
		const slot = this.selectedSlot;
		this.selectedSlot = null;
		this.hover = null;
		audio.play('deploy');
		this.onDeploy(slot, world.x, world.y);
	};

	private onPointerCancel = () => {
		this.pointerDown = false;
		this.hover = null;
	};

	private onKeyDown = (event: KeyboardEvent) => {
		if (event.key === 'Escape') {
			this.clearSelection();
			return;
		}
		const index = Number(event.key) - 1;
		if (index >= 0 && index <= 3) {
			const card = this.cardAt(index, this.latest);
			if (card) {
				this.selectedSlot = index;
				this.hover = null;
			}
		}
	};

	private attach() {
		this.canvas.addEventListener('pointerdown', this.onPointerDown);
		this.canvas.addEventListener('pointermove', this.onPointerMove);
		this.canvas.addEventListener('pointerup', this.onPointerUp);
		this.canvas.addEventListener('pointercancel', this.onPointerCancel);
		window.addEventListener('keydown', this.onKeyDown);
	}

	private detach() {
		this.canvas.removeEventListener('pointerdown', this.onPointerDown);
		this.canvas.removeEventListener('pointermove', this.onPointerMove);
		this.canvas.removeEventListener('pointerup', this.onPointerUp);
		this.canvas.removeEventListener('pointercancel', this.onPointerCancel);
		window.removeEventListener('keydown', this.onKeyDown);
	}
}
