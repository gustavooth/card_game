<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { i18n } from '#lib/i18n';
	import { api, ApiError } from '#lib/api';
	import { content, loadContent } from '#lib/stores/content';
	import { user, restoreSession } from '#lib/stores/session';
	import { parseReplay, ReplayView, type ReplayData } from '#lib/game/replay';
	import { audio } from '#lib/audio/synth';
	import { ensureSprites } from '#lib/game/sprites';
	
	let canvas = $state<HTMLCanvasElement | null>(null);
	let data = $state<ReplayData | null>(null);
	let view = $state<ReplayView | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let time = $state(0);
	let duration = $state(0);
	let playing = $state(false);
	let speed = $state(1);
	let seeking = $state(false);

	onMount(() => {
		void boot();
		audio.stopMusic();
		return () => {
			view?.stop();
		};
	});

	async function boot() {
		loading = true;
		error = null;
		try {
			if (!$user) {
				const restored = await restoreSession();
				if (!restored) {
					await goto('/login');
					return;
				}
			}
			if (!$content) {
				await loadContent();
			}
			await ensureSprites($content ?? (await loadContent()));
			const id = page.params.id;
			if (!id) {
				throw new Error('Replay inválido');
			}
			const buffer = await api.replayData(id);
			data = await parseReplay(buffer);
			if (data.frames.length === 0) {
				throw new Error($i18n.t('replay.noFrames'));
			}
			duration = (data.frames[data.frames.length - 1]?.m ?? 0) / 1000;
		} catch (cause) {
			error =
				cause instanceof ApiError
					? cause.message
					: cause instanceof Error
						? cause.message
						: $i18n.t('replay.error');
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		const loaded = $content;
		const replay = data;
		if (!canvas || !loaded || !replay) {
			return;
		}
		const instance = new ReplayView(canvas, loaded, replay, (current, total, isPlaying) => {
			time = current;
			duration = total;
			playing = isPlaying;
		});
		view = instance;
		instance.start();
		return () => {
			instance.stop();
			view = null;
		};
	});

	function toggle() {
		audio.ensure();
		view?.toggle();
		playing = view?.isPlaying ?? false;
		audio.play('click');
	}

	function setSpeed(next: number) {
		speed = next;
		view?.setSpeed(next);
		audio.play('click');
	}

	function seek(event: Event) {
		const value = Number((event.currentTarget as HTMLInputElement).value);
		time = value;
		view?.seek(value);
	}

	function formatTime(value: number): string {
		const total = Math.max(0, Math.floor(value));
		const minutes = Math.floor(total / 60);
		const seconds = total % 60;
		return `${minutes}:${seconds.toString().padStart(2, '0')}`;
	}

	function dateLabel(value: number): string {
		if (!value) {
			return '';
		}
		try {
			return new Intl.DateTimeFormat($i18n.locale, {
				day: '2-digit',
				month: 'short',
				hour: '2-digit',
				minute: '2-digit'
			}).format(new Date(value * 1000));
		} catch {
			return '';
		}
	}
</script>

<svelte:head>
	<title>{$i18n.t('common.appName')}</title>
</svelte:head>

<div class="replay-root">
	<div class="spread" style="width: min(620px, 100%)">
		<button class="btn btn-sm" onclick={() => goto('/menu')} type="button">
			← {$i18n.t('replay.backToMenu')}
		</button>
		<div class="small muted">
			{#if data}
				{$i18n.t('battle.opponent')}: {data.meta.opponent} · {dateLabel(data.meta.started_at)}
			{:else}
				{$i18n.t('replay.title')}
			{/if}
		</div>
	</div>

	<div class="replay-stage">
		<canvas bind:this={canvas} aria-label={$i18n.t('replay.title')}></canvas>

		{#if loading}
			<div class="battle-overlay">
				<div class="overlay-card">
					<div class="center"><span class="spinner"></span></div>
					<strong>{$i18n.t('replay.loading')}</strong>
				</div>
			</div>
		{:else if error}
			<div class="battle-overlay">
				<div class="overlay-card">
					<h2 style="font-size: 22px; font-weight: 900">{$i18n.t('replay.error')}</h2>
					<div class="small error">{error}</div>
					<button class="btn btn-primary" onclick={() => goto('/menu')} type="button">
						{$i18n.t('replay.backToMenu')}
					</button>
				</div>
			</div>
		{/if}
	</div>

	{#if data}
		<div class="replay-controls panel">
			<div class="spread">
				<button class="btn btn-primary" onclick={toggle} type="button">
					{playing ? `⏸ ${$i18n.t('replay.pause')}` : `▶ ${$i18n.t('replay.play')}`}
				</button>
				<div class="small muted">
					{formatTime(time)} / {formatTime(duration)}
				</div>
				<div class="speed-row">
					{#each [0.5, 1, 2] as option}
						<button
							class="btn btn-sm"
							class:active={speed === option}
							onclick={() => setSpeed(option)}
							type="button"
						>
							{option}x
						</button>
					{/each}
				</div>
			</div>
			<input
				class="range"
				type="range"
				min="0"
				max={Math.max(0.01, duration)}
				step="0.05"
				value={time}
				oninput={seek}
				onpointerdown={() => (seeking = true)}
				onpointerup={() => (seeking = false)}
				aria-label={$i18n.t('replay.title')}
			/>
			<div class="row wrap" style="justify-content: space-between">
				<span class="chip">
					{data.meta.side === 0 ? data.meta.blue : data.meta.red} vs {data.meta.opponent}
				</span>
				{#if seeking}
					<span class="chip">{$i18n.t('replay.speed')} {speed}x</span>
				{/if}
			</div>
		</div>
	{/if}
</div>
