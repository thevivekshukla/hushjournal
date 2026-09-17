import * as api from '$lib/api';
import {
	base64ToBytes,
	bytesToBase64,
	createWorkspaceSecrets,
	CryptoError,
	decryptText,
	encryptText,
	equalBytes,
	unlockDek,
	wrapDek
} from '$lib/crypto';
import { session } from '$lib/session.svelte';

export type Workspace = {
	id: string;
	name: string;
	keySalt: string;
	encryptedDek: string;
	passphraseHint: string | null;
	mask: boolean;
};

export type Shelf = {
	id: string;
	workspaceId: string;
	name: string;
	icon: string;
};

export type SaveStatus = 'saved' | 'saving' | 'error';

export type Entry = {
	id: string;
	shelfId: string;
	title: string;
	content: string;
	contentLoaded: boolean;
	saveStatus: SaveStatus;
	createdAt: string;
	updatedAt: string | null;
};

export const SHELF_ICONS = [
	'icon-[lucide--book-open]',
	'icon-[lucide--notebook]',
	'icon-[lucide--pen-line]',
	'icon-[lucide--calendar]',
	'icon-[lucide--lightbulb]',
	'icon-[lucide--bookmark]',
	'icon-[lucide--star]',
	'icon-[lucide--heart]',
	'icon-[lucide--folder]',
	'icon-[lucide--briefcase]',
	'icon-[lucide--map]',
	'icon-[lucide--camera]',
	'icon-[lucide--music]',
	'icon-[lucide--coffee]',
	'icon-[lucide--leaf]',
	'icon-[lucide--sparkles]'
] as const;

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const SAVE_DELAY_MS = 500;

export function formatEntryDate(date = new Date()) {
	return `${date.getDate()} ${MONTHS[date.getMonth()]} ${date.getFullYear()}`;
}

function shelfIcon(icon: string | null): string {
	return icon && (SHELF_ICONS as readonly string[]).includes(icon) ? icon : SHELF_ICONS[0];
}

function mapWorkspace(row: api.ApiWorkspace): Workspace {
	return {
		id: row.id,
		name: row.name,
		keySalt: row.key_salt,
		encryptedDek: row.encrypted_dek,
		passphraseHint: row.passphrase_hint,
		mask: row.mask ?? false
	};
}

function mapShelf(row: api.ApiShelf, dek: Uint8Array): Shelf {
	let name = 'Unable to decrypt';
	try {
		name = decryptText(dek, base64ToBytes(row.name), 'shelf.name');
	} catch {
		// Keep the fallback label when ciphertext does not match this DEK.
	}
	return {
		id: row.id,
		workspaceId: row.workspace_id,
		name,
		icon: shelfIcon(row.icon)
	};
}

function mapEntrySummary(row: api.ApiEntrySummary, dek: Uint8Array, previous?: Entry): Entry {
	let title = 'Unable to decrypt';
	try {
		title = decryptText(dek, base64ToBytes(row.title), 'entry.title');
	} catch {
		// Keep the fallback label when ciphertext does not match this DEK.
	}
	return {
		id: row.id,
		shelfId: row.shelf_id,
		title: previous?.contentLoaded ? previous.title : title,
		content: previous?.contentLoaded ? previous.content : '',
		contentLoaded: previous?.contentLoaded ?? false,
		saveStatus: previous?.saveStatus ?? 'saved',
		createdAt: row.created_at,
		updatedAt: row.updated_at
	};
}

function mapFullEntry(row: api.ApiEntry, dek: Uint8Array, previous?: Entry): Entry {
	const summary = mapEntrySummary(row, dek, previous);
	try {
		return {
			...summary,
			title: decryptText(dek, base64ToBytes(row.title), 'entry.title'),
			content: decryptText(dek, base64ToBytes(row.content), 'entry.content'),
			contentLoaded: true,
			saveStatus: previous?.saveStatus === 'saving' ? 'saving' : 'saved'
		};
	} catch {
		return { ...summary, contentLoaded: true };
	}
}

