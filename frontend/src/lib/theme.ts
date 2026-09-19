export const DAISYUI_THEMES = [
	'silk',
	'cupcake',
	'bumblebee',
	'emerald',
	'corporate',
	'nord',
	'lemonade',
	'winter',
	'caramellatte',
	'retro',
	'dim',
	'forest',
	'dracula',
	'night',
	'coffee',
	'synthwave',
	'abyss',
	'luxury',
	'halloween',
	'sunset'
] as const;

export type DaisyuiTheme = (typeof DAISYUI_THEMES)[number];
export type JournalTheme = '' | DaisyuiTheme;

export const JOURNAL_THEMES = [
	{ id: '' as const, label: 'App default' },
	...DAISYUI_THEMES.map((id) => ({
		id,
		label: id === 'caramellatte' ? 'Caramel latte' : id.charAt(0).toUpperCase() + id.slice(1)
	}))
];

export function journalTheme(value: string | null | undefined): JournalTheme {
	return DAISYUI_THEMES.includes(value as DaisyuiTheme) ? (value as DaisyuiTheme) : '';
}

export function restoreAppTheme() {
	if (typeof document === 'undefined') return;
	let theme = 'silk';
	try {
		const stored = localStorage.getItem('theme');
		if (stored === 'silk' || stored === 'dim') {
			theme = stored;
		} else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
			theme = 'dim';
		}
	} catch {
		// Keep silk when storage is unavailable.
	}
	document.documentElement.setAttribute('data-theme', theme);
}
