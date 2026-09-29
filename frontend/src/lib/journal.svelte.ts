import * as api from '$lib/api';
import {
	base64ToBytes,
	bytesToBase64,
	createJournalSecrets,
	CryptoError,
	decryptBytes,
	decryptText,
	encryptText,
	equalBytes,
	unlockDek,
	wrapDek,
	type Purpose
} from '$lib/crypto';
import { ApiError } from '$lib/http';
import { MIN_PASSPHRASE_LEN, minPassphraseLengthError } from '$lib/passphrase';
import { session } from '$lib/session.svelte';
import { journalTheme, type JournalTheme } from '$lib/theme';

export type Journal = {
	id: string;
	name: string;
	keySalt: string;
	encryptedDek: string;
	passphraseHint: string | null;
	mask: boolean;
	theme: JournalTheme;
	totalSize: number;
	sizeLastCalculatedAt: string | null;
};

export type Notebook = {
	id: string;
	journalId: string;
	name: string;
	icon: string;
	templateEntryContent: string | null;
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
const SAVE_DELAY_MS = 1_000;

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
		theme: journalTheme(row.theme),
		totalSize: row.total_journal_size ?? 0,
		sizeLastCalculatedAt: row.size_last_calculated_at
	};
}

const textDecoder = new TextDecoder();

function openField(dek: Uint8Array, blob: string, purpose: Purpose, id: string) {
	try {
		return decryptText(dek, base64ToBytes(blob), purpose, id);
	} catch {
		return null;
	}
}

function openTemplate(dek: Uint8Array, blob: string, id: string) {
	try {
		const text = textDecoder.decode(
			decryptBytes(dek, base64ToBytes(blob), 'notebook.template', id)
		);
		return text || null;
	} catch {
		return null;
	}
}

function mapNotebook(
	row: api.ApiNotebook,
	dek: Uint8Array
): { notebook: Notebook; legacy: boolean } {
	const opened = openField(dek, row.name, 'notebook.name', row.id);
	return {
		notebook: {
			id: row.id,
			journalId: row.journal_id,
			name: opened?.text ?? 'Unable to decrypt',
			icon: notebookIcon(row.icon),
			templateEntryContent: row.template_entry_content
				? openTemplate(dek, row.template_entry_content, row.id)
				: null,
			totalSize: row.total_notebook_size ?? 0,
			sizeLastCalculatedAt: row.size_last_calculated_at
		},
		legacy: opened?.legacy ?? false
	};
}

function mapEntrySummary(
	row: api.ApiEntrySummary,
	dek: Uint8Array,
	previous?: Entry
): { entry: Entry; legacyTitle: boolean } {
	const opened = openField(dek, row.title, 'entry.title', row.id);
	const title = opened?.text ?? 'Unable to decrypt';
	const keepLocal = previous?.contentLoaded === true;
	return {
		entry: {
			id: row.id,
			notebookId: row.notebook_id,
			title: keepLocal && previous ? previous.title : title,
			content: keepLocal && previous ? previous.content : '',
			contentLoaded: previous?.contentLoaded ?? false,
			saveStatus: previous?.saveStatus ?? 'saved',
			entryDate: row.entry_date,
			createdAt: row.created_at,
			updatedAt: row.updated_at
		},
		legacyTitle: keepLocal ? false : (opened?.legacy ?? false)
	};
}

function mapFullEntry(
	row: api.ApiEntry,
	dek: Uint8Array,
	previous?: Entry
): { entry: Entry; legacyTitle: boolean; legacyContent: boolean } {
	const summary = mapEntrySummary(row, dek, previous);
	try {
		const title = decryptText(dek, base64ToBytes(row.title), 'entry.title', row.id);
		const content = decryptText(dek, base64ToBytes(row.content), 'entry.content', row.id);
		return {
			entry: {
				...summary.entry,
				title: title.text,
				content: content.text,
				contentLoaded: true,
				saveStatus: previous?.saveStatus === 'saving' ? 'saving' : 'saved'
			},
			legacyTitle: title.legacy,
			legacyContent: content.legacy
		};
	} catch {
		return {
			entry: { ...summary.entry, contentLoaded: true },
			legacyTitle: false,
			legacyContent: false
		};
	}
}