class Journal {
	workspaces = $state.raw<Workspace[]>([]);
	shelves = $state.raw<Shelf[]>([]);
	entries = $state.raw<Entry[]>([]);
	loading = $state(false);
	loadingMore = $state(false);
	hasMore = $state(false);
	entryOrder = $state<api.EntryOrder>('desc');
	error = $state<string | null>(null);
	#saveTimers = new Map<string, ReturnType<typeof setTimeout>>();
	#dirty = new Set<string>();
	#contentLoads = new Set<string>();
	#nextCursor: string | null = null;

	workspace(id: string) {
		return this.workspaces.find((workspace) => workspace.id === id);
	}

	shelvesFor(workspaceId: string) {
		return this.shelves.filter((shelf) => shelf.workspaceId === workspaceId);
	}

	entriesFor(shelfId: string) {
		const sign = this.entryOrder === 'asc' ? 1 : -1;
		return this.entries
			.filter((entry) => entry.shelfId === shelfId)
			.toSorted((a, b) => sign * a.id.localeCompare(b.id));
	}

	entry(id: string) {
		return this.entries.find((entry) => entry.id === id);
	}

	clearWorkspace() {
		this.shelves = [];
		this.entries = [];
		this.error = null;
		this.hasMore = false;
		this.loadingMore = false;
		this.#nextCursor = null;
	}

	async loadWorkspaces() {
		this.loading = true;
		this.error = null;
		try {
			this.workspaces = (await api.listWorkspaces()).map(mapWorkspace);
		} catch (error) {
			this.error = error instanceof Error ? error.message : 'Could not load workspaces.';
			throw error;
		} finally {
			this.loading = false;
		}
	}

	async createWorkspace(name: string, passphrase: string, passphraseHint = '') {
		const secrets = await createWorkspaceSecrets(passphrase);
		const hint = passphraseHint.trim();
		try {
			const workspace = mapWorkspace(
				await api.createWorkspace({
					name,
					key_salt: bytesToBase64(secrets.keySalt),
					encrypted_dek: bytesToBase64(secrets.encryptedDek),
					...(hint ? { passphrase_hint: hint } : {})
				})
			);
			this.workspaces = [workspace, ...this.workspaces];
			session.unlock(workspace.id, secrets.dek);
			try {
				const shelf = mapShelf(
					await api.createShelf(workspace.id, {
						name: bytesToBase64(encryptText(secrets.dek, 'Journal', 'shelf.name')),
						icon: SHELF_ICONS[0]
					}),
					secrets.dek
				);
				this.shelves = [...this.shelves, shelf];
			} catch {
				// The workspace is usable; a shelf can be added after opening it.
			}
			return workspace;
		} catch (error) {
			secrets.dek.fill(0);
			throw error;
		}
	}

	async updateWorkspace(
		id: string,
		patch: {
			name?: string;
			passphraseHint?: string;
			mask?: boolean;
			keySalt?: string;
			encryptedDek?: string;
		}
	) {
		const body: {
			name?: string;
			passphrase_hint?: string;
			mask?: boolean;
			key_salt?: string;
			encrypted_dek?: string;
		} = {};
		if (patch.name !== undefined) body.name = patch.name;
		if (patch.passphraseHint !== undefined) body.passphrase_hint = patch.passphraseHint;
		if (patch.mask !== undefined) body.mask = patch.mask;
		if (patch.keySalt !== undefined) body.key_salt = patch.keySalt;
		if (patch.encryptedDek !== undefined) body.encrypted_dek = patch.encryptedDek;
		const workspace = mapWorkspace(await api.updateWorkspace(id, body));
		this.workspaces = this.workspaces.map((item) => (item.id === id ? workspace : item));
		return workspace;
	}

