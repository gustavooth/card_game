<script lang="ts">
	import { goto } from '$app/navigation';
	import { i18n, currentLocale } from '#lib/i18n';
	import { ApiError } from '#lib/api';
	import { register } from '#lib/stores/session';
	import LanguageToggle from '#lib/components/LanguageToggle.svelte';
	import { audio } from '#lib/audio/synth';
	import CrownIcon from '#lib/components/CrownIcon.svelte';

	let username = $state('');
	let email = $state('');
	let password = $state('');
	let confirmation = $state('');
	let loading = $state(false);
	let error = $state<string | null>(null);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (loading) {
			return;
		}
		error = null;
		if (password !== confirmation) {
			error = $i18n.t('auth.passwordMismatch');
			audio.play('error');
			return;
		}
		loading = true;
		audio.ensure();
		try {
			await register({
				username,
				email,
				password,
				password_confirmation: confirmation,
				locale: currentLocale()
			});
			audio.play('match_start');
			await goto('/menu');
		} catch (cause) {
			audio.play('error');
			if (cause instanceof ApiError) {
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
			<h1>{$i18n.t('auth.registerTitle')}</h1>
			<p>{$i18n.t('auth.registerSubtitle')}</p>
		</div>

		<form onsubmit={submit}>
			<div class="field">
				<label class="label" for="username">{$i18n.t('auth.username')}</label>
				<input
					id="username"
					class="input"
					bind:value={username}
					autocomplete="username"
					minlength="3"
					maxlength="20"
					required
					placeholder="ArenaKing"
				/>
				<div class="tiny muted" style="margin-top: 4px">{$i18n.t('auth.nameHint')}</div>
			</div>
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
					autocomplete="new-password"
					minlength="8"
					required
					placeholder="••••••••"
				/>
				<div class="tiny muted" style="margin-top: 4px">{$i18n.t('auth.passwordHint')}</div>
			</div>
			<div class="field">
				<label class="label" for="confirmation">{$i18n.t('auth.confirmPassword')}</label>
				<input
					id="confirmation"
					class="input"
					type="password"
					bind:value={confirmation}
					autocomplete="new-password"
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
					{$i18n.t('auth.registering')}
				{:else}
					{$i18n.t('auth.createAccount')}
				{/if}
			</button>
		</form>

		<div class="auth-links">
			{$i18n.t('auth.hasAccount')}
			<a href="/login">{$i18n.t('auth.loginInstead')}</a>
		</div>
	</div>
</div>
