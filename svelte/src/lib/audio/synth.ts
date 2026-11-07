type Scene = 'menu' | 'battle';

class AudioEngine {
	private context: AudioContext | null = null;
	private master: GainNode | null = null;
	private sfxBus: GainNode | null = null;
	private musicBus: GainNode | null = null;
	private musicTimer: ReturnType<typeof setInterval> | null = null;
	private scene: Scene | null = null;
	private lastHit = 0;
	muted = false;

	constructor() {
		if (typeof window !== 'undefined') {
			this.muted = window.localStorage.getItem('royale.muted') === '1';
		}
	}

	ensure() {
		if (typeof window === 'undefined') {
			return;
		}
		if (!this.context) {
			const AudioContextClass =
				window.AudioContext ??
				(window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
			if (!AudioContextClass) {
				return;
			}
			this.context = new AudioContextClass();
			this.master = this.context.createGain();
			this.master.gain.value = this.muted ? 0 : 0.5;
			this.master.connect(this.context.destination);
			this.sfxBus = this.context.createGain();
			this.sfxBus.gain.value = 0.7;
			this.sfxBus.connect(this.master);
			this.musicBus = this.context.createGain();
			this.musicBus.gain.value = 0.18;
			this.musicBus.connect(this.master);
		}
		if (this.context.state === 'suspended') {
			void this.context.resume();
		}
	}

	setMuted(muted: boolean) {
		this.muted = muted;
		if (typeof window !== 'undefined') {
			window.localStorage.setItem('royale.muted', muted ? '1' : '0');
		}
		if (this.master && this.context) {
			this.master.gain.setTargetAtTime(muted ? 0 : 0.5, this.context.currentTime, 0.03);
		}
	}

	toggleMute() {
		this.setMuted(!this.muted);
	}

	private tone(
		frequency: number,
		duration: number,
		options: {
			type?: OscillatorType;
			gain?: number;
			slideTo?: number;
			delay?: number;
			bus?: GainNode | null;
		} = {}
	) {
		this.ensure();
		if (!this.context) {
			return;
		}
		const now = this.context.currentTime + (options.delay ?? 0);
		const oscillator = this.context.createOscillator();
		const gain = this.context.createGain();
		oscillator.type = options.type ?? 'sine';
		oscillator.frequency.setValueAtTime(frequency, now);
		if (options.slideTo) {
			oscillator.frequency.exponentialRampToValueAtTime(
				Math.max(20, options.slideTo),
				now + duration
			);
		}
		const level = options.gain ?? 0.2;
		gain.gain.setValueAtTime(0.0001, now);
		gain.gain.exponentialRampToValueAtTime(level, now + 0.012);
		gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);
		oscillator.connect(gain);
		gain.connect(options.bus ?? this.sfxBus ?? this.context.destination);
		oscillator.start(now);
		oscillator.stop(now + duration + 0.03);
	}

	private noise(
		duration: number,
		options: {
			gain?: number;
			frequency?: number;
			type?: BiquadFilterType;
			delay?: number;
			bus?: GainNode | null;
		} = {}
	) {
		this.ensure();
		if (!this.context) {
			return;
		}
		const now = this.context.currentTime + (options.delay ?? 0);
		const frames = Math.max(1, Math.floor(this.context.sampleRate * duration));
		const buffer = this.context.createBuffer(1, frames, this.context.sampleRate);
		const data = buffer.getChannelData(0);
		for (let index = 0; index < frames; index += 1) {
			data[index] = (Math.random() * 2 - 1) * (1 - index / frames);
		}
		const source = this.context.createBufferSource();
		source.buffer = buffer;
		const filter = this.context.createBiquadFilter();
		filter.type = options.type ?? 'lowpass';
		filter.frequency.value = options.frequency ?? 1200;
		const gain = this.context.createGain();
		gain.gain.value = options.gain ?? 0.25;
		source.connect(filter);
		filter.connect(gain);
		gain.connect(options.bus ?? this.sfxBus ?? this.context.destination);
		source.start(now);
	}

