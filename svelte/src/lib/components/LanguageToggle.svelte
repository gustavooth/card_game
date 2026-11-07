<script lang="ts">
	import { get } from 'svelte/store';
	import { locale, setLocale } from '#lib/i18n';
	import { api } from '#lib/api';
	import { user } from '#lib/stores/session';
	import { audio } from '#lib/audio/synth';
	import type { Locale } from '#lib/types';

	function choose(next: Locale) {
		if (next === $locale) {
			return;
		}
		setLocale(next);
		audio.ensure();
		audio.play('click');
		if (get(user)) {
			api.setLocale(next).catch(() => undefined);
		}
	}
</script>

<div class="row" aria-label="Idioma">
	<button
		class="btn btn-sm"
		class:btn-primary={$locale === 'pt-BR'}
		onclick={() => choose('pt-BR')}
		type="button"
	>
		PT
	</button>
	<button
		class="btn btn-sm"
		class:btn-primary={$locale === 'en'}
		onclick={() => choose('en')}
		type="button"
	>
		EN
	</button>
</div>