class JournalStore {
	journals = $state.raw<Journal[]>([]);
	notebooks = $state.raw<Notebook[]>([]);
	entries = $state.raw<Entry[]>([]);
	loading = $state(false);
	loadingNotebooks = $state(false);
	loadingEntries = $state(false);
	loadingMore = $state(false);
	hasMore = $state(false);
	notebooksLoadedFor = $state<string | null>(null);
	entryOrder = $state<api.EntryOrder>('desc');
	error = $state<string | null>(null);
	#saveTimers = new Map<string, ReturnType<typeof setTimeout>>();
	#dirty = new Set<string>();
	#inflight = new Map<string, number>();
	#writes = new Map<string, Promise<void>>();
	#rebinds = new Set<string>();
	#revs = new Map<string, number>();
	#contentLoads = new Map<string, Promise<Entry | null>>();
	#nextCursor: string | null = null;
	#entriesLoad = 0;
	#notebooksLoad = 0;
	#view = 0;

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
		for (const timer of this.#saveTimers.values()) clearTimeout(timer);
		this.#saveTimers.clear();
		this.#dirty.clear();
		this.#inflight.clear();
		this.#writes.clear();
		this.#rebinds.clear();
		this.#revs.clear();
		this.#contentLoads.clear();
		this.#view += 1;
		this.notebooks = [];
		this.entries = [];
		this.error = null;
		this.hasMore = false;
		this.loadingNotebooks = false;
		this.loadingEntries = false;
		this.loadingMore = false;
		this.notebooksLoadedFor = null;
		this.#nextCursor = null;
		this.#entriesLoad += 1;
		this.#notebooksLoad += 1;
	}