	play(name: string) {
		switch (name) {
			case 'click':
				this.tone(720, 0.06, { type: 'triangle', gain: 0.12, slideTo: 900 });
				break;
			case 'deploy':
				this.noise(0.22, { gain: 0.18, frequency: 900, type: 'bandpass' });
				this.tone(220, 0.18, { type: 'triangle', gain: 0.16, slideTo: 440 });
				break;
			case 'hit':
				this.noise(0.07, { gain: 0.12, frequency: 2200, type: 'highpass' });
				break;
			case 'sword':
				this.tone(1300, 0.08, { type: 'square', gain: 0.08, slideTo: 700 });
				break;
			case 'arrow':
				this.noise(0.05, { gain: 0.08, frequency: 3200, type: 'highpass' });
				break;
			case 'magic':
				this.tone(520, 0.18, { type: 'sine', gain: 0.12, slideTo: 1040 });
				break;
			case 'spell':
				this.tone(880, 0.3, { type: 'sawtooth', gain: 0.1, slideTo: 160 });
				this.noise(0.25, { gain: 0.15, frequency: 700 });
				break;
			case 'freeze':
				this.tone(1600, 0.35, { type: 'sine', gain: 0.08, slideTo: 2400 });
				this.tone(1200, 0.35, { type: 'triangle', gain: 0.05, slideTo: 2000, delay: 0.05 });
				break;
			case 'fire':
				this.noise(0.4, { gain: 0.22, frequency: 600, type: 'lowpass' });
				break;
			case 'zap':
				this.noise(0.12, { gain: 0.18, frequency: 4200, type: 'highpass' });
				this.tone(2200, 0.1, { type: 'square', gain: 0.05 });
				break;
			case 'death':
				this.tone(320, 0.24, { type: 'sawtooth', gain: 0.11, slideTo: 80 });
				break;
			case 'tower_down':
				this.tone(110, 0.6, { type: 'sine', gain: 0.3, slideTo: 45 });
				this.noise(0.5, { gain: 0.24, frequency: 300 });
				break;
			case 'king_active':
				this.tone(392, 0.22, { type: 'triangle', gain: 0.14 });
				this.tone(523, 0.22, { type: 'triangle', gain: 0.14, delay: 0.12 });
				this.tone(659, 0.35, { type: 'triangle', gain: 0.14, delay: 0.24 });
				break;
			case 'elixir':
				this.tone(880, 0.07, { type: 'sine', gain: 0.06, slideTo: 1200 });
				break;
			case 'error':
				this.tone(180, 0.16, { type: 'square', gain: 0.1, slideTo: 120 });
				break;
			case 'queue':
				this.tone(440, 0.1, { type: 'triangle', gain: 0.08 });
				this.tone(660, 0.14, { type: 'triangle', gain: 0.08, delay: 0.1 });
				break;
			case 'victory':
				[523, 659, 784, 1047].forEach((frequency, index) => {
					this.tone(frequency, 0.4, { type: 'triangle', gain: 0.14, delay: index * 0.14 });
				});
				break;
			case 'defeat':
				[392, 349, 294, 220].forEach((frequency, index) => {
					this.tone(frequency, 0.45, { type: 'sawtooth', gain: 0.09, delay: index * 0.16 });
				});
				break;
			case 'match_start':
				this.tone(392, 0.2, { type: 'triangle', gain: 0.1 });
				this.tone(587, 0.2, { type: 'triangle', gain: 0.1, delay: 0.15 });
				this.tone(784, 0.3, { type: 'triangle', gain: 0.12, delay: 0.3 });
				break;
			default:
				break;
		}
	}

	startMusic(scene: Scene) {
		this.ensure();
		if (!this.context || this.scene === scene) {
			return;
		}
		this.stopMusic();
		this.scene = scene;
		const chords =
			scene === 'menu'
				? [
						[220, 261.63, 329.63],
						[174.61, 220, 261.63],
						[196, 246.94, 293.66],
						[164.81, 207.65, 246.94]
					]
				: [
						[196, 246.94, 293.66],
						[174.61, 220, 261.63],
						[146.83, 185, 220],
						[164.81, 207.65, 246.94]
					];
		let step = 0;
		const tick = () => {
			const chord = chords[step % chords.length];
			for (const frequency of chord) {
				this.tone(frequency, scene === 'menu' ? 2.6 : 1.1, {
					type: 'sine',
					gain: scene === 'menu' ? 0.045 : 0.035,
					bus: this.musicBus
				});
			}
			const arpeggio = chord[(step + 1) % chord.length] * 2;
			this.tone(arpeggio, scene === 'menu' ? 0.8 : 0.22, {
				type: 'triangle',
				gain: scene === 'menu' ? 0.02 : 0.03,
				bus: this.musicBus
			});
			if (scene === 'battle' && step % 2 === 0) {
				this.noise(0.08, { gain: 0.025, frequency: 180, bus: this.musicBus });
			}
			step += 1;
		};
		tick();
		this.musicTimer = setInterval(tick, scene === 'menu' ? 2600 : 900);
	}

	stopMusic() {
		if (this.musicTimer) {
			clearInterval(this.musicTimer);
			this.musicTimer = null;
		}
		this.scene = null;
	}

	handleEvents(events: number[][], localSide: 0 | 1) {
		const now = performance.now();
		for (const event of events) {
			const kind = event[0];
			switch (kind) {
				case 1:
					this.play('deploy');
					break;
				case 2:
					if (now - this.lastHit > 55) {
						this.lastHit = now;
						this.play('hit');
					}
					break;
				case 3:
					this.play('death');
					break;
				case 4: {
					const spellIndex = event[4];
					this.play(spellIndex % 3 === 0 ? 'fire' : spellIndex % 3 === 1 ? 'freeze' : 'spell');
					break;
				}
				case 5:
					this.play('tower_down');
					break;
				case 6:
					this.play('king_active');
					break;
				case 7:
					this.play(event[3] === localSide ? 'victory' : 'defeat');
					break;
				default:
					break;
			}
		}
	}
}

export const audio = new AudioEngine();
