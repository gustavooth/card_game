<script lang="ts">
	import '../app.css';
	import favicon from '#lib/assets/favicon.svg';
	import { audio } from '#lib/audio/synth';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();

	function ensureAudio() {
		audio.ensure();
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<meta name="theme-color" content="#060a13" />
</svelte:head>

<svelte:window onpointerdown={ensureAudio} onkeydown={ensureAudio} />

<div class="app-bg">
	{@render children()}
</div>
