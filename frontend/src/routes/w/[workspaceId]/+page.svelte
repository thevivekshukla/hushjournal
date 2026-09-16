<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import * as api from '$lib/api';
	import EntryEditor from '$lib/components/EntryEditor.svelte';
	import Modal from '$lib/components/Modal.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import UserMenu from '$lib/components/UserMenu.svelte';
	import { SHELF_ICONS, formatEntryDate, journal, type Shelf } from '$lib/journal.svelte';
	import { session } from '$lib/session.svelte';

	let selectedShelfId = $state<string | null>(null);
	let selectedEntryId = $state<string | null>(null);
	let mobilePane = $state<'nav' | 'editor'>('nav');
	let shelfOpen = $state(false);
	let newShelfName = $state('');
	let newShelfIcon = $state<string>(SHELF_ICONS[0]);
	let editShelfOpen = $state(false);
	let editingShelfId = $state<string | null>(null);
	let editShelfName = $state('');
	let editShelfIcon = $state<string>(SHELF_ICONS[0]);
	let deleteOpen = $state(false);
	let deleteShelfOpen = $state(false);

	const workspaceId = $derived(page.params.workspaceId ?? '');
	const workspace = $derived(journal.workspace(workspaceId));
	const shelves = $derived(journal.shelvesFor(workspaceId));
	const selectedShelf = $derived(
		shelves.find((shelf) => shelf.id === selectedShelfId) ?? shelves[0]
	);
	const entries = $derived(selectedShelf ? journal.entriesFor(selectedShelf.id) : []);
	const selectedEntry = $derived(
		entries.find((entry) => entry.id === selectedEntryId) ?? entries[0] ?? null
	);

	$effect(() => {
		if (!session.user) {
			void goto(api.login());
			return;
		}
		if (!workspace || session.unlockedWorkspaceId !== workspaceId) {
			void goto(api.workspaces());
		}
	});

	function selectShelf(id: string) {
		selectedShelfId = id;
		selectedEntryId = null;
		mobilePane = 'nav';
	}

	function selectEntry(id: string) {
		selectedEntryId = id;
		mobilePane = 'editor';
	}

	function writeToday() {
		if (!selectedShelf) return;
		const entry = journal.createEntry(selectedShelf.id, formatEntryDate());
		selectedEntryId = entry.id;
		mobilePane = 'editor';
	}

	function createShelf() {
		const name = newShelfName.trim();
		if (!name) return;
		const shelf = journal.createShelf(workspaceId, name, newShelfIcon);
		selectedShelfId = shelf.id;
		selectedEntryId = null;
		newShelfName = '';
		shelfOpen = false;
	}

	function openEditShelf(shelf: Shelf, event?: MouseEvent) {
		event?.stopPropagation();
		editingShelfId = shelf.id;
		editShelfName = shelf.name;
		editShelfIcon = shelf.icon;
		editShelfOpen = true;
	}

	function saveShelf(event: SubmitEvent) {
		event.preventDefault();
		const name = editShelfName.trim();
		if (!name || !editingShelfId) return;
		journal.updateShelf(editingShelfId, { name, icon: editShelfIcon });
		editShelfOpen = false;
	}

	function requestDeleteShelf() {
		editShelfOpen = false;
		deleteShelfOpen = true;
	}

	function confirmDeleteShelf() {
		if (!editingShelfId) return;
		if (selectedShelfId === editingShelfId || selectedShelf?.id === editingShelfId) {
			selectedShelfId = null;
			selectedEntryId = null;
		}
		journal.deleteShelf(editingShelfId);
		deleteShelfOpen = false;
		mobilePane = 'nav';
	}

	function confirmDelete() {
		if (!selectedEntry) return;
		journal.deleteEntry(selectedEntry.id);
		selectedEntryId = null;
		deleteOpen = false;
		mobilePane = 'nav';
	}

	function onKeydown(event: KeyboardEvent) {
		const target = event.target as HTMLElement | null;
		if (target && ['INPUT', 'TEXTAREA'].includes(target.tagName)) return;
		if (event.key === 'n' && !event.metaKey && !event.ctrlKey) {
			event.preventDefault();
			writeToday();
		}
	}
</script>

<svelte:window onkeydown={onKeydown} />

<svelte:head>
	<title>{workspace?.name ?? 'Journal'} · e2ejournal</title>
</svelte:head>

