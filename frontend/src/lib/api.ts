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
