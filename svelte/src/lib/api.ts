import type { Content, PlayerStats, RecentMatch, ReplayMeta, User } from '#lib/types';

export class ApiError extends Error {
	code: string;
	status: number;

	constructor(message: string, code = 'error', status = 0) {
		super(message);
		this.name = 'ApiError';
		this.code = code;
		this.status = status;
	}
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
	const headers = new Headers(init.headers);
	if (init.body && !headers.has('Content-Type')) {
		headers.set('Content-Type', 'application/json');
	}
	const response = await fetch(path, {
		...init,
		headers,
		credentials: 'include',
		cache: 'no-store'
	});
	if (!response.ok) {
		let code = 'http_error';
		let message = `Erro ${response.status}`;
		try {
			const body = await response.json();
			code = body.code ?? code;
			message = body.message ?? message;
		} catch {
			// resposta sem JSON
		}
		throw new ApiError(message, code, response.status);
	}
	if (response.status === 204) {
		return undefined as T;
	}
	return (await response.json()) as T;
}

export const api = {
	content: () => request<Content>('/api/content'),
	register: (payload: {
		username: string;
		email: string;
		password: string;
		password_confirmation: string;
		locale: string;
	}) =>
		request<{ user: User }>('/api/auth/register', {
			method: 'POST',
			body: JSON.stringify(payload)
		}),
	login: (payload: { email: string; password: string }) =>
		request<{ user: User }>('/api/auth/login', {
			method: 'POST',
			body: JSON.stringify(payload)
		}),
	logout: () => request<{ ok: boolean }>('/api/auth/logout', { method: 'POST' }),
	me: () => request<{ user: User }>('/api/auth/me'),
	getDeck: () => request<{ cards: string[] }>('/api/deck'),
	putDeck: (cards: string[]) =>
		request<{ cards: string[] }>('/api/deck', {
			method: 'PUT',
			body: JSON.stringify({ cards })
		}),
	profile: () =>
		request<{ user: User; stats: PlayerStats; recent: RecentMatch[] }>('/api/profile'),
	replays: () => request<{ replays: ReplayMeta[] }>('/api/replays'),
	setLocale: (locale: string) =>
		request<{ locale: string }>('/api/profile/locale', {
			method: 'PUT',
			body: JSON.stringify({ locale })
		}),
	replayData: async (id: string): Promise<ArrayBuffer> => {
		const response = await fetch(`/api/replays/${id}`, {
			credentials: 'include',
			cache: 'no-store'
		});
		if (!response.ok) {
			let message = 'Replay indisponível';
			try {
				const body = await response.json();
				message = body.message ?? message;
			} catch {
				// ignore
			}
			throw new ApiError(message, 'replay_error', response.status);
		}
		return response.arrayBuffer();
	}
};