{#if session.user && workspace && session.unlockedWorkspaceId === workspaceId}
	<div class="flex h-dvh min-h-0 flex-col">
		<header class="flex h-14 shrink-0 items-center gap-2 border-b border-base-300 px-3">
			{#if mobilePane === 'editor'}
				<button
					type="button"
					class="btn btn-circle btn-ghost btn-sm md:hidden"
					aria-label="Back to entries"
					onclick={() => (mobilePane = 'nav')}
				>
					<span class="icon-[lucide--arrow-left] size-4"></span>
				</button>
			{/if}
			<button
				type="button"
				class="btn gap-2 btn-ghost px-2 btn-sm"
				onclick={() => {
					session.lock();
					void goto(api.workspaces());
				}}
			>
				<span class="icon-[lucide--lock-keyhole] size-4"></span>
				<span class="font-serif text-base tracking-tight">{workspace.name}</span>
			</button>
			<div class="ml-auto flex items-center gap-1">
				<ThemeToggle />
				<UserMenu compact />
			</div>
		</header>

		<div class="flex min-h-0 flex-1 overflow-hidden">
			<aside class="hidden w-56 shrink-0 flex-col border-r border-base-300 md:flex">
				<div class="flex items-center justify-between px-3 pt-4 pb-2">
					<p class="text-xs tracking-wide text-base-content/50 uppercase">Shelves</p>
					<button
						type="button"
						class="btn btn-circle btn-ghost btn-xs"
						aria-label="New shelf"
						onclick={() => (shelfOpen = true)}
					>
						<span class="icon-[lucide--plus] size-4"></span>
					</button>
				</div>
				<nav class="flex-1 scrollbar-thin overflow-y-auto px-2 pb-4">
					{#each shelves as shelf (shelf.id)}
						<div
							class={[
								'group flex w-full items-center rounded-xl',
								selectedShelf?.id === shelf.id ? 'bg-base-200' : 'hover:bg-base-200/70'
							]}
						>
							<button
								type="button"
								class={[
									'flex min-w-0 flex-1 items-center gap-2 rounded-xl px-3 py-2 text-left text-sm transition-colors',
									selectedShelf?.id === shelf.id ? 'font-medium' : 'text-base-content/80'
								]}
								onclick={() => selectShelf(shelf.id)}
							>
								<span class={[shelf.icon, 'size-4 shrink-0']}></span>
								<span class="truncate">{shelf.name}</span>
							</button>
							<button
								type="button"
								class="btn mr-1 btn-circle btn-ghost opacity-0 btn-xs group-hover:opacity-100 focus-visible:opacity-100"
								aria-label="Edit shelf"
								onclick={(event) => openEditShelf(shelf, event)}
							>
								<span class="icon-[lucide--pencil] size-3.5"></span>
							</button>
						</div>
					{/each}
					{#if shelves.length === 0}
						<p class="px-3 py-6 text-sm text-base-content/60">Create a shelf to start writing.</p>
					{/if}
				</nav>
			</aside>

			<aside
				class={[
					'min-h-0 w-full shrink-0 flex-col border-r border-base-300 md:w-72',
					mobilePane === 'nav' ? 'flex' : 'hidden md:flex'
				]}
			>
				<div class="flex gap-2 overflow-x-auto px-3 pt-3 md:hidden">
					{#each shelves as shelf (shelf.id)}
						<button
							type="button"
							class={[
								'btn rounded-full btn-sm',
								selectedShelf?.id === shelf.id ? 'btn-neutral' : 'btn-ghost'
							]}
							onclick={() => selectShelf(shelf.id)}
						>
							<span class={[shelf.icon, 'size-4']}></span>
							{shelf.name}
						</button>
					{/each}
					<button
						type="button"
						class="btn btn-circle btn-ghost btn-sm"
						aria-label="New shelf"
						onclick={() => (shelfOpen = true)}
					>
						<span class="icon-[lucide--plus] size-4"></span>
					</button>
				</div>
				<div class="flex items-center justify-between gap-2 px-3 pt-4 pb-2">
					<div class="flex min-w-0 flex-1 items-center gap-1">
						<p class="truncate text-sm font-medium">{selectedShelf?.name ?? 'Entries'}</p>
						{#if selectedShelf}
							<button
								type="button"
								class="btn btn-circle shrink-0 btn-ghost btn-xs"
								aria-label="Edit shelf"
								onclick={() => openEditShelf(selectedShelf)}
							>
								<span class="icon-[lucide--pencil] size-3.5"></span>
							</button>
						{/if}
					</div>
					<button
						type="button"
						class="btn shrink-0 rounded-full btn-neutral btn-sm"
						onclick={writeToday}
					>
						<span class="icon-[lucide--plus] size-4"></span>
						Note
					</button>
				</div>
				<div class="flex-1 scrollbar-thin overflow-y-auto px-2 pb-4">
					{#each entries as entry (entry.id)}
						<button
							type="button"
							class={[
								'w-full truncate rounded-xl px-3 py-2.5 text-left text-sm font-medium transition-colors',
								selectedEntry?.id === entry.id ? 'bg-base-200' : 'hover:bg-base-200/70'
							]}
							onclick={() => selectEntry(entry.id)}
						>
							{entry.title}
						</button>
					{:else}
						<p class="px-3 py-8 text-sm text-base-content/60">
							No entries yet. Press <kbd class="kbd kbd-sm">n</kbd> or Note.
						</p>
					{/each}
				</div>
			</aside>

			<main
				class={[
					'flex min-h-0 min-w-0 flex-1 flex-col',
					mobilePane === 'editor' ? 'flex' : 'hidden md:flex'
				]}
			>
				{#if selectedEntry}
					{#key selectedEntry.id}
						<EntryEditor entry={selectedEntry} onDelete={() => (deleteOpen = true)} />
					{/key}
				{:else}
					<div class="flex flex-1 flex-col items-center justify-center px-6 text-center">
						<span class="icon-[lucide--pen-line] size-8 text-base-content/30"></span>
						<p class="mt-3 font-serif text-xl">Pick an entry, or start today’s page.</p>
						<button type="button" class="btn mt-4 rounded-full btn-neutral" onclick={writeToday}>
							Write today
						</button>
					</div>
				{/if}
			</main>
		</div>
	</div>
{/if}

<Modal
	bind:open={shelfOpen}
	title="New shelf"
	description="Shelf names are encrypted before they are stored."
>
	<label class="w-full" for="new-shelf-name">
		<span class="mb-1 block text-sm">Name</span>
		<input
			id="new-shelf-name"
			name="name"
			class="input w-full"
			type="text"
			autocomplete="off"
			bind:value={newShelfName}
		/>
	</label>
	<div>
		<p class="mb-2 text-sm">Icon</p>
		<div class="flex flex-wrap gap-2">
			{#each SHELF_ICONS as icon (icon)}
				<button
					type="button"
					class={['btn btn-square btn-sm', newShelfIcon === icon ? 'btn-neutral' : 'btn-ghost']}
					aria-label="Shelf icon"
					onclick={() => (newShelfIcon = icon)}
				>
					<span class={[icon, 'size-4']}></span>
				</button>
			{/each}
		</div>
	</div>
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (shelfOpen = false)}>Cancel</button>
		<button type="button" class="btn btn-neutral" onclick={createShelf}>Create</button>
	{/snippet}
</Modal>

<Modal
	bind:open={editShelfOpen}
	title="Edit shelf"
	description="Shelf names are encrypted before they are stored."
>
	<form id="edit-shelf-form" class="flex flex-col gap-3" onsubmit={saveShelf}>
		<label class="w-full" for="edit-shelf-name">
			<span class="mb-1 block text-sm">Name</span>
			<!-- svelte-ignore a11y_autofocus -->
			<input
				id="edit-shelf-name"
				name="name"
				class="input w-full"
				type="text"
				placeholder="Name"
				autocomplete="off"
				bind:value={editShelfName}
				autofocus
			/>
		</label>
		<div>
			<p class="mb-2 text-sm">Icon</p>
			<div class="flex flex-wrap gap-2">
				{#each SHELF_ICONS as icon (icon)}
					<button
						type="button"
						class={['btn btn-square btn-sm', editShelfIcon === icon ? 'btn-neutral' : 'btn-ghost']}
						aria-label="Shelf icon"
						onclick={() => (editShelfIcon = icon)}
					>
						<span class={[icon, 'size-4']}></span>
					</button>
				{/each}
			</div>
		</div>
	</form>
	{#snippet footer()}
		<button type="button" class="btn mr-auto btn-error" onclick={requestDeleteShelf}>
			Delete
		</button>
		<button type="button" class="btn btn-ghost" onclick={() => (editShelfOpen = false)}>
			Cancel
		</button>
		<button type="submit" form="edit-shelf-form" class="btn btn-neutral">Save</button>
	{/snippet}
</Modal>

<Modal
	bind:open={deleteShelfOpen}
	title="Delete this shelf?"
	description="This removes the shelf and its notes from the prototype. Later it will delete the ciphertext on the server."
>
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (deleteShelfOpen = false)}>
			Cancel
		</button>
		<button type="button" class="btn btn-error" onclick={confirmDeleteShelf}>Delete</button>
	{/snippet}
</Modal>

<Modal
	bind:open={deleteOpen}
	title="Delete this entry?"
	description="This removes the page from the prototype. Later it will delete the ciphertext on the server."
>
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (deleteOpen = false)}>Cancel</button>
		<button type="button" class="btn btn-error" onclick={confirmDelete}>Delete</button>
	{/snippet}
</Modal>
