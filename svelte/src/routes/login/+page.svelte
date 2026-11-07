<script lang="ts">
	import { goto } from '$app/navigation';
	import { i18n } from '#lib/i18n';
	import { ApiError } from '#lib/api';
	import { login } from '#lib/stores/session';
	import LanguageToggle from '#lib/components/LanguageToggle.svelte';
	import { audio } from '#lib/audio/synth';

	let email = $state('');
	let password = $state('');
	let loading = $state(false);
	let error = $state<string | null>(null);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (loading) {
			return;
		}
		error = null;
		loading = true;
		audio.ensure();
		try {
			await login(email, password);
			audio.play('match_start');
			await goto('/menu');
		} catch (cause) {
			audio.play('error');
			if (cause instanceof ApiError && cause.code === 'unauthorized') {
				error = $i18n.t('auth.invalidCredentials');
			} else if (cause instanceof ApiError) {
				error = cause.message;
			} else {
				error = $i18n.t('errors.network');
			}
		} finally {
			loading = false;
		}
	}
</script>

<div class="auth-screen page">
	<section class="auth-hero">
		<div class="logo">
			<div class="logo-mark">R</div>
			<div>
				<div style="font-size: 18px; font-weight: 900">{$i18n.t('common.appName')}</div>
				<div class="tiny muted">9:16 · 30 Hz · autoritativo</div>
			</div>
		</div>
		<div class="auth-hero-content">
			<h1 class="auth-title">{$i18n.t('auth.loginTitle')}</h1>
			<p class="auth-sub">{$i18n.t('auth.loginSubtitle')}</p>
			<div class="auth-features">
				<span class="chip">🃏 {$i18n.t('menu.cards')}</span>
				<span class="chip">⚔ {$i18n.t('menu.battle')}</span>
				<span class="chip">💬 {$i18n.t('menu.chat')}</span>
			</div>
		</div>
		<div class="small muted">{$i18n.t('common.appName')} · 2026</div>
	</section>

	<section class="auth-panel">
		<div class="lang-row"><LanguageToggle /></div>
		<div class="auth-card panel">
			<h2 style="font-size: 24px; font-weight: 900">{$i18n.t('auth.login')}</h2>
			<p class="small muted" style="margin: 6px 0 18px">{$i18n.t('auth.loginSubtitle')}</p>

			<form onsubmit={submit}>
				<div class="field">
					<label class="label" for="email">{$i18n.t('auth.email')}</label>
					<input
						id="email"
						class="input"
						type="email"
						bind:value={email}
						autocomplete="email"
						required
						placeholder="voce@exemplo.com"
					/>
				</div>
				<div class="field">
					<label class="label" for="password">{$i18n.t('auth.password')}</label>
					<input
						id="password"
						class="input"
						type="password"
						bind:value={password}
						autocomplete="current-password"
						required
						placeholder="••••••••"
					/>
				</div>
				{#if error}
					<div class="error" style="margin-bottom: 10px">{error}</div>
				{/if}
				<button class="btn btn-primary btn-lg" style="width: 100%" type="submit" disabled={loading}>
					{#if loading}
						<span class="spinner sm"></span>
						{$i18n.t('auth.loggingIn')}
					{:else}
						{$i18n.t('auth.login')}
					{/if}
				</button>
			</form>

			<div class="auth-links">
				{$i18n.t('auth.noAccount')}
				<a href="/register">{$i18n.t('auth.createOne')}</a>
			</div>
		</div>
	</section>
</div>
