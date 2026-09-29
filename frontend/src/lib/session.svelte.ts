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
	appConfig = $state.raw<api.ApiAppConfig>({
		disable_user_signup: false,
		disable_password_form: false,
		disable_google_login: false
	});
	ready = $state(false);
	loadError = $state<string | null>(null);
	unlockedJournalId = $state<string | null>(null);
	#dek: Uint8Array | null = null;
	#afterClear: (() => void) | null = null;

	constructor() {
		setUnauthorizedHandler(() => this.clearLocal());
	}

	setAfterClear(handler: () => void) {
		this.#afterClear = handler;
	}

	get dek() {
		return this.#dek;
	}

	async hydrate(fetcher: typeof fetch = fetch) {
		if (this.ready) return;
		this.loadError = null;
		const [userResult, configResult] = await Promise.allSettled([
			api.getUser(fetcher),
			api.getAppConfig(fetcher)
		]);
		if (userResult.status === 'fulfilled') {
			this.user = mapUser(userResult.value);
		} else {
			this.user = null;
			const error = userResult.reason;
			if (!(error instanceof ApiError && error.status === 401)) {
				this.loadError = error instanceof Error ? error.message : 'Could not reach the server.';
			}
		}
		this.appConfig =
			configResult.status === 'fulfilled'
				? configResult.value
				: { disable_user_signup: false, disable_password_form: false, disable_google_login: false };
		this.ready = true;
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
		this.#afterClear?.();
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
