import { get, writable } from 'svelte/store';
import { api } from '#lib/api';
import { setLocale } from '#lib/i18n';
import type { User } from '#lib/types';

export const user = writable<User | null>(null);
export const deck = writable<string[]>([]);
export const authChecked = writable(false);

export async function restoreSession(): Promise<User | null> {
	try {
		const result = await api.me();
		user.set(result.user);
		setLocale(result.user.locale === 'en' ? 'en' : 'pt-BR');
		await refreshDeck();
		return result.user;
	} catch {
		user.set(null);
		return null;
	} finally {
		authChecked.set(true);
	}
}

export async function refreshDeck() {
	try {
		const result = await api.getDeck();
		deck.set(result.cards);
	} catch {
		deck.set([]);
	}
}

export async function login(email: string, password: string): Promise<User> {
	const result = await api.login({ email, password });
	user.set(result.user);
	setLocale(result.user.locale === 'en' ? 'en' : 'pt-BR');
	await refreshDeck();
	return result.user;
}

export async function register(payload: {
	username: string;
	email: string;
	password: string;
	password_confirmation: string;
	locale: string;
}): Promise<User> {
	const result = await api.register(payload);
	user.set(result.user);
	setLocale(result.user.locale === 'en' ? 'en' : 'pt-BR');
	await refreshDeck();
	return result.user;
}

export async function logout() {
	try {
		await api.logout();
	} finally {
		user.set(null);
		deck.set([]);
	}
}

export async function saveDeck(cards: string[]) {
	await api.putDeck(cards);
	deck.set(cards);
}

export function currentUser(): User | null {
	return get(user);
}

export function currentDeck(): string[] {
	return get(deck);
}
