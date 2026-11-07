import { derived, get, writable } from 'svelte/store';
import type { Locale } from '#lib/types';
import ptBR from './ui.pt-BR.json';
import en from './ui.en.json';
import cardsPt from './cards.pt-BR.json';
import cardsEn from './cards.en.json';

type Dict = Record<string, unknown>;
type CardText = { name: string; description: string };

const dictionaries: Record<Locale, Dict> = {
	'pt-BR': ptBR as Dict,
	en: en as Dict
};

const cardDictionaries: Record<Locale, Record<string, CardText>> = {
	'pt-BR': cardsPt as Record<string, CardText>,
	en: cardsEn as Record<string, CardText>
};

function initialLocale(): Locale {
	if (typeof window === 'undefined') {
		return 'pt-BR';
	}
	const stored = window.localStorage.getItem('royale.locale');
	return stored === 'en' ? 'en' : 'pt-BR';
}

export const locale = writable<Locale>(initialLocale());

export function setLocale(next: Locale) {
	locale.set(next);
	if (typeof window !== 'undefined') {
		window.localStorage.setItem('royale.locale', next);
	}
}

export function currentLocale(): Locale {
	return get(locale);
}

function lookup(dict: Dict, key: string): string | undefined {
	let current: unknown = dict;
	for (const part of key.split('.')) {
		if (typeof current !== 'object' || current === null) {
			return undefined;
		}
		current = (current as Dict)[part];
	}
	return typeof current === 'string' ? current : undefined;
}

export function createTranslator(target: Locale) {
	const dict = dictionaries[target];
	return (key: string, vars?: Record<string, string | number>): string => {
		let value = lookup(dict, key) ?? key;
		if (vars) {
			for (const [name, replacement] of Object.entries(vars)) {
				value = value.replaceAll(`{${name}}`, String(replacement));
			}
		}
		return value;
	};
}

export const i18n = derived(locale, ($locale) => ({
	locale: $locale,
	t: createTranslator($locale),
	card: (slug: string): CardText =>
		cardDictionaries[$locale][slug] ?? { name: slug, description: '' }
}));
