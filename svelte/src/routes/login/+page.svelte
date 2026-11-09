<script lang="ts">
	import { goto } from '$app/navigation';
	import { i18n } from '#lib/i18n';
	import { ApiError } from '#lib/api';
	import { login } from '#lib/stores/session';
	import LanguageToggle from '#lib/components/LanguageToggle.svelte';
	import { audio } from '#lib/audio/synth';
	import CrownIcon from '#lib/components/CrownIcon.svelte';

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
<svelte:head>
	<title>{$i18n.t('common.appName')}</title>
</svelte:head>

<div class="auth-page">
	<div class="auth-single">
		<div class="auth-lang"><LanguageToggle /></div>
		<div class="auth-brand">
			<div class="logo-mark"><CrownIcon size={22} /></div>
			<div>
				<div class="auth-brand-name">{$i18n.t('common.appName')}</div>
				<div class="tiny muted">{$i18n.t('common.tagline')}</div>
			</div>
		</div>

		<div class="auth-heading">
			<h1>{$i18n.t('auth.loginTitle')}</h1>
			<p>{$i18n.t('auth.loginSubtitle')}</p>
		</div>

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
</div>