	async changeWorkspacePassphrase(id: string, currentPassphrase: string, nextPassphrase: string) {
		const workspace = this.workspace(id);
		if (!workspace) throw new Error('Workspace not found.');
		const sessionDek = session.requireDek();
		const dek = await unlockDek(
			currentPassphrase,
			base64ToBytes(workspace.keySalt),
			base64ToBytes(workspace.encryptedDek)
		);
		try {
			if (!equalBytes(dek, sessionDek)) throw new CryptoError('Wrong passphrase');
		} finally {
			dek.fill(0);
		}
		const wrapped = await wrapDek(nextPassphrase, sessionDek);
		return this.updateWorkspace(id, {
			keySalt: bytesToBase64(wrapped.keySalt),
			encryptedDek: bytesToBase64(wrapped.encryptedDek)
		});
	}

	async unlockWorkspace(workspace: Workspace, passphrase: string) {
		const dek = await unlockDek(
			passphrase,
			base64ToBytes(workspace.keySalt),
			base64ToBytes(workspace.encryptedDek)
		);
		session.unlock(workspace.id, dek);
	}

	async loadShelves(workspaceId: string) {
		const dek = session.dek;
		if (!dek) return;
		this.loading = true;
		this.error = null;
		try {
			this.shelves = (await api.listShelves(workspaceId)).map((row) => mapShelf(row, dek));
		} catch (error) {
			this.error = error instanceof Error ? error.message : 'Could not load shelves.';
			throw error;
		} finally {
			this.loading = false;
		}
	}

	async createShelf(workspaceId: string, name: string, icon: string) {
		const dek = session.requireDek();
		const shelf = mapShelf(
			await api.createShelf(workspaceId, {
				name: bytesToBase64(encryptText(dek, name, 'shelf.name')),
				icon
			}),
			dek
		);
		this.shelves = [...this.shelves, shelf];
		return shelf;
	}

	async updateShelf(id: string, patch: { name?: string; icon?: string }) {
		const dek = session.requireDek();
		const body: { name?: string; icon?: string } = {};
		if (patch.name !== undefined) {
			body.name = bytesToBase64(encryptText(dek, patch.name, 'shelf.name'));
		}
		if (patch.icon !== undefined) body.icon = patch.icon;
		const shelf = mapShelf(await api.updateShelf(id, body), dek);
		this.shelves = this.shelves.map((item) => (item.id === id ? shelf : item));
	}

	async deleteShelf(id: string) {
		await this.flush();
		await api.deleteShelf(id);
		this.shelves = this.shelves.filter((shelf) => shelf.id !== id);
		this.entries = this.entries.filter((entry) => entry.shelfId !== id);
	}

	async loadEntries(shelfId: string, opts: { cursor?: string | null; append?: boolean } = {}) {
		const dek = session.dek;
		if (!dek) return;
		const append = opts.append ?? false;
		if (append) this.loadingMore = true;
		else {
			await this.flush();
			this.hasMore = false;
			this.#nextCursor = null;
		}
		this.error = null;
		try {
			const page = await api.listEntries(shelfId, {
				cursor: opts.cursor,
				order: this.entryOrder
			});
			const previous = new Map(this.entries.map((entry) => [entry.id, entry]));
			const incoming = page.entries.map((row) => mapEntrySummary(row, dek, previous.get(row.id)));
			if (append) {
				const seen = new Set(
					this.entries.filter((entry) => entry.shelfId === shelfId).map((entry) => entry.id)
				);
				this.entries = [...this.entries, ...incoming.filter((entry) => !seen.has(entry.id))];
			} else {
				this.entries = [...this.entries.filter((entry) => entry.shelfId !== shelfId), ...incoming];
			}
			this.#nextCursor = page.next_cursor;
			this.hasMore = page.next_cursor != null;
		} catch (error) {
			this.error = error instanceof Error ? error.message : 'Could not load notes.';
		} finally {
			this.loadingMore = false;
		}
	}

