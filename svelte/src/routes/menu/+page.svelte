<script lang="ts">
	import { get } from 'svelte/store';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { i18n } from '#lib/i18n';
	import { api, ApiError } from '#lib/api';
	import { user, deck, saveDeck, logout, restoreSession } from '#lib/stores/session';
	import { content, loadContent } from '#lib/stores/content';
	import {
		match,
		result,
		queuePosition,
		joinQueue,
		leaveQueue,
		battleError,
		clearBattleError,
		ensureSocket,
		resetBattle
	} from '#lib/stores/battle';
	import { gameSocket } from '#lib/net/socket';
	import { chatMessages, chatConnected } from '#lib/stores/chat';
	import { sendChat } from '#lib/stores/battle';
	import { audio } from '#lib/audio/synth';
	import { ensureSprites } from '#lib/game/sprites';
	import type { CardDef, PlayerStats, RecentMatch, ReplayMeta, User } from '#lib/types';
	import CardArt from '#lib/components/CardArt.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import LanguageToggle from '#lib/components/LanguageToggle.svelte';
	import AudioToggle from '#lib/components/AudioToggle.svelte';

	type Tab = 'battle' | 'cards' | 'chat';
	type DeckSaveState = 'idle' | 'saving' | 'saved';

	let tab = $state<Tab>('battle');
	let selected = $state<CardDef | null>(null);
	let profileOpen = $state(false);
	let profile = $state<{ user: User; stats: PlayerStats; recent: RecentMatch[] } | null>(null);
	let replays = $state<ReplayMeta[]>([]);
	let profileLoading = $state(false);
	let profileError = $state<string | null>(null);
	let search = $state('');
	let typeFilter = $state('all');
	let rarityFilter = $state('all');
	let saveState = $state<DeckSaveState>('idle');
	let toast = $state<{ text: string; kind: 'info' | 'error' | 'success' } | null>(null);
	let chatText = $state('');
	let cooldownUntil = $state(0);
	let clock = $state(Date.now());
	let wasQueued = false;
	let chatList: HTMLDivElement | undefined = $state();
	let saveTimer: ReturnType<typeof setTimeout> | null = null;
	let toastTimer: ReturnType<typeof setTimeout> | null = null;
	let cooldownTimer: ReturnType<typeof setInterval> | null = null;

	const rarityColors: Record<string, string> = {
		common: '#94a3b8',
		rare: '#f59e0b',
		epic: '#a855f7',
		legendary: '#22d3ee'
	};

	const cards = $derived(
		$content
			? [...$content.definitions.cards].sort((a, b) => a.sort - b.sort)
			: []
	);

	const filteredCards = $derived(
		cards.filter((card) => {
			const text = search.trim().toLowerCase();
			const label = $i18n.card(card.slug).name.toLowerCase();
			if (text && !card.slug.includes(text) && !label.includes(text)) {
				return false;
			}
			if (typeFilter !== 'all' && card.type.kind !== typeFilter) {
				return false;
			}
			if (rarityFilter !== 'all' && card.rarity !== rarityFilter) {
				return false;
			}
			return true;
		})
	);

	const chatRemaining = $derived(Math.max(0, Math.ceil((cooldownUntil - clock) / 1000)));
	const deckFull = $derived($deck.length >= 8);
	const canBattle = $derived($deck.length === 8);
	const activeMatch = $derived(!!$match && !$result);
	const localName = $derived($user?.username ?? '');

	onMount(() => {
		void ensureAuthenticated();
		ensureSocket();
		audio.ensure();
		audio.startMusic('menu');
		cooldownTimer = setInterval(() => {
			clock = Date.now();
		}, 500);
		return () => {
			if (saveTimer) {
				clearTimeout(saveTimer);
			}
			if (toastTimer) {
				clearTimeout(toastTimer);
			}
			if (cooldownTimer) {
				clearInterval(cooldownTimer);
			}
		};
	});

	$effect(() => {
		const position = $queuePosition;
		const current = $match;
		const ended = $result;
		if (position !== null) {
			wasQueued = true;
		}
		if (wasQueued && current && !ended) {
			wasQueued = false;
			void goto('/battle');
		}
	});

	$effect(() => {
		const error = $battleError;
		if (error) {
			showToast(error.message || translateError(error.code), 'error');
			audio.play('error');
			clearBattleError();
		}
	});

	$effect(() => {
		const list = $chatMessages;
		if (chatList && list.length >= 0) {
			requestAnimationFrame(() => {
				if (chatList) {
					chatList.scrollTop = chatList.scrollHeight;
				}
			});
		}
	});

	async function ensureAuthenticated() {
		try {
			const loaded = $content ?? (await loadContent());
			await ensureSprites(loaded);
		} catch {
			await goto('/');
			return;
		}
		if (!$user) {
			const restored = await restoreSession();
			if (!restored) {
				await goto('/login');
			}
		}
	}

	function translateError(code: string): string {
		const key = `errors.${code}`;
		const translated = $i18n.t(key);
		return translated === key ? $i18n.t('common.error') : translated;
	}

	function showToast(text: string, kind: 'info' | 'error' | 'success' = 'info') {
		toast = { text, kind };
		if (toastTimer) {
			clearTimeout(toastTimer);
		}
		toastTimer = setTimeout(() => {
			toast = null;
		}, 3200);
	}

	function switchTab(next: Tab) {
		tab = next;
		audio.play('click');
	}

	function inDeck(card: CardDef): boolean {
		return $deck.includes(card.slug);
	}

	function toggleCard(card: CardDef) {
		const current = get(deck);
		let next: string[];
		if (current.includes(card.slug)) {
			next = current.filter((slug) => slug !== card.slug);
		} else {
			if (current.length >= 8) {
				showToast($i18n.t('menu.deckFull'), 'error');
				audio.play('error');
				return;
			}
			next = [...current, card.slug];
		}
		deck.set(next);
		audio.play('click');
		scheduleSave(next);
	}

	function removeFromDeck(slug: string) {
		const next = get(deck).filter((item) => item !== slug);
		deck.set(next);
		audio.play('click');
		scheduleSave(next);
	}

	function scheduleSave(next: string[]) {
		if (!get(user)) {
			return;
		}
		if (saveTimer) {
			clearTimeout(saveTimer);
		}
		saveState = 'saving';
		saveTimer = setTimeout(async () => {
			try {
				await saveDeck(next);
				saveState = 'saved';
				setTimeout(() => {
					if (saveState === 'saved') {
						saveState = 'idle';
					}
				}, 1400);
			} catch (cause) {
				saveState = 'idle';
				showToast(cause instanceof ApiError ? cause.message : $i18n.t('errors.network'), 'error');
			}
		}, 420);
	}

	function startBattle() {
		if (!canBattle) {
			showToast($i18n.t('menu.deckNeedsEight'), 'error');
			return;
		}
		audio.ensure();
		audio.play('queue');
		audio.startMusic('battle');
		joinQueue();
	}

	function cancelQueue() {
		leaveQueue();
		audio.play('click');
		audio.startMusic('menu');
	}

	async function openProfile() {
		profileOpen = true;
		profileLoading = true;
		profileError = null;
		try {
			const [data, replayData] = await Promise.all([api.profile(), api.replays()]);
			profile = data;
			replays = replayData.replays;
		} catch (cause) {
			profileError = cause instanceof ApiError ? cause.message : $i18n.t('errors.network');
		} finally {
			profileLoading = false;
		}
	}

	async function doLogout() {
		if (!window.confirm($i18n.t('menu.logoutConfirm'))) {
			return;
		}
		audio.play('click');
		audio.stopMusic();
		await logout();
		gameSocket.disconnect();
		resetBattle();
		await goto('/login');
	}

	function openCard(card: CardDef) {
		selected = card;
		audio.play('click');
	}

	function closeCard() {
		selected = null;
	}

	function formatDate(value: string): string {
		try {
			return new Intl.DateTimeFormat($i18n.locale, {
				day: '2-digit',
				month: 'short',
				hour: '2-digit',
				minute: '2-digit'
			}).format(new Date(value));
		} catch {
			return value;
		}
	}

	function formatClock(value: number): string {
		try {
			return new Intl.DateTimeFormat($i18n.locale, {
				hour: '2-digit',
				minute: '2-digit'
			}).format(new Date(value));
		} catch {
			return '';
		}
	}

	function sendMessage() {
		const text = chatText.trim();
		if (!text || chatRemaining > 0) {
			return;
		}
		if (!sendChat(text)) {
			showToast($i18n.t('menu.offlineHint'), 'error');
			return;
		}
		chatText = '';
		cooldownUntil = Date.now() + 2000;
		audio.play('click');
	}

	function cardStats(card: CardDef): { label: string; value: string }[] {
		if (!$content) {
			return [];
		}
		if (card.type.kind === 'troop') {
			const troop = $content.definitions.troops[card.type.troop];
			if (!troop) {
				return [];
			}
			return [
				{ label: $i18n.t('menu.hitpoints'), value: String(troop.health) },
				{ label: $i18n.t('menu.damage'), value: String(troop.attack.damage) },
				{ label: $i18n.t('menu.speed'), value: troop.speed.toFixed(1) },
				{ label: $i18n.t('menu.range'), value: troop.attack.range.toFixed(1) },
				{ label: $i18n.t('menu.elixir'), value: String(card.elixir_cost) },
				{ label: $i18n.t('menu.troop'), value: `${card.type.count}x` }
			];
		}
		if (card.type.kind === 'building') {
			const building = $content.definitions.buildings[card.type.building];
			if (!building) {
				return [];
			}
			return [
				{ label: $i18n.t('menu.hitpoints'), value: String(building.health) },
				{ label: $i18n.t('menu.damage'), value: String(building.attack?.damage ?? 0) },
				{ label: $i18n.t('menu.range'), value: (building.attack?.range ?? 0).toFixed(1) },
				{ label: $i18n.t('menu.elixir'), value: String(card.elixir_cost) },
				{
					label: $i18n.t('menu.building'),
					value: `${Math.round((building.lifetime_ms ?? 0) / 1000)}s`
				}
			];
		}
		const spell = $content.definitions.spells[card.type.spell];
		if (!spell) {
			return [];
		}
		return [
			{ label: $i18n.t('menu.damage'), value: String(spell.damage) },
			{ label: $i18n.t('menu.range'), value: spell.radius.toFixed(1) },
			{ label: $i18n.t('menu.elixir'), value: String(card.elixir_cost) },
			{ label: $i18n.t('menu.spell'), value: `${(spell.delay_ms / 1000).toFixed(1)}s` }
		];
	}