	#bump(id: string) {
		this.#revs.set(id, (this.#revs.get(id) ?? 0) + 1);
	}

	#sealed(dek: Uint8Array, text: string, purpose: Purpose, id: string) {
		return bytesToBase64(encryptText(dek, text, purpose, id));
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

	async createJournal(
		name: string,
		passphrase: string,
		passphraseHint = '',
		theme: JournalTheme = ''
	) {
		if (passphrase.length < MIN_PASSPHRASE_LEN) {
			throw new CryptoError(minPassphraseLengthError('Passphrase'));
		}
		const secrets = await createJournalSecrets(passphrase);
		const hint = passphraseHint.trim();
		try {
			const journal = mapJournal(
				await api.createJournal({
					name,
					key_salt: bytesToBase64(secrets.keySalt),
					encrypted_dek: bytesToBase64(secrets.encryptedDek),
					...(hint ? { passphrase_hint: hint } : {}),
					...(theme ? { theme } : {})
				})
			);
			this.journals = [journal, ...this.journals];
			session.unlock(journal.id, secrets.dek);
			try {
				const created = mapNotebook(
					await api.createNotebook(journal.id, {
						name: bytesToBase64(encryptText(secrets.dek, 'Notes', 'notebook.name')),
						icon: NOTEBOOK_ICONS[0]
					}),
					secrets.dek
				);
				if (created.legacy) await this.#rebindNotebook(created.notebook);
				this.notebooks = [...this.notebooks, created.notebook];
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
			theme?: JournalTheme;
			keySalt?: string;
			encryptedDek?: string;
		}
	) {
		const body: {
			name?: string;
			passphrase_hint?: string;
			mask?: boolean;
			theme?: string;
			key_salt?: string;
			encrypted_dek?: string;
		} = {};
		if (patch.name !== undefined) body.name = patch.name;
		if (patch.passphraseHint !== undefined) body.passphrase_hint = patch.passphraseHint;
		if (patch.mask !== undefined) body.mask = patch.mask;
		if (patch.theme !== undefined) body.theme = patch.theme;
		if (patch.keySalt !== undefined) body.key_salt = patch.keySalt;
		if (patch.encryptedDek !== undefined) body.encrypted_dek = patch.encryptedDek;
		const journal = mapJournal(await api.updateJournal(id, body));
		this.journals = this.journals.map((item) => (item.id === id ? journal : item));
		return journal;
	}

	async changeJournalPassphrase(id: string, currentPassphrase: string, nextPassphrase: string) {
		if (nextPassphrase.length < MIN_PASSPHRASE_LEN) {
			throw new CryptoError(minPassphraseLengthError('New passphrase'));
		}
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
		this.loadingNotebooks = true;
		this.#notebooksLoad += 1;
		const load = this.#notebooksLoad;
		this.error = null;
		try {
			const mapped = (await api.listNotebooks(journalId)).map((row) => mapNotebook(row, dek));
			if (load !== this.#notebooksLoad) return;
			this.notebooks = mapped.map((item) => item.notebook);
			this.notebooksLoadedFor = journalId;
			for (const item of mapped) {
				if (item.legacy) void this.#rebindNotebook(item.notebook);
			}
		} catch (error) {
			this.error = error instanceof Error ? error.message : 'Could not load notebooks.';
			throw error;
		} finally {
			if (load === this.#notebooksLoad) this.loadingNotebooks = false;
		}
	}

	async createNotebook(journalId: string, name: string, icon: string) {
		const dek = session.requireDek();
		const mapped = mapNotebook(
			await api.createNotebook(journalId, {
				name: bytesToBase64(encryptText(dek, name, 'notebook.name')),
				icon
			}),
			dek
		);
		if (mapped.legacy) await this.#rebindNotebook(mapped.notebook);
		this.notebooks = [...this.notebooks, mapped.notebook];
		return mapped.notebook;
	}

	async updateNotebook(
		id: string,
		patch: { name?: string; icon?: string; templateEntryContent?: string | null }
	) {
		this.#bump(id);
		const dek = session.requireDek();
		const body: { name?: string; icon?: string; template_entry_content?: string | null } = {};
		if (patch.name !== undefined) {
			body.name = this.#sealed(dek, patch.name, 'notebook.name', id);
		}
		if (patch.icon !== undefined) body.icon = patch.icon;
		if (patch.templateEntryContent !== undefined) {
			body.template_entry_content = patch.templateEntryContent
				? this.#sealed(dek, patch.templateEntryContent, 'notebook.template', id)
				: null;
		}
		const mapped = mapNotebook(await api.updateNotebook(id, body), dek);
		this.notebooks = this.notebooks.map((item) => (item.id === id ? mapped.notebook : item));
		if (mapped.legacy) void this.#rebindNotebook(mapped.notebook);
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
			this.loadingEntries = true;
			this.#entriesLoad += 1;
		}
		const load = this.#entriesLoad;
		this.error = null;
		try {
			const page = await api.listEntries(notebookId, {
				cursor: opts.cursor,
				order: this.entryOrder
			});
			if (!append && load !== this.#entriesLoad) return;
			const previous = new Map(this.entries.map((entry) => [entry.id, entry]));
			const incoming = page.entries.map((row) => mapEntrySummary(row, dek, previous.get(row.id)));
			const incomingEntries = incoming.map((item) => item.entry);
			if (append) {
				const seen = new Set(
					this.entries.filter((entry) => entry.notebookId === notebookId).map((entry) => entry.id)
				);
				this.entries = [...this.entries, ...incomingEntries.filter((entry) => !seen.has(entry.id))];
			} else {
				const incomingIds = new Set(incomingEntries.map((entry) => entry.id));
				const kept = this.entries.filter(
					(entry) =>
						entry.notebookId === notebookId &&
						!incomingIds.has(entry.id) &&
						(entry.contentLoaded || this.#dirty.has(entry.id))
				);
				this.entries = [
					...this.entries.filter((entry) => entry.notebookId !== notebookId),
					...incomingEntries,
					...kept
				];
			}
			this.#nextCursor = page.next_cursor;
			this.hasMore = page.next_cursor != null;
			for (const item of incoming) {
				if (item.legacyTitle) void this.#rebindEntry(item.entry, { title: true });
			}
		} catch (error) {
			this.error = error instanceof Error ? error.message : 'Could not load notes.';
		} finally {
			if (append) this.loadingMore = false;
			else if (load === this.#entriesLoad) this.loadingEntries = false;
		}
	}

	async loadMore(notebookId: string) {
		if (!this.#nextCursor || this.loadingMore) return;
		await this.loadEntries(notebookId, { cursor: this.#nextCursor, append: true });
	}

	async loadEntry(id: string, opts: { force?: boolean } = {}): Promise<Entry | null> {
		const pending = this.#contentLoads.get(id);
		if (pending) return pending;
		if (!opts.force && this.entry(id)?.contentLoaded) return this.entry(id) ?? null;
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
		const view = this.#view;
		const keepLocal = () => this.#dirty.has(id) || this.#inflight.has(id);
		const previous = this.entry(id);
		if (previous && keepLocal()) return this.#markLoaded(id) ?? previous;
		try {
			const row = await api.getEntry(id);
			if (view !== this.#view || !session.dek) return null;
			const latest = this.entry(id);
			if (latest && keepLocal()) return this.#markLoaded(id) ?? latest;
			const mapped = mapFullEntry(row, dek, latest);
			this.entries = latest
				? this.entries.map((entry) => (entry.id === id ? mapped.entry : entry))
				: [...this.entries, mapped.entry];
			if (mapped.legacyTitle || mapped.legacyContent) {
				await this.#rebindEntry(mapped.entry, {
					title: mapped.legacyTitle,
					content: mapped.legacyContent
				});
			}
			if (view !== this.#view) return null;
			return this.entry(id) ?? mapped.entry;
		} catch (error) {
			if (error instanceof ApiError && error.status === 404) return null;
			if (view !== this.#view) return null;
			this.error = error instanceof Error ? error.message : 'Could not load the note.';
			throw error;
		}
	}

	#markLoaded(id: string) {
		const current = this.entry(id);
		if (!current || current.contentLoaded) return current;
		this.entries = this.entries.map((entry) =>
			entry.id === id ? { ...entry, contentLoaded: true } : entry
		);
		return this.entry(id);
	}

	async createEntry(notebookId: string, title = formatEntryDate()) {
		const dek = session.requireDek();
		const template = this.notebooks.find((item) => item.id === notebookId)?.templateEntryContent;
		const row = await api.createEntry(notebookId, {
			title: bytesToBase64(encryptText(dek, title, 'entry.title')),
			content: bytesToBase64(encryptText(dek, template || '', 'entry.content')),
			entry_date: isoDate()
		});
		const mapped = mapFullEntry(row, dek);
		const entry = mapped.entry;
		const others = this.entries.filter((item) => item.notebookId !== notebookId);
		const loaded = this.entries.filter((item) => item.notebookId === notebookId);
		const sign = this.entryOrder === 'asc' ? 1 : -1;
		this.entries = [
			...others,
			...[...loaded, entry].toSorted((a, b) => sign * a.id.localeCompare(b.id))
		];
		if (mapped.legacyTitle || mapped.legacyContent) {
			await this.#rebindEntry(entry, {
				title: mapped.legacyTitle,
				content: mapped.legacyContent
			});
		}
		return this.entry(entry.id) ?? entry;
	}

	updateEntry(id: string, patch: { title?: string; content?: string }) {
		this.#bump(id);
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
			await api.updateEntry(id, { entry_date: entryDate });
			this.entries = this.entries.map((item) =>
				item.id === id
					? {
							...item,
							updatedAt: new Date().toISOString(),
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

	async flush(id?: string, opts?: { keepalive?: boolean }) {
		const run = (pendingId: string) => {
			this.#cancelSave(pendingId, false);
			if (!this.#dirty.has(pendingId)) return Promise.resolve();
			return this.#save(pendingId, opts);
		};
		if (id) {
			await run(id);
			return;
		}
		const ids = [...new Set([...this.#dirty, ...this.#saveTimers.keys()])];
		if (opts?.keepalive) {
			const pending = [...new Set([...ids, ...this.#inflight.keys()])];
			for (const pendingId of pending) this.#dirty.add(pendingId);
			await Promise.all(pending.map((pendingId) => run(pendingId)));
			return;
		}
		for (const pendingId of ids) await run(pendingId);
	}

	#cancelSave(id: string, forgetDirty = true) {
		const timer = this.#saveTimers.get(id);
		if (timer) clearTimeout(timer);
		this.#saveTimers.delete(id);
		if (forgetDirty) this.#dirty.delete(id);
	}

	async #save(id: string, opts?: { keepalive?: boolean }) {
		if (opts?.keepalive) {
			await this.#writeEntry(id, opts);
			return;
		}
		const previous = this.#writes.get(id) ?? Promise.resolve();
		const current = previous.catch(() => undefined).then(() => this.#writeEntry(id));
		this.#writes.set(id, current);
		try {
			await current;
		} finally {
			if (this.#writes.get(id) === current) this.#writes.delete(id);
		}
	}

	async #writeEntry(id: string, opts?: { keepalive?: boolean }) {
		const view = this.#view;
		const dek = session.dek;
		const entry = this.entry(id);
		if (!dek || !entry || !this.#dirty.has(id)) return;
		const title = entry.title;
		const content = entry.content;
		this.#dirty.delete(id);
		this.#inflight.set(id, (this.#inflight.get(id) ?? 0) + 1);
		this.#setSaveStatus(id, 'saving');
		try {
			await api.updateEntry(
				id,
				{
					title: this.#sealed(dek, title, 'entry.title', id),
					content: this.#sealed(dek, content, 'entry.content', id)
				},
				opts
			);
			if (view !== this.#view || !session.dek || this.#dirty.has(id)) return;
			this.entries = this.entries.map((item) =>
				item.id === id
					? {
							...item,
							updatedAt: new Date().toISOString(),
							saveStatus: 'saved'
						}
					: item
			);
		} catch (error) {
			if (view !== this.#view || !session.dek) return;
			this.#dirty.add(id);
			this.#setSaveStatus(id, 'error');
			this.error = error instanceof Error ? error.message : 'Could not save.';
		} finally {
			const left = (this.#inflight.get(id) ?? 1) - 1;
			if (left <= 0) this.#inflight.delete(id);
			else this.#inflight.set(id, left);
		}
	}

	async #rebindNotebook(notebook: Notebook) {
		const slot = `notebook:${notebook.id}`;
		if (this.#rebinds.has(slot) || notebook.name === 'Unable to decrypt' || !session.dek) return;
		this.#rebinds.add(slot);
		const rev = this.#revs.get(notebook.id) ?? 0;
		const view = this.#view;
		const dek = session.dek;
		try {
			await api.updateNotebook(notebook.id, {
				name: this.#sealed(dek, notebook.name, 'notebook.name', notebook.id)
			});
			if (view !== this.#view || !session.dek || (this.#revs.get(notebook.id) ?? 0) === rev) return;
			const current = this.notebooks.find((item) => item.id === notebook.id);
			const dekNow = session.dek;
			if (!current || !dekNow) return;
			await api.updateNotebook(notebook.id, {
				name: this.#sealed(dekNow, current.name, 'notebook.name', notebook.id)
			});
		} catch {
			this.#rebinds.delete(slot);
		}
	}

	async #rebindEntry(entry: Entry, which: { title?: boolean; content?: boolean }) {
		const title = which.title === true && entry.title !== 'Unable to decrypt';
		const content = which.content === true;
		if (!title && !content) return;
		const slot = `entry:${entry.id}:${title ? 't' : ''}${content ? 'c' : ''}`;
		if (this.#rebinds.has(slot) || !session.dek) return;
		if (
			this.#dirty.has(entry.id) ||
			this.#inflight.has(entry.id) ||
			this.#saveTimers.has(entry.id)
		) {
			return;
		}
		this.#rebinds.add(slot);
		const rev = this.#revs.get(entry.id) ?? 0;
		const view = this.#view;
		const dek = session.dek;
		const body: { title?: string; content?: string } = {};
		if (title) body.title = this.#sealed(dek, entry.title, 'entry.title', entry.id);
		if (content) body.content = this.#sealed(dek, entry.content, 'entry.content', entry.id);
		try {
			await api.updateEntry(entry.id, body);
			if (view !== this.#view || !session.dek) return;
			const edited =
				(this.#revs.get(entry.id) ?? 0) !== rev ||
				this.#dirty.has(entry.id) ||
				this.#inflight.has(entry.id);
			if (!edited) return;
			const current = this.entry(entry.id);
			if (!current?.contentLoaded) return;
			this.#dirty.add(entry.id);
			await this.#save(entry.id);
		} catch {
			this.#rebinds.delete(slot);
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

session.setAfterClear(() => journal.clearJournal());
