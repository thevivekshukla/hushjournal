export class ApiError extends Error {
	status: number;

	constructor(status: number, message: string) {
		super(message);
		this.name = 'ApiError';
		this.status = status;
	}
}

let onUnauthorized: (() => void) | null = null;

export function setUnauthorizedHandler(handler: (() => void) | null) {
	onUnauthorized = handler;
}

export async function apiFetch<T>(
	path: string,
	init: RequestInit = {},
	fetcher: typeof fetch = fetch
): Promise<T> {
	const headers = new Headers(init.headers);
	if (init.body !== undefined && !headers.has('Content-Type')) {
		headers.set('Content-Type', 'application/json');
	}

	const response = await (fetcher ?? fetch)(path, {
		...init,
		credentials: 'include',
		headers
	});

	if (response.status === 401) onUnauthorized?.();
	if (response.status === 204) return undefined as T;

	const text = await response.text();
	let data: unknown = null;
	if (text) {
		try {
			data = JSON.parse(text);
		} catch {
			data = { error: text };
		}
	}

	if (!response.ok) {
		const message =
			typeof data === 'object' && data !== null && 'error' in data && typeof data.error === 'string'
				? data.error
				: response.statusText;
		throw new ApiError(response.status, message);
	}

	return data as T;
}
