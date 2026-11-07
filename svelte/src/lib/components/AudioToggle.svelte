<script lang="ts">
	import { onMount } from 'svelte';
	import { audio } from '#lib/audio/synth';

	let muted = $state(false);

	onMount(() => {
		muted = audio.muted;
	});

	function toggle() {
		audio.ensure();
		audio.toggleMute();
		muted = audio.muted;
		audio.play('click');
	}
</script>

<button class="btn btn-icon" onclick={toggle} type="button" aria-label="Som" title="Som">
	{#if muted}
		<svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
			<path d="M4 9v6h4l5 4V5L8 9H4Z" fill="currentColor" />
			<path d="m16 9 5 6m0-6-5 6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
		</svg>
	{:else}
		<svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
			<path d="M4 9v6h4l5 4V5L8 9H4Z" fill="currentColor" />
			<path d="M16 9a4 4 0 0 1 0 6M18.5 6.5a7.5 7.5 0 0 1 0 11" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
		</svg>
	{/if}
</button>
