import * as api from '$lib/api';
import {
	base64ToBytes,
	bytesToBase64,
	createJournalSecrets,
	CryptoError,
	decryptText,
	encryptText,
	equalBytes,
	unlockDek,
	wrapDek
} from '$lib/crypto';
import { ApiError } from '$lib/http';
import { session } from '$lib/session.svelte';

export type Journal = {
	id: string;
	name: string;
	keySalt: string;
	encryptedDek: string;
	passphraseHint: string | null;
	mask: boolean;
	totalSize: number;
	sizeLastCalculatedAt: string | null;
};

export type Notebook = {
	id: string;
	journalId: string;
	name: string;
	icon: string;
	totalSize: number;
	sizeLastCalculatedAt: string | null;
};

export type SaveStatus = 'saved' | 'saving' | 'error';

export type Entry = {
	id: string;
	notebookId: string;
	title: string;
	content: string;
	contentLoaded: boolean;
	saveStatus: SaveStatus;
	entryDate: string;
	createdAt: string;
	updatedAt: string | null;
};

export const NOTEBOOK_ICONS = [
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
const SAVE_DELAY_MS = 2_000;

export function isoDate(date = new Date()) {
	const year = date.getFullYear();
	const month = String(date.getMonth() + 1).padStart(2, '0');
	const day = String(date.getDate()).padStart(2, '0');
	return `${year}-${month}-${day}`;
}

export function parseIsoDate(iso: string) {
	const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso);
	if (!match) return new Date();
	return new Date(Number(match[1]), Number(match[2]) - 1, Number(match[3]));
}

export function formatEntryDate(date = new Date()) {
	return `${date.getDate()} ${MONTHS[date.getMonth()]} ${date.getFullYear()}`;
}

