<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { get } from 'svelte/store';
	import { i18n } from '#lib/i18n';
	import { content, loadContent } from '#lib/stores/content';
	import { user, restoreSession } from '#lib/stores/session';
	import {
		match,
		result,
		frames,
		battleError,
		clearBattleError,
		socketConnected,
		deploy,
		joinQueue,
		resetBattle,
		ensureSocket,
		dismissMatch
	} from '#lib/stores/battle';
	import { audio } from '#lib/audio/synth';
	import { GameView } from '#lib/game/gameView';
	import { ensureSprites } from '#lib/game/sprites';
	import AudioToggle from '#lib/components/AudioToggle.svelte';
	import type { EndInfo } from '#lib/types';

	let canvas = $state<HTMLCanvasElement | null>(null);
	let toast = $state<string | null>(null);
	let toastTimer: ReturnType<typeof setTimeout> | null = null;
	let booted = $state(false);

	const localName = $derived($user?.username ?? $i18n.t('battle.you'));

	onMount(() => {
		void boot();
		ensureSocket();
		audio.ensure();
		audio.startMusic('battle');
		return () => {
			if (toastTimer) {
				clearTimeout(toastTimer);
			}
		};
	});

	async function boot() {
		if (!$user) {
			const restored = await restoreSession();
			if (!restored) {
				await goto('/login');
				return;
			}
		}
		if (!$content) {
			try {
				await loadContent();
			} catch {
				await goto('/');
				return;
			}
		}
		if ($content) {
			await ensureSprites($content);
		}
		booted = true;
	}

	$effect(() => {
		const loaded = $content;
		const info = $match;
		if (!canvas || !loaded || !info) {
			return;
		}
		const view = new GameView(
			canvas,
			loaded,
			info,
			get(user)?.username ?? $i18n.t('battle.you'),
			(slot, x, y) => {
				deploy(slot, x, y);
			},
			(code) => {
				showToast(
					code === 'not_enough_elixir'
						? $i18n.t('battle.notEnoughElixir')
						: $i18n.t('battle.invalidPlacement')
				);
			}
		);
		view.start();
		return () => view.stop();
	});

	$effect(() => {
		const error = $battleError;
		if (error) {
			showToast(error.message || translateError(error.code));
			clearBattleError();
		}
	});

	function translateError(code: string): string {
		const key = `errors.${code}`;
		const translated = $i18n.t(key);
		return translated === key ? $i18n.t('common.error') : translated;
	}

	function showToast(message: string) {
		toast = message;
		if (toastTimer) {
			clearTimeout(toastTimer);
		}
		toastTimer = setTimeout(() => {
			toast = null;
		}, 2600);
	}

	function reasonLabel(reason: string): string {
		switch (reason) {
			case 'king_destroyed':
				return $i18n.t('battle.kingDestroyed');
			case 'sudden_death':
				return $i18n.t('battle.suddenDeath');
			case 'regulation_crowns':
				return $i18n.t('battle.regulationCrowns');
			case 'overtime_tiebreak':
				return $i18n.t('battle.overtimeTiebreak');
			default:
				return $i18n.t('battle.draw');
		}
	}

	function crownsFor(info: EndInfo | null): { local: number; enemy: number } {
		if (!info || !$match) {
			return { local: 0, enemy: 0 };
		}
		const index = $match.side;
		return {
			local: info.crowns[index] ?? 0,
			enemy: info.crowns[1 - index] ?? 0
		};
	}

	async function backToMenu() {
		dismissMatch();
		audio.stopMusic();
		await goto('/menu');
	}

	async function playAgain() {
		resetBattle();
		joinQueue();
		await goto('/menu');
	}
</script>

<svelte:head>
	<title>{$i18n.t('menu.battle')} · {$i18n.t('common.appName')}</title>
</svelte:head>

<div class="battle-root">
	<div class="stage-9x16">
		<canvas bind:this={canvas} aria-label={$i18n.t('menu.battle')}></canvas>

		<div class="queue-badge">
			<button class="btn hud-chip back-chip" onclick={backToMenu} type="button" aria-label={$i18n.t('common.back')}>
				←
			</button>
			<AudioToggle />
		</div>

		{#if !$socketConnected}
			<div class="connection-banner">{$i18n.t('battle.connectionLost')}</div>
		{/if}

		{#if $match && $frames.length === 0 && !$result}
			<div class="battle-overlay">
				<div class="overlay-card">
					<div class="center"><span class="spinner"></span></div>
					<strong>{$i18n.t('battle.waitingStart')}</strong>
					<div class="small muted">{$i18n.t('battle.deployHint')}</div>
				</div>
			</div>
		{/if}

		{#if !$match && booted}
			<div class="battle-overlay">
				<div class="overlay-card">
					<h2 style="font-size: 24px; font-weight: 900">{$i18n.t('battle.waitingOpponent')}</h2>
					<p class="small muted">{$i18n.t('battle.deployHint')}</p>
					<button class="btn btn-primary" onclick={backToMenu} type="button">
						{$i18n.t('battle.returnToMenu')}
					</button>
				</div>
			</div>
		{/if}

		{#if $result}
			<div class="battle-overlay">
				<div class="overlay-card">
					<div class="tiny muted" style="letter-spacing: .16em; text-transform: uppercase">
						{$i18n.t('battle.matchEnded')}
					</div>
					<h2
						class="result-title"
						class:win={$result.result === 'win'}
						class:loss={$result.result === 'loss'}
						class:draw={$result.result === 'draw'}
					>
						{$i18n.t(
							$result.result === 'win'
								? 'battle.victory'
								: $result.result === 'loss'
									? 'battle.defeat'
									: 'battle.draw'
						)}
					</h2>
					<div class="crown-row">
						<span>👑 {crownsFor($result).local}</span>
						<span class="muted">×</span>
						<span>👑 {crownsFor($result).enemy}</span>
					</div>
					<div class="small muted">{reasonLabel($result.reason)}</div>
					<div class="small muted">{$i18n.t('battle.replaySaved')}</div>
					<div class="row" style="justify-content: center; flex-wrap: wrap">
						<button class="btn" onclick={backToMenu} type="button">
							{$i18n.t('battle.returnToMenu')}
						</button>
						<button class="btn btn-primary" onclick={playAgain} type="button">
							{$i18n.t('battle.playAgain')}
						</button>
					</div>
				</div>
			</div>
		{/if}

		{#if toast}
			<div class="toast error">{toast}</div>
		{/if}
	</div>
</div>
