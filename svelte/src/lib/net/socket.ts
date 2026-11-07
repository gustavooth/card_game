import { decode, encode } from '@msgpack/msgpack';

type SocketMessage = Record<string, unknown>;
type Listener = (message: SocketMessage) => void;
type StatusListener = (connected: boolean) => void;

class GameSocket {
	private socket: WebSocket | null = null;
	private listeners = new Set<Listener>();
	private statusListeners = new Set<StatusListener>();
	private manualClose = false;
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	private reconnectAttempt = 0;
	private pingTimer: ReturnType<typeof setInterval> | null = null;

	get connected() {
		return this.socket?.readyState === WebSocket.OPEN;
	}

	connect() {
		if (typeof window === 'undefined' || this.socket) {
			return;
		}
		this.manualClose = false;
		const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
		const socket = new WebSocket(`${protocol}://${window.location.host}/ws`);
		socket.binaryType = 'arraybuffer';
		this.socket = socket;

		socket.onopen = () => {
			this.reconnectAttempt = 0;
			this.notifyStatus(true);
			this.startPing();
		};
		socket.onmessage = (event) => {
			try {
				const data =
					event.data instanceof ArrayBuffer
						? new Uint8Array(event.data)
						: new Uint8Array(event.data as ArrayBuffer);
				const message = decode(data) as SocketMessage;
				for (const listener of this.listeners) {
					listener(message);
				}
			} catch (error) {
				console.warn('Mensagem inválida do servidor', error);
			}
		};
		socket.onclose = () => {
			this.stopPing();
			this.socket = null;
			this.notifyStatus(false);
			if (!this.manualClose) {
				this.scheduleReconnect();
			}
		};
		socket.onerror = () => {
			socket.close();
		};
	}

	disconnect() {
		this.manualClose = true;
		this.stopPing();
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
			this.reconnectTimer = null;
		}
		this.socket?.close();
		this.socket = null;
	}

	onMessage(listener: Listener) {
		this.listeners.add(listener);
		return () => this.listeners.delete(listener);
	}

	onStatus(listener: StatusListener) {
		this.statusListeners.add(listener);
		return () => this.statusListeners.delete(listener);
	}

	send(message: Record<string, unknown>) {
		if (!this.connected) {
			return false;
		}
		this.socket?.send(encode(message));
		return true;
	}

	joinQueue() {
		this.send({ t: 'q' });
	}

	leaveQueue() {
		this.send({ t: 'lq' });
	}

	deploy(slot: number, x: number, y: number) {
		this.send({ t: 'd', s: slot, x, y });
	}

	chat(text: string) {
		return this.send({ t: 'c', m: text });
	}

	private startPing() {
		this.stopPing();
		this.pingTimer = setInterval(() => {
			if (this.connected) {
				this.send({ t: 'p', ts: Date.now() });
			}
		}, 12_000);
	}

	private stopPing() {
		if (this.pingTimer) {
			clearInterval(this.pingTimer);
			this.pingTimer = null;
		}
	}

	private scheduleReconnect() {
		if (this.reconnectTimer) {
			return;
		}
		const delay = Math.min(8000, 700 * Math.pow(1.7, this.reconnectAttempt));
		this.reconnectAttempt += 1;
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = null;
			this.connect();
		}, delay);
	}

	private notifyStatus(connected: boolean) {
		for (const listener of this.statusListeners) {
			listener(connected);
		}
	}
}

export const gameSocket = new GameSocket();
