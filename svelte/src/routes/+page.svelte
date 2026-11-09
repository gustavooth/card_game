<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { get } from 'svelte/store';
	import { i18n } from '#lib/i18n';
	import { loadContent } from '#lib/stores/content';
	import { restoreSession, user } from '#lib/stores/session';
	import { bakeAll } from '#lib/game/sprites';
	import { ensureSocket } from '#lib/stores/battle';
	import CrownIcon from '#lib/components/CrownIcon.svelte';
	import { audio } from '#lib/audio/synth';

	let progress = $state(0.02);
	let stage = $state<'content' | 'session' | 'sprites' | 'audio' | 'ready'>('content');
	let error = $state<string | null>(null);

	const stages = ['content', 'session', 'sprites', 'audio'] as const;

	onMount(() => {
		void boot();
	});

	async function boot() {
		try {
			error = null;
			stage = 'content';
			progress = 0.05;
			const loaded = await loadContent(true);
			progress = 0.15;

			stage = 'session';
			const restored = await restoreSession();
			progress = 0.25;

			stage = 'sprites';
			await bakeAll(loaded, (value) => {
				progress = 0.25 + value * 0.62;
			});
			progress = 0.9;

			stage = 'audio';
			audio.ensure();
			progress = 0.98;

			stage = 'ready';
			progress = 1;
			const current = restored ?? get(user);
			if (current) {
				ensureSocket();
			}
			setTimeout(() => {
				void goto(current ? '/menu' : '/login');
			}, 320);
		} catch (cause) {
			error = cause instanceof Error ? cause.message : 'Falha ao carregar o jogo';
		}
	}

	function stageState(index: number): 'done' | 'active' | 'idle' {
		const current = stages.indexOf(stage as (typeof stages)[number]);
		if (current === -1) {
			return 'done';
		}
		if (index < current) {
			return 'done';
		}
		if (index === current) {
			return 'active';
		}
		return 'idle';
	}

	function label(key: string) {
		return $i18n.t(`loading.${key}`);
	}
</script>

<div class="loading-screen">
	<div class="loading-card panel">
		<div class="loading-emblem"><CrownIcon size={48} /></div>
		<div>
			<div class="tiny muted" style="letter-spacing: .18em; text-transform: uppercase">
				{$i18n.t('common.appName')}
			</div>
			<h1 style="font-size: 24px; font-weight: 900; margin-top: 4px">
				{error ? $i18n.t('loading.errorTitle') : $i18n.t('loading.title')}
			</h1>
		</div>

		{#if error}
			<div class="error">{error}</div>
			<div class="small muted">{$i18n.t('loading.errorHint')}</div>
			<button class="btn btn-primary" onclick={() => location.reload()} type="button">
				{$i18n.t('common.retry')}
			</button>
		{:else}
			<div class="progress"><span style:width={`${Math.round(progress * 100)}%`}></span></div>
			<div class="loading-stages">
				{#each ['content', 'session', 'sprites', 'audio'] as key, index}
					<div class="loading-stage" class:done={stageState(index) === 'done'} class:active={stageState(index) === 'active'}>
						<span class="dot"></span>
						<span>{label(key)}</span>
						{#if stageState(index) === 'done'}
							<span style="margin-left: auto">✓</span>
						{:else if stageState(index) === 'active'}
							<span class="spinner sm" style="margin-left: auto"></span>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
