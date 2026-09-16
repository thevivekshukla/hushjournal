import { apiFetch } from './http';

export const login = () => '/login';
export const workspaces = () => '/workspaces';
export const workspace = (id: string) => `/w/${id}`;

export const health = () => '/health';
export const google = (next?: string) =>
	next ? `/api/auth/google?next=${encodeURIComponent(next)}` : '/api/auth/google';
export const googleCallback = () => '/api/auth/google/callback';
export const logout = () => '/api/auth/logout';
export const user = () => '/api/user';
export const workspaceCollection = () => '/api/workspaces';
export const workspaceItem = (id: string) => `/api/workspaces/${id}`;
export const workspaceShelves = (workspaceId: string) => `/api/workspaces/${workspaceId}/shelves`;
export const shelf = (id: string) => `/api/shelves/${id}`;
export const shelfEntries = (shelfId: string) => `/api/shelves/${shelfId}/entries`;
export const entry = (id: string) => `/api/entries/${id}`;

export type ApiUser = {
	id: string;
	name: string;
	email: string | null;
	google_email: string | null;
	google_avatar_url: string | null;
};

export type ApiWorkspace = {
	id: string;
	user_id: string;
	name: string;
	key_salt: string;
	encrypted_dek: string;
	created_at: string;
	updated_at: string | null;
};

export type ApiShelf = {
	id: string;
	workspace_id: string;
	name: string;
	icon: string | null;
	created_at: string;
	updated_at: string | null;
};

export type ApiEntrySummary = {
	id: string;
	shelf_id: string;
	title: string;
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

export const listWorkspaces = () => apiFetch<ApiWorkspace[]>(workspaceCollection());
export const createWorkspace = (body: { name: string; key_salt: string; encrypted_dek: string }) =>
	apiFetch<ApiWorkspace>(workspaceCollection(), {
		method: 'POST',
		body: JSON.stringify(body)
	});

export const listShelves = (workspaceId: string) =>
	apiFetch<ApiShelf[]>(workspaceShelves(workspaceId));
export const createShelf = (workspaceId: string, body: { name: string; icon?: string }) =>
	apiFetch<ApiShelf>(workspaceShelves(workspaceId), {
		method: 'POST',
		body: JSON.stringify(body)
	});
export const updateShelf = (id: string, body: { name?: string; icon?: string }) =>
	apiFetch<ApiShelf>(shelf(id), {
		method: 'PATCH',
		body: JSON.stringify(body)
	});
export const deleteShelf = (id: string) => apiFetch<void>(shelf(id), { method: 'DELETE' });

export const listEntries = (
	shelfId: string,
	params: { cursor?: string | null; order?: EntryOrder; limit?: number } = {}
) => {
	const search = new URLSearchParams();
	if (params.cursor) search.set('cursor', params.cursor);
	if (params.order) search.set('order', params.order);
	if (params.limit) search.set('limit', String(params.limit));
	const query = search.toString();
	return apiFetch<ApiEntryPage>(
		query ? `${shelfEntries(shelfId)}?${query}` : shelfEntries(shelfId)
	);
};
export const getEntry = (id: string) => apiFetch<ApiEntry>(entry(id));
export const createEntry = (shelfId: string, body: { title: string; content: string }) =>
	apiFetch<ApiEntry>(shelfEntries(shelfId), {
		method: 'POST',
		body: JSON.stringify(body)
	});
export const updateEntry = (id: string, body: { title?: string; content?: string }) =>
	apiFetch<ApiEntry>(entry(id), {
		method: 'PATCH',
		body: JSON.stringify(body)
	});
export const deleteEntry = (id: string) => apiFetch<void>(entry(id), { method: 'DELETE' });
