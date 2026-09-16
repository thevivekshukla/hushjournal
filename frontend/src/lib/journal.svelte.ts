export type Workspace = {
	id: string;
	name: string;
};

export type Shelf = {
	id: string;
	workspaceId: string;
	name: string;
	icon: string;
};

export type Entry = {
	id: string;
	shelfId: string;
	title: string;
	content: string;
	createdAt: string;
	updatedAt: string | null;
};

export const SHELF_ICONS = [
	'icon-[lucide--book-open]',
	'icon-[lucide--lightbulb]',
	'icon-[lucide--map]',
	'icon-[lucide--coffee]',
	'icon-[lucide--leaf]',
	'icon-[lucide--pen-line]'
] as const;

const STORAGE_KEY = 'e2ejournal.journal';

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

export function formatEntryDate(date = new Date()) {
	return `${date.getDate()} ${MONTHS[date.getMonth()]} ${date.getFullYear()}`;
}

function nowIso() {
	return new Date().toISOString();
}

function seed(): { workspaces: Workspace[]; shelves: Shelf[]; entries: Entry[] } {
	return {
		workspaces: [
			{ id: 'ws-personal', name: 'Personal' },
			{ id: 'ws-work', name: 'Work' }
		],
		shelves: [
			{
				id: 'shelf-journal',
				workspaceId: 'ws-personal',
				name: 'Journal',
				icon: 'icon-[lucide--book-open]'
			},
			{
				id: 'shelf-ideas',
				workspaceId: 'ws-personal',
				name: 'Ideas',
				icon: 'icon-[lucide--lightbulb]'
			},
			{
				id: 'shelf-notes',
				workspaceId: 'ws-work',
				name: 'Notes',
				icon: 'icon-[lucide--pen-line]'
			}
		],
		entries: [
			{
				id: 'entry-today',
				shelfId: 'shelf-journal',
				title: formatEntryDate(),
				content:
					'Quiet morning. Made coffee, sat by the window, and finally opened this page.\n\nThe point of this app is simple: write here, and nobody else can read it. Not even the server.',
				createdAt: nowIso(),
				updatedAt: null
			},
			{
				id: 'entry-yesterday',
				shelfId: 'shelf-journal',
				title: formatEntryDate(new Date(Date.now() - 86400000)),
				content: 'Walked longer than I meant to. Came home with dusty shoes and a clearer head.',
				createdAt: new Date(Date.now() - 86400000).toISOString(),
				updatedAt: null
			},
			{
				id: 'entry-idea',
				shelfId: 'shelf-ideas',
				title: 'A shelf for each kind of writing',
				content:
					'Journal for the day. Ideas for things that are not yet a day. Work stays in its own workspace so it never sits next to the rest.',
				createdAt: nowIso(),
				updatedAt: null
			},
			{
				id: 'entry-work',
				shelfId: 'shelf-notes',
				title: formatEntryDate(),
				content:
					'Standup notes:\n- Ship the workspace APIs\n- Prototype the journal UI\n- Keep the layout quiet',
				createdAt: nowIso(),
				updatedAt: null
			}
		]
	};
}

function load() {
	if (typeof sessionStorage === 'undefined') return seed();
	try {
		const raw = sessionStorage.getItem(STORAGE_KEY);
		if (!raw) return seed();
		return JSON.parse(raw) as ReturnType<typeof seed>;
	} catch {
		return seed();
	}
}

class Journal {
	workspaces = $state<Workspace[]>([]);
	shelves = $state<Shelf[]>([]);
	entries = $state<Entry[]>([]);

	constructor() {
		const data = load();
		this.workspaces = data.workspaces;
		this.shelves = data.shelves;
		this.entries = data.entries;
	}

	private persist() {
		if (typeof sessionStorage === 'undefined') return;
		sessionStorage.setItem(
			STORAGE_KEY,
			JSON.stringify({
				workspaces: this.workspaces,
				shelves: this.shelves,
				entries: this.entries
			})
		);
	}

	workspace(id: string) {
		return this.workspaces.find((workspace) => workspace.id === id);
	}

	shelvesFor(workspaceId: string) {
		return this.shelves.filter((shelf) => shelf.workspaceId === workspaceId);
	}

	entriesFor(shelfId: string) {
		return this.entries
			.filter((entry) => entry.shelfId === shelfId)
			.toSorted((a, b) => b.createdAt.localeCompare(a.createdAt));
	}

	createWorkspace(name: string) {
		const workspace: Workspace = { id: crypto.randomUUID(), name };
		this.workspaces = [workspace, ...this.workspaces];
		const shelf: Shelf = {
			id: crypto.randomUUID(),
			workspaceId: workspace.id,
			name: 'Journal',
			icon: 'icon-[lucide--book-open]'
		};
		this.shelves = [...this.shelves, shelf];
		this.persist();
		return workspace;
	}

	createShelf(workspaceId: string, name: string, icon: string) {
		const shelf: Shelf = {
			id: crypto.randomUUID(),
			workspaceId,
			name,
			icon
		};
		this.shelves = [...this.shelves, shelf];
		this.persist();
		return shelf;
	}

	createEntry(shelfId: string, title = formatEntryDate()) {
		const existing = this.entries.find(
			(entry) => entry.shelfId === shelfId && entry.title === title
		);
		if (existing) return existing;
		const entry: Entry = {
			id: crypto.randomUUID(),
			shelfId,
			title,
			content: '',
			createdAt: nowIso(),
			updatedAt: null
		};
		this.entries = [entry, ...this.entries];
		this.persist();
		return entry;
	}

	updateEntry(id: string, patch: { title?: string; content?: string }) {
		this.entries = this.entries.map((entry) =>
			entry.id === id ? { ...entry, ...patch, updatedAt: nowIso() } : entry
		);
		this.persist();
	}

	deleteEntry(id: string) {
		this.entries = this.entries.filter((entry) => entry.id !== id);
		this.persist();
	}
}

export const journal = new Journal();
