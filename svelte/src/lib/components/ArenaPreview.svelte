<script lang="ts">
	import { get } from 'svelte/store';
	import { content } from '#lib/stores/content';
	import { drawTower } from '#lib/game/arena';
	import { drawBaked, ensureBaked, spritesVersion } from '#lib/game/sprites';
	import { TEAM_COLORS } from '#lib/game/constants';
	import type { Content } from '#lib/types';

	let canvas: HTMLCanvasElement;
	const width = 640;
	const height = 640;

	$effect(() => {
		const loaded = get(content);
		const version = $spritesVersion;
		void version;
		if (!canvas || !loaded) {
			return;
		}
		const dpr = Math.min(2, window.devicePixelRatio || 1);
		canvas.width = width * dpr;
		canvas.height = height * dpr;
		const context = canvas.getContext('2d');
		if (!context) {
			return;
		}
		context.setTransform(dpr, 0, 0, dpr, 0, 0);
		draw(context, loaded);
	});

	function draw(context: CanvasRenderingContext2D, loaded: Content) {
		context.clearRect(0, 0, width, height);
		const backdrop = context.createLinearGradient(0, 0, 0, height);
		backdrop.addColorStop(0, '#13233f');
		backdrop.addColorStop(0.55, '#0b1728');
		backdrop.addColorStop(1, '#070d18');
		context.fillStyle = backdrop;
		context.fillRect(0, 0, width, height);

		// iluminação do diorama
		const glow = context.createRadialGradient(width / 2, height / 2, 40, width / 2, height / 2, 300);
		glow.addColorStop(0, 'rgba(96, 165, 250, 0.22)');
		glow.addColorStop(1, 'rgba(96, 165, 250, 0)');
		context.fillStyle = glow;
		context.fillRect(0, 0, width, height);

		// base de pedra
		context.save();
		context.beginPath();
		context.roundRect(52, 24, width - 104, height - 36, 26);
		context.fillStyle = '#1f2937';
		context.fill();
		context.strokeStyle = '#0f172a';
		context.lineWidth = 5;
		context.stroke();
		context.restore();

		// plataforma
		const left = 60;
		const top = 60;
		const right = width - 60;
		const bottom = height - 60;
		context.save();
		context.beginPath();
		context.roundRect(left, top, right - left, bottom - top, 18);
		context.clip();
		context.fillStyle = '#3f7a3c';
		context.fillRect(left, top, right - left, bottom - top);

		// grama
		const tile = 24;
		for (let y = top; y < bottom; y += tile) {
			for (let x = left; x < right; x += tile) {
				const noise = Math.abs(Math.sin(x * 12.98 + y * 78.23) * 43758.54) % 1;
				context.fillStyle = `rgb(${52 + Math.floor(noise * 12)}, ${118 + Math.floor(noise * 20)}, ${55 + Math.floor(noise * 10)})`;
				context.fillRect(x, y, tile + 1, tile + 1);
			}
		}

		// rio
		const riverY = top + (bottom - top) * 0.5;
		const river = context.createLinearGradient(0, riverY - 16, 0, riverY + 16);
		river.addColorStop(0, '#0ea5e9');
		river.addColorStop(0.5, '#0284c7');
		river.addColorStop(1, '#075985');
		context.fillStyle = river;
		context.fillRect(left, riverY - 14, right - left, 28);
		context.fillStyle = '#8b5a2b';
		for (const bx of [164, 404]) {
			context.fillRect(bx, riverY - 22, 72, 44);
			context.strokeStyle = '#4a2c12';
			context.lineWidth = 2;
			context.strokeRect(bx, riverY - 22, 72, 44);
			for (let plank = 0; plank < 4; plank += 1) {
				context.beginPath();
				context.moveTo(bx + 4, riverY - 18 + plank * 11);
				context.lineTo(bx + 68, riverY - 18 + plank * 11);
				context.stroke();
			}
		}

		// caminhos de pedra
		context.fillStyle = 'rgba(120, 113, 108, 0.55)';
		for (const bx of [200, 440]) {
			context.fillRect(bx - 12, top + 40, 24, riverY - top - 52);
			context.fillRect(bx - 12, riverY + 12, 24, bottom - riverY - 36);
		}
		context.fillRect(left + 80, 200, right - left - 160, 18);
		context.fillRect(left + 80, 422, right - left - 160, 18);

		// borda de pedra
		context.strokeStyle = 'rgba(203, 213, 225, 0.25)';
		context.lineWidth = 8;
		context.strokeRect(left + 4, top + 4, right - left - 8, bottom - top - 8);
		context.restore();

		// moldura dourada da plataforma
		context.save();
		context.strokeStyle = 'rgba(247, 212, 92, 0.75)';
		context.lineWidth = 2;
		context.strokeRect(left + 6, top + 6, right - left - 12, bottom - top - 12);
		for (const [cx, cy] of [
			[left + 6, top + 6],
			[right - 6, top + 6],
			[left + 6, bottom - 6],
			[right - 6, bottom - 6]
		]) {
			context.beginPath();
			context.arc(cx, cy, 4, 0, Math.PI * 2);
			context.fillStyle = '#f7d45c';
			context.fill();
			context.strokeStyle = '#5d3603';
			context.lineWidth = 1;
			context.stroke();
		}
		context.restore();

		// torres
		const princess = loaded.definitions.towers['princess_tower'];
		const king = loaded.definitions.towers['king_tower'];
		if (!princess || !king) {
			return;
		}
		const towers = [
			{ def: princess, side: 1 as const, x: 200, y: 224, role: 'Princess' },
			{ def: princess, side: 1 as const, x: 440, y: 224, role: 'Princess' },
			{ def: king, side: 1 as const, x: 320, y: 200, role: 'King' },
			{ def: princess, side: 0 as const, x: 200, y: 416, role: 'Princess' },
			{ def: princess, side: 0 as const, x: 440, y: 416, role: 'Princess' },
			{ def: king, side: 0 as const, x: 320, y: 350, role: 'King' }
		];
		for (const tower of towers) {
			drawTower(
				context,
				tower.def,
				tower.side,
				tower.x,
				tower.y,
				tower.def.health,
				tower.def.health,
				tower.role === 'Princess',
				tower.role
			);
		}

		// tropas decorativas
		const units = [
			{ slug: 'knight', side: 0 as const, x: 300, y: 390, size: 56 },
			{ slug: 'giant', side: 0 as const, x: 236, y: 398, size: 88 },
			{ slug: 'archer', side: 1 as const, x: 386, y: 260, size: 48 },
			{ slug: 'musketeer', side: 1 as const, x: 420, y: 270, size: 54 }
		];
		const sorted = [...units].sort((a, b) => a.y - b.y);
		for (const unit of sorted) {
			const troop = loaded.definitions.troops[unit.slug];
			if (!troop) {
				continue;
			}
			const baked = ensureBaked(troop.visual, unit.side, 'front', troop.slug);
			context.beginPath();
			context.ellipse(unit.x, unit.y + 2, unit.size * 0.22, unit.size * 0.08, 0, 0, Math.PI * 2);
			context.fillStyle = 'rgba(0,0,0,0.3)';
			context.fill();
			drawBaked(context, baked, {
				x: unit.x,
				y: unit.y,
				size: unit.size,
				facing: 1,
				clip: 'idle',
				elapsedMs: 120 * unit.x
			});
			if (unit.side === 0) {
				context.beginPath();
				context.ellipse(unit.x, unit.y, unit.size * 0.24, unit.size * 0.1, 0, 0, Math.PI * 2);
				context.strokeStyle = TEAM_COLORS.blue.glow;
				context.lineWidth = 2;
				context.stroke();
			}
		}

		// moldura luminosa
		context.beginPath();
		context.roundRect(52, 24, width - 104, height - 36, 26);
		context.strokeStyle = 'rgba(250, 204, 21, 0.35)';
		context.lineWidth = 2;
		context.stroke();
	}
</script>

<canvas bind:this={canvas} style:width="100%" style:height="100%" aria-label="Arena clássica"></canvas>
