export type User = {
	id: string;
	name: string;
	email: string;
};

export const demoUser: User = {
	id: 'user-1',
	name: 'Vivek',
	email: 'vivek@example.com'
};

const STORAGE_KEY = 'e2ejournal.session';

function readStored(): { user: User | null; unlockedWorkspaceId: string | null } {
	if (typeof sessionStorage === 'undefined') {
		return { user: null, unlockedWorkspaceId: null };
	}
	try {
		const raw = sessionStorage.getItem(STORAGE_KEY);
		if (!raw) return { user: null, unlockedWorkspaceId: null };
		return JSON.parse(raw);
	} catch {
		return { user: null, unlockedWorkspaceId: null };
	}
}

class Session {
	user = $state<User | null>(null);
	unlockedWorkspaceId = $state<string | null>(null);

	constructor() {
		const stored = readStored();
		this.user = stored.user;
		this.unlockedWorkspaceId = stored.unlockedWorkspaceId;
	}

	login(user: User) {
		this.user = user;
		this.persist();
	}

	unlock(workspaceId: string) {
		this.unlockedWorkspaceId = workspaceId;
		this.persist();
	}

	lock() {
		this.unlockedWorkspaceId = null;
		this.persist();
	}

	logout() {
		this.user = null;
		this.unlockedWorkspaceId = null;
		this.persist();
	}

	persist() {
		if (typeof sessionStorage === 'undefined') return;
		sessionStorage.setItem(
			STORAGE_KEY,
			JSON.stringify({
				user: this.user,
				unlockedWorkspaceId: this.unlockedWorkspaceId
			})
		);
	}
}

export const session = new Session();
