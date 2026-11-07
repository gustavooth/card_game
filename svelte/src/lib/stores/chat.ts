import { writable } from 'svelte/store';

export interface ChatEntry {
	user: string;
	text: string;
	ts: number;
}

export const chatMessages = writable<ChatEntry[]>([]);
export const chatConnected = writable(false);

export function appendChat(entry: ChatEntry) {
	chatMessages.update((messages) => [...messages.slice(-99), entry]);
}