export function formatBytes(bytes: number) {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function notebookIcon(icon: string | null): string {
	return icon && (NOTEBOOK_ICONS as readonly string[]).includes(icon) ? icon : NOTEBOOK_ICONS[0];
}

function mapJournal(row: api.ApiJournal): Journal {
	return {
		id: row.id,
		name: row.name,
		keySalt: row.key_salt,
		encryptedDek: row.encrypted_dek,
		passphraseHint: row.passphrase_hint,
		mask: row.mask ?? false,
		totalSize: row.total_journal_size ?? 0,
		sizeLastCalculatedAt: row.size_last_calculated_at
	};
}

function mapNotebook(row: api.ApiNotebook, dek: Uint8Array): Notebook {
	let name = 'Unable to decrypt';
	try {
		name = decryptText(dek, base64ToBytes(row.name), 'notebook.name');
	} catch {
		// Keep the fallback label when ciphertext does not match this DEK.
	}
	return {
		id: row.id,
		journalId: row.journal_id,
		name,
		icon: notebookIcon(row.icon),
		totalSize: row.total_notebook_size ?? 0,
		sizeLastCalculatedAt: row.size_last_calculated_at
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
		notebookId: row.notebook_id,
		title: previous?.contentLoaded ? previous.title : title,
		content: previous?.contentLoaded ? previous.content : '',
		contentLoaded: previous?.contentLoaded ?? false,
		saveStatus: previous?.saveStatus ?? 'saved',
		entryDate: row.entry_date,
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

class JournalStore {
	journals = $state.raw<Journal[]>([]);
	notebooks = $state.raw<Notebook[]>([]);
	entries = $state.raw<Entry[]>([]);
	loading = $state(false);
	loadingMore = $state(false);
	hasMore = $state(false);
	notebooksLoadedFor = $state<string | null>(null);
	entryOrder = $state<api.EntryOrder>('desc');
	error = $state<string | null>(null);
	#saveTimers = new Map<string, ReturnType<typeof setTimeout>>();
	#dirty = new Set<string>();
	#contentLoads = new Map<string, Promise<Entry | null>>();
	#nextCursor: string | null = null;

	getJournal(id: string) {
		return this.journals.find((item) => item.id === id);
	}

	notebooksFor(journalId: string) {
		return this.notebooks.filter((notebook) => notebook.journalId === journalId);
	}

	entriesFor(notebookId: string) {
		const sign = this.entryOrder === 'asc' ? 1 : -1;
		return this.entries
			.filter((entry) => entry.notebookId === notebookId)
			.toSorted((a, b) => sign * a.id.localeCompare(b.id));
	}

	entry(id: string) {
		return this.entries.find((entry) => entry.id === id);
	}

	clearJournal() {
		this.notebooks = [];
		this.entries = [];
		this.error = null;
		this.hasMore = false;
		this.loadingMore = false;
		this.notebooksLoadedFor = null;
		this.#nextCursor = null;
	}

	async loadJournals() {
		this.loading = true;
		this.error = null;
		try {
			this.journals = (await api.listJournals()).map(mapJournal);
		} catch (error) {
			this.error = error instanceof Error ? error.message : 'Could not load journals.';
			throw error;
		} finally {
			this.loading = false;
		}
	}

	async loadJournal(id: string) {
		const existing = this.getJournal(id);
		if (existing) return existing;
		this.loading = true;
		this.error = null;
		try {
			const row = mapJournal(await api.getJournal(id));
			this.journals = [row, ...this.journals.filter((item) => item.id !== row.id)];
			return row;
		} catch (error) {
			if (error instanceof ApiError && error.status === 404) return null;
			this.error = error instanceof Error ? error.message : 'Could not load this journal.';
			throw error;
		} finally {
			this.loading = false;
		}
	}

	async createJournal(name: string, passphrase: string, passphraseHint = '') {
		const secrets = await createJournalSecrets(passphrase);
		const hint = passphraseHint.trim();
		try {
			const journal = mapJournal(
				await api.createJournal({
					name,
					key_salt: bytesToBase64(secrets.keySalt),
					encrypted_dek: bytesToBase64(secrets.encryptedDek),
					...(hint ? { passphrase_hint: hint } : {})
				})
			);
			this.journals = [journal, ...this.journals];
			session.unlock(journal.id, secrets.dek);
			try {
				const notebook = mapNotebook(
					await api.createNotebook(journal.id, {
						name: bytesToBase64(encryptText(secrets.dek, 'Notes', 'notebook.name')),
						icon: NOTEBOOK_ICONS[0]
					}),
					secrets.dek
				);
				this.notebooks = [...this.notebooks, notebook];
			} catch {
				// The journal is usable; a notebook can be added after opening it.
			}
			return journal;
		} catch (error) {
			secrets.dek.fill(0);
			throw error;
		}
	}

	async updateJournal(
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
		const journal = mapJournal(await api.updateJournal(id, body));
		this.journals = this.journals.map((item) => (item.id === id ? journal : item));
		return journal;
	}

	async changeJournalPassphrase(id: string, currentPassphrase: string, nextPassphrase: string) {
		const journal = this.getJournal(id);
		if (!journal) throw new Error('Journal not found.');
		const sessionDek = session.requireDek();
		const dek = await unlockDek(
			currentPassphrase,
			base64ToBytes(journal.keySalt),
			base64ToBytes(journal.encryptedDek)
		);
		try {
			if (!equalBytes(dek, sessionDek)) throw new CryptoError('Wrong passphrase');
		} finally {
			dek.fill(0);
		}
		const wrapped = await wrapDek(nextPassphrase, sessionDek);
		return this.updateJournal(id, {
			keySalt: bytesToBase64(wrapped.keySalt),
			encryptedDek: bytesToBase64(wrapped.encryptedDek)
		});
	}

	async unlockJournal(journal: Journal, passphrase: string) {
		const dek = await unlockDek(
			passphrase,
			base64ToBytes(journal.keySalt),
			base64ToBytes(journal.encryptedDek)
		);
		session.unlock(journal.id, dek);
	}

	async loadNotebooks(journalId: string) {
		const dek = session.dek;
		if (!dek) return;
		this.loading = true;
		this.error = null;
		try {
			this.notebooks = (await api.listNotebooks(journalId)).map((row) => mapNotebook(row, dek));
			this.notebooksLoadedFor = journalId;
		} catch (error) {
			this.error = error instanceof Error ? error.message : 'Could not load notebooks.';
			throw error;
		} finally {
			this.loading = false;
		}
	}

	async createNotebook(journalId: string, name: string, icon: string) {
		const dek = session.requireDek();
		const notebook = mapNotebook(
			await api.createNotebook(journalId, {
				name: bytesToBase64(encryptText(dek, name, 'notebook.name')),
				icon
			}),
			dek
		);
		this.notebooks = [...this.notebooks, notebook];
		return notebook;
	}

	async updateNotebook(id: string, patch: { name?: string; icon?: string }) {
		const dek = session.requireDek();
		const body: { name?: string; icon?: string } = {};
		if (patch.name !== undefined) {
			body.name = bytesToBase64(encryptText(dek, patch.name, 'notebook.name'));
		}
		if (patch.icon !== undefined) body.icon = patch.icon;
		const notebook = mapNotebook(await api.updateNotebook(id, body), dek);
		this.notebooks = this.notebooks.map((item) => (item.id === id ? notebook : item));
	}

	async deleteNotebook(id: string) {
		await this.flush();
		await api.deleteNotebook(id);
		this.notebooks = this.notebooks.filter((notebook) => notebook.id !== id);
		this.entries = this.entries.filter((entry) => entry.notebookId !== id);
	}

	async loadEntries(notebookId: string, opts: { cursor?: string | null; append?: boolean } = {}) {
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
			const page = await api.listEntries(notebookId, {
				cursor: opts.cursor,
				order: this.entryOrder
			});
			const previous = new Map(this.entries.map((entry) => [entry.id, entry]));
			const incoming = page.entries.map((row) => mapEntrySummary(row, dek, previous.get(row.id)));
			if (append) {
				const seen = new Set(
					this.entries.filter((entry) => entry.notebookId === notebookId).map((entry) => entry.id)
				);
				this.entries = [...this.entries, ...incoming.filter((entry) => !seen.has(entry.id))];
			} else {
				const incomingIds = new Set(incoming.map((entry) => entry.id));
				const kept = this.entries.filter(
					(entry) =>
						entry.notebookId === notebookId &&
						!incomingIds.has(entry.id) &&
						(entry.contentLoaded || this.#dirty.has(entry.id))
				);
				this.entries = [
					...this.entries.filter((entry) => entry.notebookId !== notebookId),
					...incoming,
					...kept
				];
			}
			this.#nextCursor = page.next_cursor;
			this.hasMore = page.next_cursor != null;
		} catch (error) {
			this.error = error instanceof Error ? error.message : 'Could not load notes.';
		} finally {
			this.loadingMore = false;
		}
	}

	async loadMore(notebookId: string) {
		if (!this.#nextCursor || this.loadingMore) return;
		await this.loadEntries(notebookId, { cursor: this.#nextCursor, append: true });
	}

	async loadEntry(id: string): Promise<Entry | null> {
		const current = this.entry(id);
		if (current?.contentLoaded) return current;
		const pending = this.#contentLoads.get(id);
		if (pending) return pending;
		const promise = this.#fetchEntry(id);
		this.#contentLoads.set(id, promise);
		try {
			return await promise;
		} finally {
			this.#contentLoads.delete(id);
		}
	}

	async #fetchEntry(id: string): Promise<Entry | null> {
		const dek = session.dek;
		if (!dek) return null;
		try {
			const row = await api.getEntry(id);
			const previous = this.entry(id);
			if (previous && this.#dirty.has(id)) {
				this.entries = this.entries.map((entry) =>
					entry.id === id ? { ...entry, contentLoaded: true } : entry
				);
				return this.entry(id) ?? previous;
			}
			const loaded = mapFullEntry(row, dek, previous);
			this.entries = previous
				? this.entries.map((entry) => (entry.id === id ? loaded : entry))
				: [...this.entries, loaded];
			return loaded;
		} catch (error) {
			if (error instanceof ApiError && error.status === 404) return null;
			this.error = error instanceof Error ? error.message : 'Could not load the note.';
			throw error;
		}
	}

	async createEntry(notebookId: string, title = formatEntryDate()) {
		const dek = session.requireDek();
		const row = await api.createEntry(notebookId, {
			title: bytesToBase64(encryptText(dek, title, 'entry.title')),
			content: bytesToBase64(encryptText(dek, '', 'entry.content')),
			entry_date: isoDate()
		});
		const entry = mapFullEntry(row, dek);
		const others = this.entries.filter((item) => item.notebookId !== notebookId);
		const loaded = this.entries.filter((item) => item.notebookId === notebookId);
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

	async updateEntryDate(id: string, entryDate: string) {
		const current = this.entry(id);
		if (!current || current.entryDate === entryDate) return;
		this.entries = this.entries.map((entry) => (entry.id === id ? { ...entry, entryDate } : entry));
		const keepSaving = this.#dirty.has(id) || this.#saveTimers.has(id);
		if (!keepSaving) this.#setSaveStatus(id, 'saving');
		try {
			const row = await api.updateEntry(id, { entry_date: entryDate });
			this.entries = this.entries.map((item) =>
				item.id === id
					? {
							...item,
							entryDate: row.entry_date,
							updatedAt: row.updated_at,
							saveStatus:
								this.#dirty.has(id) || this.#saveTimers.has(id) ? item.saveStatus : 'saved'
						}
					: item
			);
		} catch (error) {
			this.#setSaveStatus(id, 'error');
			this.error = error instanceof Error ? error.message : 'Could not save.';
		}
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

export const journal = new JournalStore();
