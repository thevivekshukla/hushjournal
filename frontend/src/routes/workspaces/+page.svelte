<script lang="ts">
	import { goto } from '$app/navigation';
	import Modal from '$lib/components/Modal.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import UserMenu from '$lib/components/UserMenu.svelte';
	import { journal } from '$lib/journal.svelte';
	import { session } from '$lib/session.svelte';

	let createOpen = $state(false);
	let unlockOpen = $state(false);
	let pendingWorkspaceId = $state<string | null>(null);
	let newName = $state('');
	let passphrase = $state('');
	let error = $state('');

	const pendingWorkspace = $derived(
		pendingWorkspaceId ? journal.workspace(pendingWorkspaceId) : undefined
	);

	$effect(() => {
		if (!session.user) void goto('/');
	});

	function openUnlock(id: string) {
		pendingWorkspaceId = id;
		passphrase = '';
		error = '';
		unlockOpen = true;
	}

	function unlock() {
		if (!passphrase.trim() || !pendingWorkspaceId) {
			error = 'Enter the workspace passphrase.';
			return;
		}
		session.unlock(pendingWorkspaceId);
		unlockOpen = false;
		void goto(`/w/${pendingWorkspaceId}`);
	}

	function create() {
		const name = newName.trim();
		if (!name) {
			error = 'Name the workspace.';
			return;
		}
		if (!passphrase.trim()) {
			error = 'Choose a passphrase. It never leaves this device.';
			return;
		}
		const workspace = journal.createWorkspace(name);
		session.unlock(workspace.id);
		createOpen = false;
		newName = '';
		passphrase = '';
		void goto(`/w/${workspace.id}`);
	}

	function openCreate() {
		error = '';
		newName = '';
		passphrase = '';
		createOpen = true;
	}
</script>

<svelte:head>
	<title>Workspaces · e2ejournal</title>
</svelte:head>

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

		<section class="mt-10 grid gap-4 sm:grid-cols-2">
			{#each journal.workspaces as workspace (workspace.id)}
				{@const shelfCount = journal.shelvesFor(workspace.id).length}
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
						<p class="text-sm text-base-content/60">
							{shelfCount}
							{shelfCount === 1 ? 'shelf' : 'shelves'}
						</p>
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
	<label class="w-full">
		<span class="mb-1 block text-sm">Passphrase</span>
		<input
			class="input w-full"
			type="password"
			autocomplete="current-password"
			bind:value={passphrase}
			onkeydown={(event) => event.key === 'Enter' && unlock()}
		/>
	</label>
	{#if error}
		<p class="text-sm text-error">{error}</p>
	{/if}
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (unlockOpen = false)}>Cancel</button>
		<button type="button" class="btn btn-neutral" onclick={unlock}>Unlock</button>
	{/snippet}
</Modal>

<Modal
	bind:open={createOpen}
	title="New workspace"
	description="Give it a name and a passphrase. Both encryption keys are derived on this device."
>
	<label class="w-full">
		<span class="mb-1 block text-sm">Name</span>
		<input class="input w-full" type="text" bind:value={newName} />
	</label>
	<label class="w-full">
		<span class="mb-1 block text-sm">Passphrase</span>
		<input
			class="input w-full"
			type="password"
			autocomplete="new-password"
			bind:value={passphrase}
		/>
	</label>
	{#if error}
		<p class="text-sm text-error">{error}</p>
	{/if}
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (createOpen = false)}>Cancel</button>
		<button type="button" class="btn btn-neutral" onclick={create}>Create</button>
	{/snippet}
</Modal>
