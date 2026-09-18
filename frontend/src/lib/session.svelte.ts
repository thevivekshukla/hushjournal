import * as api from '$lib/api';
import { zeroKey } from '$lib/crypto';
import { ApiError, setUnauthorizedHandler } from '$lib/http';

export type User = {
	id: string;
	name: string;
	email: string;
	avatarUrl: string | null;
};

function mapUser(user: api.ApiUser): User {
	return {
		id: user.id,
		name: user.name,
		email: user.email ?? user.google_email ?? user.username ?? '',
		avatarUrl: user.google_avatar_url
	};
}

class Session {
	user = $state.raw<User | null>(null);
	ready = $state(false);
	loadError = $state<string | null>(null);
	unlockedJournalId = $state<string | null>(null);
	#dek: Uint8Array | null = null;

	constructor() {
		setUnauthorizedHandler(() => this.clearLocal());
	}

	get dek() {
		return this.#dek;
	}

	async hydrate() {
		if (this.ready) return;
		this.loadError = null;
		try {
			this.user = mapUser(await api.getUser());
		} catch (error) {
			this.user = null;
			if (!(error instanceof ApiError && error.status === 401)) {
				this.loadError = error instanceof Error ? error.message : 'Could not reach the server.';
			}
		} finally {
			this.ready = true;
		}
	}

	setUser(user: api.ApiUser) {
		this.user = mapUser(user);
		this.ready = true;
		this.loadError = null;
	}

	unlock(journalId: string, dek: Uint8Array) {
		zeroKey(this.#dek);
		this.#dek = dek;
		this.unlockedJournalId = journalId;
	}

	lock() {
		zeroKey(this.#dek);
		this.#dek = null;
		this.unlockedJournalId = null;
	}

	clearLocal() {
		this.lock();
		this.user = null;
	}

	async logout() {
		try {
			await api.logoutUser();
		} catch {
			// Cookie may already be gone; still drop local secrets.
		} finally {
			this.clearLocal();
		}
	}

	requireDek() {
		if (!this.#dek || !this.unlockedJournalId) {
			throw new Error('Journal is locked.');
		}
		return this.#dek;
	}
}

export const session = new Session();