</script>

<div class="menu-shell">
	<header class="topbar">
		<div class="topbar-inner">
			<button
				class="logo"
				style="border: 0; background: transparent; padding: 0; color: inherit"
				onclick={() => switchTab('battle')}
				type="button"
			>
				<div class="logo-mark">R</div>
				<div style="text-align: left">
					<div style="font-size: 15px; font-weight: 900">{$i18n.t('common.appName')}</div>
					<div class="tiny muted">1v1 · 30 Hz</div>
				</div>
			</button>

			<div class="grow"></div>

			<div class="user-pill">
				<div class="avatar">{localName.slice(0, 1).toUpperCase()}</div>
				<div>
					<div style="font-size: 12px; font-weight: 800">{localName}</div>
					<div class="tiny muted user-email">{$user?.email ?? ''}</div>
				</div>
			</div>
			<button class="btn btn-sm" onclick={openProfile} type="button">
				{$i18n.t('menu.profile')}
			</button>
			<AudioToggle />
			<LanguageToggle />
			<button class="btn btn-sm btn-ghost" onclick={doLogout} type="button">
				{$i18n.t('auth.logout')}
			</button>
		</div>
	</header>

	<main class="menu-body">
		<div class="tabs">
			<button class="tab" class:active={tab === 'cards'} onclick={() => switchTab('cards')} type="button">
				{$i18n.t('menu.cards')} · 50
			</button>
			<button class="tab" class:active={tab === 'battle'} onclick={() => switchTab('battle')} type="button">
				{$i18n.t('menu.battle')}
			</button>
			<button class="tab" class:active={tab === 'chat'} onclick={() => switchTab('chat')} type="button">
				{$i18n.t('menu.chat')}
			</button>
		</div>

		{#if tab === 'battle'}
			<section class="battle-layout">
				<div class="battle-hero">
					<div class="arena-badge">
						<span class="chip">🏰 Arena clássica</span>
						<span class="chip">18 × 32</span>
					</div>
					<div class="battle-hero-content">
						<div class="tiny muted" style="letter-spacing: .18em; text-transform: uppercase">
							1v1 · 3:00 + 2:00
						</div>
						<h1 class="battle-title">{$i18n.t('menu.battle')}</h1>
						<p class="small muted" style="max-width: 460px">
							{$i18n.t('menu.deckHint')}
						</p>

						{#if $queuePosition !== null}
							<div class="panel" style="padding: 14px; display: grid; gap: 10px; max-width: 420px">
								<div class="row">
									<span class="spinner sm"></span>
									<strong>{$i18n.t('menu.searching')}</strong>
								</div>
								<div class="small muted">
									{$i18n.t('menu.queuePosition', { position: $queuePosition ?? 1 })}
								</div>
								<button class="btn btn-danger" onclick={cancelQueue} type="button">
									{$i18n.t('menu.cancelQueue')}
								</button>
							</div>
						{:else if activeMatch}
							<div class="panel" style="padding: 14px; display: grid; gap: 10px; max-width: 420px">
								<strong>{$i18n.t('battle.inProgress')}</strong>
								<div class="small muted">
									{$i18n.t('battle.opponent')}: {$match?.opponent}
								</div>
								<button class="btn btn-primary" onclick={() => goto('/battle')} type="button">
									{$i18n.t('battle.reconnect')}
								</button>
							</div>
						{:else}
							<div class="row wrap">
								<button
									class="btn btn-primary btn-lg"
									onclick={startBattle}
									disabled={!canBattle}
									type="button"
								>
									⚔ {$i18n.t('menu.battleButton')}
								</button>
								<span class="chip">{$deck.length}/8 {$i18n.t('menu.cards').toLowerCase()}</span>
								{#if !canBattle}
									<span class="small error">{$i18n.t('menu.deckNeedsEight')}</span>
								{/if}
							</div>
						{/if}
					</div>
				</div>

				<div class="panel menu-panel deck-card">
					<div class="spread">
						<div>
							<div class="tiny muted" style="text-transform: uppercase; letter-spacing: .08em">
								{$i18n.t('menu.deck')}
							</div>
							<div style="font-size: 18px; font-weight: 900">{$deck.length}/8</div>
						</div>
						<button class="btn btn-sm" onclick={() => switchTab('cards')} type="button">
							{$i18n.t('menu.cards')}
						</button>
					</div>
					<div class="deck-grid">
						{#each Array(8) as _, index}
							{#if $deck[index]}
								{@const card = $content?.definitions.cards.find((item) => item.slug === $deck[index])}
								<div class="deck-slot filled" title={card?.slug}>
									{#if card}
										<CardArt {card} size={100} />
										<span class="cost-orb slot-cost">{card.elixir_cost}</span>
									{/if}
								</div>
							{:else}
								<div class="deck-slot"><span class="muted">+</span></div>
							{/if}
						{/each}
					</div>
					<div class="small muted">{$i18n.t('menu.chooseDeck')}</div>
				</div>
			</section>
		{:else if tab === 'cards'}
			<section class="panel menu-panel" style="display: grid; gap: 14px">
				<div class="spread wrap">
					<div>
						<h2 style="font-size: 22px; font-weight: 900">{$i18n.t('menu.cards')}</h2>
						<div class="small muted">{$i18n.t('menu.cardsGridHint')}</div>
					</div>
					<div class="row">
						<span class="chip">{$deck.length}/8</span>
						{#if saveState === 'saving'}
							<span class="chip">{$i18n.t('menu.saving')}</span>
						{:else if saveState === 'saved'}
							<span class="chip success">✓ {$i18n.t('menu.deckSaved')}</span>
						{/if}
					</div>
				</div>

				<div class="card-filters">
					<input
						class="input"
						bind:value={search}
						placeholder={$i18n.t('menu.searchCards')}
						aria-label={$i18n.t('menu.searchCards')}
					/>
					<select class="select" bind:value={typeFilter} aria-label={$i18n.t('menu.filterType')}>
						<option value="all">{$i18n.t('common.all')}</option>
						<option value="troop">{$i18n.t('menu.troop')}</option>
						<option value="spell">{$i18n.t('menu.spell')}</option>
						<option value="building">{$i18n.t('menu.building')}</option>
					</select>
					<select class="select" bind:value={rarityFilter} aria-label={$i18n.t('menu.filterRarity')}>
						<option value="all">{$i18n.t('common.all')}</option>
						<option value="common">{$i18n.t('rarity.common')}</option>
						<option value="rare">{$i18n.t('rarity.rare')}</option>
						<option value="epic">{$i18n.t('rarity.epic')}</option>
						<option value="legendary">{$i18n.t('rarity.legendary')}</option>
					</select>
				</div>

				{#if filteredCards.length === 0}
					<div class="muted center" style="padding: 40px">{$i18n.t('menu.noCards')}</div>
				{:else}
					<div class="card-grid">
						{#each filteredCards as card (card.slug)}
							<button
								class="card-tile"
								class:in-deck={inDeck(card)}
								onclick={() => openCard(card)}
								type="button"
							>
								<div class="card-tile-art">
									<span
										class="rarity-line"
										style:background={rarityColors[card.rarity] ?? '#94a3b8'}
									></span>
									<span class="cost-orb">{card.elixir_cost}</span>
									<CardArt {card} size={120} />
								</div>
								<div class="card-tile-body">
									<div class="card-name">{$i18n.card(card.slug).name}</div>
									<div class="card-meta">
										<span>{$i18n.t(`menu.${card.type.kind}`)}</span>
										<span>{$i18n.t(`rarity.${card.rarity}`)}</span>
									</div>
								</div>
							</button>
						{/each}
					</div>
				{/if}

				<div class="deck-bar">
					<div class="deck-bar-grid">
						{#each Array(8) as _, index}
							{@const slug = $deck[index]}
							{#if slug}
								{@const card = $content?.definitions.cards.find((item) => item.slug === slug)}
								<button
									class="deck-mini"
									onclick={() => removeFromDeck(slug)}
									title={card ? $i18n.card(card.slug).name : slug}
									type="button"
								>
									{#if card}
										<CardArt {card} size={90} />
									{/if}
									<span class="remove" aria-hidden="true">×</span>
								</button>
							{:else}
								<div class="deck-mini empty">+</div>
							{/if}
						{/each}
					</div>
					<div style="min-width: 100px; text-align: right">
						<div class="tiny muted">{$i18n.t('menu.deck')}</div>
						<div style="font-size: 18px; font-weight: 900">{$deck.length}/8</div>
					</div>
				</div>
			</section>
		{:else}
			<section class="panel menu-panel chat-layout">
				<div class="spread wrap">
					<div>
						<h2 style="font-size: 22px; font-weight: 900">{$i18n.t('menu.chat')}</h2>
						<div class="small muted">{$i18n.t('menu.chatHint')}</div>
					</div>
					<span class="chip" class:success={$chatConnected}>
						<span
							class="dot"
							style:background={$chatConnected ? '#22c55e' : '#64748b'}
						></span>
						{$chatConnected ? $i18n.t('common.online') : $i18n.t('common.offline')}
					</span>
				</div>

				<div class="chat-list" bind:this={chatList}>
					{#if $chatMessages.length === 0}
						<div class="muted center" style="padding: 30px">{$i18n.t('menu.noMessages')}</div>
					{/if}
					{#each $chatMessages as message (message.ts + message.user)}
						<div class="chat-line" class:own={message.user === localName}>
							<div class="spread">
								<span class="chat-author">{message.user}</span>
								<span class="tiny muted">{formatClock(message.ts)}</span>
							</div>
							<div class="chat-text">{message.text}</div>
						</div>
					{/each}
				</div>

				<div class="chat-input-row">
					<input
						class="input"
						bind:value={chatText}
						maxlength="200"
						placeholder={$i18n.t('menu.chatPlaceholder')}
						onkeydown={(event) => {
							if (event.key === 'Enter') {
								sendMessage();
							}
						}}
					/>
					<button
						class="btn btn-primary"
						onclick={sendMessage}
						disabled={!$chatConnected || chatRemaining > 0 || chatText.trim().length === 0}
						type="button"
					>
						{#if chatRemaining > 0}
							{chatRemaining}s
						{:else}
							{$i18n.t('menu.send')}
						{/if}
					</button>
				</div>
			</section>
		{/if}
	</main>

	{#if selected}
		<Modal title={$i18n.t('menu.cardDetails')} onclose={closeCard}>
			<div class="card-detail-head">
				<div class="card-detail-art">
					<CardArt card={selected} size={150} />
				</div>
				<div>
					<div class="row wrap" style="margin-bottom: 6px">
						<span class="badge">{selected.elixir_cost}</span>
						<span class="chip">{$i18n.t(`rarity.${selected.rarity}`)}</span>
						<span class="chip">{$i18n.t(`menu.${selected.type.kind}`)}</span>
					</div>
					<h3 style="font-size: 22px; font-weight: 900">{$i18n.card(selected.slug).name}</h3>
					<p class="small muted" style="margin-top: 6px; line-height: 1.45">
						{$i18n.card(selected.slug).description}
					</p>
				</div>
			</div>

			<div class="card-detail-stats" style="margin-top: 14px">
				{#each cardStats(selected) as stat}
					<div class="detail-stat">
						<b>{stat.value}</b>
						<span>{stat.label}</span>
					</div>
				{/each}
			</div>

			<div class="row" style="margin-top: 16px">
				{#if inDeck(selected)}
					<button class="btn btn-danger grow" onclick={() => toggleCard(selected!)} type="button">
						{$i18n.t('menu.removeFromDeck')}
					</button>
				{:else}
					<button
						class="btn btn-primary grow"
						onclick={() => toggleCard(selected!)}
						disabled={deckFull}
						type="button"
					>
						{deckFull ? $i18n.t('menu.deckFull') : $i18n.t('menu.addToDeck')}
					</button>
				{/if}
				<button class="btn" onclick={closeCard} type="button">{$i18n.t('common.close')}</button>
			</div>
		</Modal>
	{/if}

	{#if profileOpen}
		<Modal title={$i18n.t('menu.profile')} onclose={() => (profileOpen = false)} wide>
			{#if profileLoading}
				<div class="center" style="padding: 30px"><span class="spinner"></span></div>
			{:else if profileError}
				<div class="error">{profileError}</div>
			{:else if profile}
				<div class="row wrap" style="margin-bottom: 14px">
					<div class="avatar" style="width: 46px; height: 46px; font-size: 20px">
						{profile.user.username.slice(0, 1).toUpperCase()}
					</div>
					<div>
						<div style="font-size: 18px; font-weight: 900">{profile.user.username}</div>
						<div class="tiny muted">{profile.user.email}</div>
					</div>
				</div>

				<div class="stats-grid" style="margin-bottom: 16px">
					<div class="stat-card">
						<div class="stat-value">{profile.stats.games}</div>
						<div class="stat-label">{$i18n.t('menu.games')}</div>
					</div>
					<div class="stat-card">
						<div class="stat-value" style="color: #86efac">{profile.stats.wins}</div>
						<div class="stat-label">{$i18n.t('menu.wins')}</div>
					</div>
					<div class="stat-card">
						<div class="stat-value" style="color: #fca5a5">{profile.stats.losses}</div>
						<div class="stat-label">{$i18n.t('menu.losses')}</div>
					</div>
					<div class="stat-card">
						<div class="stat-value" style="color: #fcd34d">{profile.stats.draws}</div>
						<div class="stat-label">{$i18n.t('menu.draws')}</div>
					</div>
				</div>

				<h3 style="font-size: 15px; font-weight: 900; margin-bottom: 8px">
					{$i18n.t('menu.recentMatches')}
				</h3>
				{#if profile.recent.length === 0}
					<div class="small muted">{$i18n.t('menu.noMatches')}</div>
				{:else}
					<div class="recent-list">
						{#each profile.recent as recent}
							<div class="recent-row">
								<span>
									vs <strong>{recent.opponent}</strong>
									<span class="muted">· {formatDate(recent.ended_at)}</span>
								</span>
								<span class={`result-${recent.result}`}>
									{recent.crowns} : {recent.opponent_crowns}
									· {$i18n.t(`menu.${recent.result === 'win' ? 'wins' : recent.result === 'loss' ? 'losses' : 'draws'}`)}
								</span>
							</div>
						{/each}
					</div>
				{/if}

				<h3 style="font-size: 15px; font-weight: 900; margin: 16px 0 8px">
					{$i18n.t('menu.replays')}
				</h3>
				{#if replays.length === 0}
					<div class="small muted">{$i18n.t('menu.noReplays')}</div>
				{:else}
					<div class="replay-list">
						{#each replays as replay}
							<div class="replay-row">
								<span>{formatDate(replay.created_at)}</span>
								<a class="btn btn-sm" href={`/replay/${replay.id}`}>
									{$i18n.t('menu.watchReplay')}
								</a>
							</div>
						{/each}
					</div>
				{/if}
			{/if}
		</Modal>
	{/if}

	{#if toast}
		<div class="toast" class:error={toast.kind === 'error'} class:success={toast.kind === 'success'}>
			{toast.text}
		</div>
	{/if}
</div>