	async loadMore(shelfId: string) {
		if (!this.#nextCursor || this.loadingMore) return;
		await this.loadEntries(shelfId, { cursor: this.#nextCursor, append: true });
	}

	async ensureContent(id: string) {
		const dek = session.dek;
		const current = this.entry(id);
		if (!dek || !current || current.contentLoaded || this.#contentLoads.has(id)) return;
		this.#contentLoads.add(id);
		try {
			const row = await api.getEntry(id);
			const previous = this.entry(id);
			if (!previous) return;
			if (this.#dirty.has(id)) {
				this.entries = this.entries.map((entry) =>
					entry.id === id ? { ...entry, contentLoaded: true } : entry
				);
				return;
			}
			const loaded = mapFullEntry(row, dek, previous);
			this.entries = this.entries.map((entry) => (entry.id === id ? loaded : entry));
		} catch (error) {
			this.error = error instanceof Error ? error.message : 'Could not load the note.';
		} finally {
			this.#contentLoads.delete(id);
		}
	}

	async createEntry(shelfId: string, title = formatEntryDate()) {
		const dek = session.requireDek();
		const row = await api.createEntry(shelfId, {
			title: bytesToBase64(encryptText(dek, title, 'entry.title')),
			content: bytesToBase64(encryptText(dek, '', 'entry.content'))
		});
		const entry = mapFullEntry(row, dek);
		const others = this.entries.filter((item) => item.shelfId !== shelfId);
		const loaded = this.entries.filter((item) => item.shelfId === shelfId);
		const sign = this.entryOrder === 'asc' ? 1 : -1;
		this.entries = [
			...others,
			...[...loaded, entry].toSorted((a, b) => sign * a.id.localeCompare(b.id))
		];
		return entry;
	}

	updateEntry(id: string, patch: { title?: string; content?: string }) {
		this.entries = this.entries.map((entry) =>
			entry.id === id
				? {
						...entry,
						...patch,
						contentLoaded: true,
						saveStatus: 'saving',
						updatedAt: new Date().toISOString()
					}
				: entry
		);
		this.#dirty.add(id);
		const previous = this.#saveTimers.get(id);
		if (previous) clearTimeout(previous);
		this.#saveTimers.set(
			id,
			setTimeout(() => {
				this.#saveTimers.delete(id);
				void this.#save(id);
			}, SAVE_DELAY_MS)
		);
	}

	async deleteEntry(id: string) {
		this.#cancelSave(id);
		await api.deleteEntry(id);
		this.entries = this.entries.filter((entry) => entry.id !== id);
	}

	async flush(id?: string) {
		if (id) {
			this.#cancelSave(id, false);
			if (this.#dirty.has(id)) await this.#save(id);
			return;
		}
		const ids = [...new Set([...this.#dirty, ...this.#saveTimers.keys()])];
		for (const pendingId of ids) {
			this.#cancelSave(pendingId, false);
			if (this.#dirty.has(pendingId)) await this.#save(pendingId);
		}
	}

	#cancelSave(id: string, forgetDirty = true) {
		const timer = this.#saveTimers.get(id);
		if (timer) clearTimeout(timer);
		this.#saveTimers.delete(id);
		if (forgetDirty) this.#dirty.delete(id);
	}

	async #save(id: string) {
		const dek = session.dek;
		const entry = this.entry(id);
		if (!dek || !entry || !this.#dirty.has(id)) return;
		this.#dirty.delete(id);
		this.#setSaveStatus(id, 'saving');
		try {
			const row = await api.updateEntry(id, {
				title: bytesToBase64(encryptText(dek, entry.title, 'entry.title')),
				content: bytesToBase64(encryptText(dek, entry.content, 'entry.content'))
			});
			if (this.#dirty.has(id)) return;
			this.entries = this.entries.map((item) =>
				item.id === id
					? {
							...item,
							updatedAt: row.updated_at,
							saveStatus: 'saved'
						}
					: item
			);
		} catch (error) {
			this.#dirty.add(id);
			this.#setSaveStatus(id, 'error');
			this.error = error instanceof Error ? error.message : 'Could not save.';
		}
	}

	#setSaveStatus(id: string, saveStatus: SaveStatus) {
		this.entries = this.entries.map((entry) =>
			entry.id === id ? { ...entry, saveStatus } : entry
		);
	}
}

export { CryptoError };

export const journal = new Journal();
