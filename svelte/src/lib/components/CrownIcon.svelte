<script lang="ts">
	interface Props {
		size?: number;
		class?: string;
	}

	let { size = 24, class: className = '' }: Props = $props();
</script>

<svg
	class={className}
	width={size}
	height={size}
	viewBox="0 0 64 64"
	fill="none"
	aria-hidden="true"
>
	<defs>
		<linearGradient id="royal-crown-gold" x1="8" y1="8" x2="56" y2="58" gradientUnits="userSpaceOnUse">
			<stop offset="0" stop-color="#fff7c2" />
			<stop offset="0.28" stop-color="#ffd95e" />
			<stop offset="0.58" stop-color="#e5a923" />
			<stop offset="0.82" stop-color="#b9780f" />
			<stop offset="1" stop-color="#8a4d05" />
		</linearGradient>
		<linearGradient id="royal-crown-light" x1="12" y1="18" x2="52" y2="40" gradientUnits="userSpaceOnUse">
			<stop offset="0" stop-color="#fffbe0" stop-opacity="0.95" />
			<stop offset="1" stop-color="#ffe27a" stop-opacity="0" />
		</linearGradient>
	</defs>
	<path
		d="M8 46 5.5 16.5l14.7 11.8L32 7l11.8 21.3 14.7-11.8L56 46H8Z"
		fill="url(#royal-crown-gold)"
		stroke="#5d3603"
		stroke-width="2.2"
		stroke-linejoin="round"
	/>
	<path
		d="M10 28.5 8.2 18.7l11.3 9.1L32 13.8l12.5 14 11.3-9.1-1.8 9.8"
		fill="none"
		stroke="url(#royal-crown-light)"
		stroke-width="2.4"
		stroke-linecap="round"
		stroke-linejoin="round"
	/>
	<path
		d="M8 46h48v7.5c0 2-1.6 3.5-3.6 3.5H11.6A3.6 3.6 0 0 1 8 53.5V46Z"
		fill="url(#royal-crown-gold)"
		stroke="#5d3603"
		stroke-width="2.2"
	/>
	<path d="M12 48h40" stroke="#fff2a8" stroke-opacity="0.45" stroke-width="1.4" />
	<circle cx="32" cy="24.5" r="4.2" fill="#d9233b" stroke="#5d0a16" stroke-width="1.4" />
	<circle cx="31" cy="23.4" r="1.4" fill="#ff9aa7" fill-opacity="0.85" />
	<circle cx="14.5" cy="40" r="3" fill="#2f6ee8" stroke="#0b2f73" stroke-width="1.2" />
	<circle cx="49.5" cy="40" r="3" fill="#2f6ee8" stroke="#0b2f73" stroke-width="1.2" />
	<circle cx="32" cy="49.5" r="2.6" fill="#57d68d" stroke="#0c4f2c" stroke-width="1.2" />
</svg>
