<script lang="ts">
	import { get } from 'svelte/store';
	import type { CardDef, Side } from '#lib/types';
	import { content } from '#lib/stores/content';
	import { drawCardIcon } from '#lib/game/cardArt';
	import { spritesVersion } from '#lib/game/sprites';

	interface Props {
		card: CardDef;
		size?: number;
		side?: Side;
	}

	let { card, size = 96, side = 0 }: Props = $props();
	let canvas: HTMLCanvasElement;

	$effect(() => {
		const version = $spritesVersion;
		void version;
		const loaded = get(content);
		if (!canvas || !loaded) {
			return;
		}
		const dpr = Math.min(2, window.devicePixelRatio || 1);
		const pixels = Math.round(size * dpr);
		if (canvas.width !== pixels || canvas.height !== pixels) {
			canvas.width = pixels;
			canvas.height = pixels;
		}
		const context = canvas.getContext('2d');
		if (!context) {
			return;
		}
		context.setTransform(dpr, 0, 0, dpr, 0, 0);
		context.clearRect(0, 0, size, size);
		drawCardIcon(context, card, loaded.definitions, 0, 0, size, side);
	});
</script>

<canvas
	bind:this={canvas}
	style:width="100%"
	style:height="100%"
	aria-label={card.slug}
></canvas>
