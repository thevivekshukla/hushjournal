<script lang="ts">
	import { goto } from '$app/navigation';
	import * as api from '$lib/api';
	import Modal from '$lib/components/Modal.svelte';
	import NoIndex from '$lib/components/NoIndex.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import UserMenu from '$lib/components/UserMenu.svelte';
	import { CryptoError, journal } from '$lib/journal.svelte';
	import { session } from '$lib/session.svelte';
	import { untrack } from 'svelte';

	const HINT_MAX = 255;

	let createOpen = $state(false);
	let unlockOpen = $state(false);
	let pendingWorkspaceId = $state<string | null>(null);
	let newName = $state('');
	let passphrase = $state('');
	let hint = $state('');
	let error = $state('');
	let busy = $state(false);

	const pendingWorkspace = $derived(
		pendingWorkspaceId ? journal.workspace(pendingWorkspaceId) : undefined
	);

	$effect(() => {
		if (!session.user) {
			void goto(api.login());
			return;
		}
		untrack(() => {
			void journal.loadWorkspaces();
		});
	});

	function openUnlock(id: string) {
		pendingWorkspaceId = id;
		passphrase = '';
		error = '';
		unlockOpen = true;
	}

	async function unlock() {
		if (!passphrase.trim() || !pendingWorkspace) {
			error = 'Enter the workspace passphrase.';
			return;
		}
		busy = true;
		error = '';
		try {
			await journal.unlockWorkspace(pendingWorkspace, passphrase);
			unlockOpen = false;
			passphrase = '';
			void goto(api.workspace(pendingWorkspace.id));
		} catch (cause) {
			error = cause instanceof CryptoError ? cause.message : 'Could not unlock this workspace.';
		} finally {
			busy = false;
		}
	}

	async function create() {
		const name = newName.trim();
		if (!name) {
			error = 'Name the workspace.';
			return;
		}
		if (!passphrase.trim()) {
			error = 'Choose a passphrase. It never leaves this device.';
			return;
		}
		busy = true;
		error = '';
		try {
			const workspace = await journal.createWorkspace(name, passphrase, hint);
			createOpen = false;
			newName = '';
			passphrase = '';
			hint = '';
			void goto(api.workspace(workspace.id));
		} catch (cause) {
			error = cause instanceof Error ? cause.message : 'Could not create the workspace.';
		} finally {
			busy = false;
		}
	}

	function openCreate() {
		error = '';
		newName = '';
		passphrase = '';
		hint = '';
		createOpen = true;
	}
</script>

<svelte:head>
	<title>Workspaces · e2ejournal</title>
</svelte:head>

<NoIndex />

{#if session.user}
	<div class="mx-auto flex min-h-dvh w-full max-w-3xl flex-col px-5 py-6">
		<header class="flex items-center justify-between gap-3">
			<div>
				<p class="font-serif text-lg tracking-tight">e2ejournal</p>
				<p class="text-sm text-base-content/60">Choose a workspace to unlock.</p>
			</div>
			<div class="flex items-center gap-1">
				<ThemeToggle />
				<UserMenu />
			</div>
		</header>

		{#if journal.loading && journal.workspaces.length === 0}
			<p class="mt-10 text-sm text-base-content/60">Loading workspaces…</p>
		{:else if journal.error && journal.workspaces.length === 0}
			<p class="mt-10 text-sm text-error">{journal.error}</p>
		{/if}

		<section class="mt-10 grid gap-4 sm:grid-cols-2">
			{#each journal.workspaces as workspace (workspace.id)}
				<button
					type="button"
					class="card border border-base-300 bg-base-200/70 text-left transition-colors hover:bg-base-200"
					onclick={() => openUnlock(workspace.id)}
				>
					<div class="card-body gap-3 p-5">
						<div class="flex items-start justify-between">
							<h2 class="font-serif text-2xl font-semibold tracking-tight">{workspace.name}</h2>
							<span class="icon-[lucide--lock-keyhole] size-5 text-base-content/50"></span>
						</div>
						{#if workspace.passphraseHint}
							<p class="text-sm text-base-content/60">Hint: {workspace.passphraseHint}</p>
						{:else}
							<p class="text-sm text-base-content/60">Passphrase stays on this device.</p>
						{/if}
					</div>
				</button>
			{/each}

			<button
				type="button"
				class="card border border-dashed border-base-300 text-left text-base-content/70 transition-colors hover:border-base-content/30 hover:text-base-content"
				onclick={openCreate}
			>
				<div class="card-body items-start justify-center gap-2 p-5">
					<span class="icon-[lucide--plus] size-5"></span>
					<p class="font-medium">New workspace</p>
				</div>
			</button>
		</section>
	</div>
{/if}

<Modal
	bind:open={unlockOpen}
	title="Unlock {pendingWorkspace?.name ?? 'workspace'}"
	description="The passphrase stays on this device. The server never sees it."
>
	{#if pendingWorkspace?.passphraseHint}
		<p class="text-sm text-base-content/70">Hint: {pendingWorkspace.passphraseHint}</p>
	{/if}
	<label class="w-full" for="workspace-unlock-passphrase">
		<span class="mb-1 block text-sm">Passphrase</span>
		<input
			id="workspace-unlock-passphrase"
			name="passphrase"
			class="input w-full"
			type="password"
			autocomplete="current-password"
			bind:value={passphrase}
			onkeydown={(event) => event.key === 'Enter' && !busy && void unlock()}
		/>
	</label>
	{#if error}
		<p class="text-sm text-error">{error}</p>
	{/if}
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (unlockOpen = false)}>Cancel</button>
		<button type="button" class="btn btn-neutral" onclick={() => void unlock()} disabled={busy}>
			{busy ? 'Unlocking…' : 'Unlock'}
		</button>
	{/snippet}
</Modal>

<Modal
	bind:open={createOpen}
	title="New workspace"
	description="Give it a name and a passphrase. Both encryption keys are derived on this device."
>
	<label class="w-full" for="workspace-name">
		<span class="mb-1 block text-sm">Name</span>
		<input
			id="workspace-name"
			name="name"
			class="input w-full"
			type="text"
			autocomplete="off"
			bind:value={newName}
		/>
	</label>
	<label class="w-full" for="workspace-passphrase">
		<span class="mb-1 block text-sm">Passphrase</span>
		<input
			id="workspace-passphrase"
			name="new-passphrase"
			class="input w-full"
			type="password"
			autocomplete="new-password"
			bind:value={passphrase}
		/>
	</label>
	<label class="w-full" for="workspace-passphrase-hint">
		<span class="mb-1 block text-sm">Passphrase hint (optional)</span>
		<input
			id="workspace-passphrase-hint"
			name="passphrase-hint"
			class="input w-full"
			type="text"
			autocomplete="off"
			maxlength={HINT_MAX}
			bind:value={hint}
		/>
		<span class="mt-1 block text-xs text-base-content/50">Shown before unlock. Not encrypted.</span>
	</label>
	{#if error}
		<p class="text-sm text-error">{error}</p>
	{/if}
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (createOpen = false)}>Cancel</button>
		<button type="button" class="btn btn-neutral" onclick={() => void create()} disabled={busy}>
			{busy ? 'Creating…' : 'Create'}
		</button>
	{/snippet}
</Modal>
