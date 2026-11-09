<script lang="ts">
	import { i18n } from '#lib/i18n';

	type Screen = 'home' | 'cards' | 'chat' | 'profile';

	interface Props {
		tab: Screen;
		onchange: (tab: Screen) => void;
	}

	let { tab, onchange }: Props = $props();

	const items: { id: Screen; label: string; icon: string }[] = [
		{ id: 'cards', label: 'menu.cards', icon: 'cards' },
		{ id: 'home', label: 'menu.battle', icon: 'battle' },
		{ id: 'chat', label: 'menu.chat', icon: 'chat' },
		{ id: 'profile', label: 'menu.profile', icon: 'profile' }
	];
</script>

<nav class="bottom-nav" aria-label="Navegação principal">
	{#each items as item}
		<button
			type="button"
			class="nav-item"
			class:active={tab === item.id}
			class:center={item.id === 'home'}
			onclick={() => onchange(item.id)}
			aria-label={$i18n.t(item.label)}
			aria-current={tab === item.id ? 'page' : undefined}
		>
			<span class="nav-icon">
				{#if item.icon === 'cards'}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<rect x="4" y="5" width="11" height="15" rx="2" fill="currentColor" opacity=".45" />
						<rect x="9" y="3" width="11" height="16" rx="2" fill="currentColor" />
						<path d="M12 7h5M12 10h5M12 13h3" stroke="#0b1220" stroke-width="1.4" stroke-linecap="round" />
					</svg>
				{:else if item.icon === 'battle'}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M4 4l7 7-2 2-7-7V4h2Z" fill="currentColor" />
						<path d="M20 4l-7 7 2 2 7-7V4h-2Z" fill="currentColor" opacity=".8" />
						<path d="M5 20l6-6M19 20l-6-6" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" />
						<path d="M8 17l-2 2M16 17l2 2" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" />
					</svg>
				{:else if item.icon === 'chat'}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M4 5h16v10H9l-5 4V5Z" fill="currentColor" />
						<path d="M8 9h8M8 12h5" stroke="#0b1220" stroke-width="1.5" stroke-linecap="round" />
					</svg>
				{:else}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<circle cx="12" cy="8" r="4" fill="currentColor" />
						<path d="M4 21c0-4 3.6-7 8-7s8 3 8 7H4Z" fill="currentColor" opacity=".8" />
					</svg>
				{/if}
			</span>
			<span class="nav-label">{$i18n.t(item.label)}</span>
		</button>
	{/each}
</nav>
