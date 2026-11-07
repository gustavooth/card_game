<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		onclose: () => void;
		children: Snippet;
		wide?: boolean;
	}

	let { title, onclose, children, wide = false }: Props = $props();

	function onKey(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			onclose();
		}
	}
</script>

<svelte:window onkeydown={onKey} />

<div
	class="modal-backdrop"
	role="presentation"
	onclick={onclose}
>
	<div
		class="modal"
		style:width={wide ? 'min(760px, 100%)' : undefined}
		role="dialog"
		tabindex="-1"
		aria-modal="true"
		aria-label={title}
		onclick={(event) => event.stopPropagation()}
		onkeydown={(event) => event.stopPropagation()}
	>
		<div class="spread" style="margin-bottom: 14px">
			<h2 style="font-size: 20px; font-weight: 900">{title}</h2>
			<button class="btn btn-sm" onclick={onclose} type="button">✕</button>
		</div>
		{@render children()}
	</div>
</div>
