import { apiFetch } from './http';

export const login = () => '/login';
export const journals = () => '/journals';
export const journal = (id: string) => `/journals/${id}`;
export const journalNotebook = (journalId: string, notebookId: string) =>
	`/journals/${journalId}/notebook/${notebookId}`;
export const journalEntry = (journalId: string, notebookId: string, entryId: string) =>
	`/journals/${journalId}/notebook/${notebookId}/entry/${entryId}`;

export const health = () => '/health';
export const google = (next?: string) =>
	next ? `/api/auth/google?next=${encodeURIComponent(next)}` : '/api/auth/google';
export const googleCallback = () => '/api/auth/google/callback';
export const logout = () => '/api/auth/logout';
export const passwordLogin = () => '/api/auth/login';
export const passwordSignup = () => '/api/auth/signup';
export const user = () => '/api/user';
export const journalCollection = () => '/api/journals';
export const journalItem = (id: string) => `/api/journals/${id}`;
export const journalNotebooks = (journalId: string) => `/api/journals/${journalId}/notebooks`;
export const notebook = (id: string) => `/api/notebooks/${id}`;
export const notebookEntries = (notebookId: string) => `/api/notebooks/${notebookId}/entries`;
export const entry = (id: string) => `/api/entries/${id}`;

export type ApiUser = {
	id: string;
	name: string;
	username: string | null;
	email: string | null;
	google_email: string | null;
	google_avatar_url: string | null;
};

export type ApiJournal = {
	id: string;
	user_id: string;
	name: string;
	key_salt: string;
	encrypted_dek: string;
	passphrase_hint: string | null;
	mask: boolean;
	total_journal_size: number;
	size_last_calculated_at: string | null;
	created_at: string;
	updated_at: string | null;
};

export type ApiNotebook = {
	id: string;
	journal_id: string;
	name: string;
	icon: string | null;
	total_notebook_size: number;
	size_last_calculated_at: string | null;
	created_at: string;
	updated_at: string | null;
};

export type ApiEntrySummary = {
	id: string;
	notebook_id: string;
	title: string;
	entry_date: string;
	created_at: string;
	updated_at: string | null;
};

export type ApiEntry = ApiEntrySummary & {
	content: string;
};

export type EntryOrder = 'asc' | 'desc';

export type ApiEntryPage = {
	entries: ApiEntrySummary[];
	next_cursor: string | null;
};

export const getUser = () => apiFetch<ApiUser>(user());
export const logoutUser = () => apiFetch<void>(logout(), { method: 'POST' });
export const loginWithPassword = (body: { username: string; password: string }) =>
	apiFetch<ApiUser>(passwordLogin(), {
		method: 'POST',
		body: JSON.stringify(body)
	});
export const signupWithPassword = (body: { username: string; password: string }) =>
	apiFetch<ApiUser>(passwordSignup(), {
		method: 'POST',
		body: JSON.stringify(body)
	});

export const listJournals = () => apiFetch<ApiJournal[]>(journalCollection());
export const createJournal = (body: {
	name: string;
	key_salt: string;
	encrypted_dek: string;
	passphrase_hint?: string;
}) =>
	apiFetch<ApiJournal>(journalCollection(), {
		method: 'POST',
		body: JSON.stringify(body)
	});
export const updateJournal = (
	id: string,
	body: {
		name?: string;
		passphrase_hint?: string;
		mask?: boolean;
		key_salt?: string;
		encrypted_dek?: string;
	}
) =>
	apiFetch<ApiJournal>(journalItem(id), {
		method: 'PATCH',
		body: JSON.stringify(body)
	});

export const listNotebooks = (journalId: string) =>
	apiFetch<ApiNotebook[]>(journalNotebooks(journalId));
export const createNotebook = (journalId: string, body: { name: string; icon?: string }) =>
	apiFetch<ApiNotebook>(journalNotebooks(journalId), {
		method: 'POST',
		body: JSON.stringify(body)
	});
export const updateNotebook = (id: string, body: { name?: string; icon?: string }) =>
	apiFetch<ApiNotebook>(notebook(id), {
		method: 'PATCH',
		body: JSON.stringify(body)
	});
export const deleteNotebook = (id: string) => apiFetch<void>(notebook(id), { method: 'DELETE' });

export const listEntries = (
	notebookId: string,
	params: { cursor?: string | null; order?: EntryOrder; limit?: number } = {}
) => {
	const search = new URLSearchParams();
	if (params.cursor) search.set('cursor', params.cursor);
	if (params.order) search.set('order', params.order);
	if (params.limit) search.set('limit', String(params.limit));
	const query = search.toString();
	return apiFetch<ApiEntryPage>(
		query ? `${notebookEntries(notebookId)}?${query}` : notebookEntries(notebookId)
	);
};
export const getEntry = (id: string) => apiFetch<ApiEntry>(entry(id));
export const createEntry = (
	notebookId: string,
	body: { title: string; content: string; entry_date?: string }
) =>
	apiFetch<ApiEntry>(notebookEntries(notebookId), {
		method: 'POST',
		body: JSON.stringify(body)
	});
export const updateEntry = (
	id: string,
	body: { title?: string; content?: string; entry_date?: string }
) =>
	apiFetch<ApiEntry>(entry(id), {
		method: 'PATCH',
		body: JSON.stringify(body)
	});
export const deleteEntry = (id: string) => apiFetch<void>(entry(id), { method: 'DELETE' });
