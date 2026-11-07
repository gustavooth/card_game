import { get, writable } from 'svelte/store';
import { api, ApiError } from '#lib/api';
import type {
	BuildingDef,
	CardDef,
	Content,
	SpellDef,
	TroopDef,
	TowerDef
} from '#lib/types';

export const content = writable<Content | null>(null);
export const contentError = writable<string | null>(null);

export async function loadContent(force = false): Promise<Content> {
	const existing = get(content);
	if (existing && !force) {
		return existing;
	}
	try {
		const loaded = await api.content();
		content.set(loaded);
		contentError.set(null);
		return loaded;
	} catch (error) {
		const message = error instanceof ApiError ? error.message : 'Falha ao carregar conteúdo';
		contentError.set(message);
		throw error;
	}
}

export function cardByIndex(index: number): CardDef | undefined {
	const loaded = get(content);
	return loaded?.definitions.cards[index];
}

export function cardBySlug(slug: string): CardDef | undefined {
	return get(content)?.definitions.cards.find((card) => card.slug === slug);
}

export function cardIndex(slug: string): number {
	const loaded = get(content);
	if (!loaded) {
		return -1;
	}
	return loaded.catalog.cards.indexOf(slug);
}

export function troopForCard(card: CardDef): TroopDef | undefined {
	const loaded = get(content);
	if (!loaded || card.type.kind !== 'troop') {
		return undefined;
	}
	return loaded.definitions.troops[card.type.troop];
}

export function buildingForCard(card: CardDef): BuildingDef | undefined {
	const loaded = get(content);
	if (!loaded || card.type.kind !== 'building') {
		return undefined;
	}
	return loaded.definitions.buildings[card.type.building];
}

export function spellForCard(card: CardDef): SpellDef | undefined {
	const loaded = get(content);
	if (!loaded || card.type.kind !== 'spell') {
		return undefined;
	}
	return loaded.definitions.spells[card.type.spell];
}

export function troopBySlug(slug: string): TroopDef | undefined {
	return get(content)?.definitions.troops[slug];
}

export function buildingBySlug(slug: string): BuildingDef | undefined {
	return get(content)?.definitions.buildings[slug];
}

export function towerBySlug(slug: string): TowerDef | undefined {
	return get(content)?.definitions.towers[slug];
}

export function troopByIndex(index: number): TroopDef | undefined {
	const loaded = get(content);
	if (!loaded) {
		return undefined;
	}
	const slug = loaded.catalog.troops[index];
	return loaded.definitions.troops[slug];
}

export function buildingByIndex(index: number): BuildingDef | undefined {
	const loaded = get(content);
	if (!loaded) {
		return undefined;
	}
	const slug = loaded.catalog.buildings[index];
	return loaded.definitions.buildings[slug];
}

export function towerByIndex(index: number): TowerDef | undefined {
	const loaded = get(content);
	if (!loaded) {
		return undefined;
	}
	const slug = loaded.catalog.towers[index];
	return loaded.definitions.towers[slug];
}

export function spellByIndex(index: number): SpellDef | undefined {
	const loaded = get(content);
	if (!loaded) {
		return undefined;
	}
	const slug = loaded.catalog.spells[index];
	return loaded.definitions.spells[slug];
}

export function projectileVisual(index: number): string {
	const loaded = get(content);
	if (!loaded) {
		return 'ball';
	}
	const slug = loaded.catalog.projectiles[index];
	return loaded.definitions.projectiles[slug]?.visual ?? 'ball';
}

export function effectSlug(index: number): string {
	const loaded = get(content);
	if (!loaded) {
		return '';
	}
	return loaded.catalog.effects[index] ?? '';
}

export function effectKind(index: number): string {
	const loaded = get(content);
	const slug = loaded?.catalog.effects[index] ?? '';
	return loaded?.definitions.effects[slug]?.kind ?? '';
}

export function spellVisual(index: number): string {
	return spellByIndex(index)?.visual.variant ?? '';
}

export function arenaDef(slug: string) {
	return get(content)?.definitions.arenas[slug];
}

export function allCards(): CardDef[] {
	return get(content)?.definitions.cards ?? [];
}

export function allTroops(): TroopDef[] {
	const loaded = get(content);
	return loaded ? Object.values(loaded.definitions.troops) : [];
}
