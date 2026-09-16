<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import * as api from '$lib/api';
	import EntryEditor from '$lib/components/EntryEditor.svelte';
	import Modal from '$lib/components/Modal.svelte';
	import NoIndex from '$lib/components/NoIndex.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import UserMenu from '$lib/components/UserMenu.svelte';
	import WorkspaceEditModal from '$lib/components/WorkspaceEditModal.svelte';
	import { SHELF_ICONS, formatEntryDate, journal, type Shelf } from '$lib/journal.svelte';
	import { session } from '$lib/session.svelte';
	import { untrack } from 'svelte';
	import type { Attachment } from 'svelte/attachments';

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
	let workspaceEditOpen = $state(false);
	let editName = $state('');
	let editHint = $state('');
	let editMask = $state(false);
	let busy = $state(false);

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
	const maskOn = $derived(workspace?.mask === true);

	$effect(() => {
		if (!session.user) {
			void goto(api.login());
			return;
		}
		if (session.unlockedWorkspaceId !== workspaceId) {
			void goto(api.workspaces());
		}
	});

	$effect(() => {
		if (session.unlockedWorkspaceId !== workspaceId) return;
		const id = workspaceId;
		untrack(() => {
			void journal.loadShelves(id);
		});
	});

	$effect(() => {
		const shelfId = selectedShelf?.id;
		if (!shelfId) return;
		untrack(() => {
			void journal.loadEntries(shelfId);
		});
	});

	$effect(() => {
		const id = selectedEntry?.id;
		if (!id) return;
		untrack(() => {
			void journal.ensureContent(id);
		});
	});

	function selectShelf(id: string) {
		if (selectedShelfId === id) return;
		void journal.flush();
		selectedShelfId = id;
		selectedEntryId = null;
		mobilePane = 'nav';
	}

	function setEntryOrder(order: api.EntryOrder) {
		if (journal.entryOrder === order) return;
		journal.entryOrder = order;
		if (selectedShelf) void journal.loadEntries(selectedShelf.id);
	}

	function toggleEntryOrder() {
		setEntryOrder(journal.entryOrder === 'desc' ? 'asc' : 'desc');
	}

	async function loadMore() {
		if (!selectedShelf || journal.loadingMore || busy) return;
		try {
			await journal.loadMore(selectedShelf.id);
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not load more notes.';
		}
	}

	function loadMoreSentinel(_key: string): Attachment {
		return (node) => {
			const root = node.parentElement;
			if (!root) return;
			const observer = new IntersectionObserver(
				(records) => {
					if (records.some((record) => record.isIntersecting)) void loadMore();
				},
				{ root, rootMargin: '240px 0px' }
			);
			observer.observe(node);
			return () => observer.disconnect();
		};
	}

	function selectEntry(id: string) {
		if (selectedEntryId !== id) void journal.flush(selectedEntryId ?? undefined);
		selectedEntryId = id;
		mobilePane = 'editor';
	}

	async function writeToday() {
		if (!selectedShelf) return;
		busy = true;
		try {
			await journal.flush();
			const entry = await journal.createEntry(selectedShelf.id, formatEntryDate());
			selectedEntryId = entry.id;
			mobilePane = 'editor';
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not create a note.';
		} finally {
			busy = false;
		}
	}

	async function createShelf() {
		const name = newShelfName.trim();
		if (!name) return;
		busy = true;
		try {
			const shelf = await journal.createShelf(workspaceId, name, newShelfIcon);
			selectedShelfId = shelf.id;
			selectedEntryId = null;
			newShelfName = '';
			shelfOpen = false;
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not create the shelf.';
		} finally {
			busy = false;
		}
	}

	function openEditShelf(shelf: Shelf, event?: MouseEvent) {
		event?.stopPropagation();
		editingShelfId = shelf.id;
		editShelfName = shelf.name;
		editShelfIcon = shelf.icon;
		editShelfOpen = true;
	}

	async function saveShelf(event: SubmitEvent) {
		event.preventDefault();
		const name = editShelfName.trim();
		if (!name || !editingShelfId) return;
		busy = true;
		try {
			await journal.updateShelf(editingShelfId, { name, icon: editShelfIcon });
			editShelfOpen = false;
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not save the shelf.';
		} finally {
			busy = false;
		}
	}

	function requestDeleteShelf() {
		editShelfOpen = false;
		deleteShelfOpen = true;
	}

	async function confirmDeleteShelf() {
		if (!editingShelfId) return;
		busy = true;
		try {
			if (selectedShelfId === editingShelfId || selectedShelf?.id === editingShelfId) {
				selectedShelfId = null;
				selectedEntryId = null;
			}
			await journal.deleteShelf(editingShelfId);
			deleteShelfOpen = false;
			mobilePane = 'nav';
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not delete the shelf.';
		} finally {
			busy = false;
		}
	}

	async function confirmDelete() {
		if (!selectedEntry) return;
		busy = true;
		try {
			await journal.deleteEntry(selectedEntry.id);
			selectedEntryId = null;
			deleteOpen = false;
			mobilePane = 'nav';
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not delete the note.';
		} finally {
			busy = false;
		}
	}

	function onKeydown(event: KeyboardEvent) {
		const target = event.target as HTMLElement | null;
		if (target && ['INPUT', 'TEXTAREA'].includes(target.tagName)) return;
		if (event.key === 'n' && !event.metaKey && !event.ctrlKey) {
			event.preventDefault();
			void writeToday();
		}
	}

	function openWorkspaceEdit() {
		if (!workspace) return;
		editName = workspace.name;
		editHint = workspace.passphraseHint ?? '';
		editMask = workspace.mask;
		workspaceEditOpen = true;
	}

	function lockWorkspace() {
		void journal.flush();
		session.lock();
		journal.clearWorkspace();
		void goto(api.workspaces());
	}

	function onPageHide() {
		void journal.flush();
	}
</script>

<svelte:window onkeydown={onKeydown} onpagehide={onPageHide} />

<svelte:head>
	<title>{workspace?.name ?? 'Journal'} · e2ejournal</title>
</svelte:head>

<NoIndex />

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
			<button type="button" class="btn gap-2 btn-ghost px-2 btn-sm" onclick={lockWorkspace}>
				<span class="icon-[lucide--lock-keyhole] size-4"></span>
				<span class="font-serif text-base tracking-tight">{workspace.name}</span>
			</button>
			<button
				type="button"
				class="btn btn-circle btn-ghost btn-sm"
				aria-label="Edit workspace"
				onclick={openWorkspaceEdit}
			>
				<span class="icon-[lucide--pencil] size-4"></span>
			</button>
			<div class="ml-auto flex items-center gap-1">
				<ThemeToggle />
				<UserMenu compact />
			</div>
		</header>

		{#if journal.error}
			<p class="border-b border-error/20 bg-error/10 px-4 py-2 text-sm text-error">
				{journal.error}
			</p>
		{/if}

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
								aria-label={maskOn && selectedShelf?.id !== shelf.id ? 'Shelf' : shelf.name}
								onclick={() => selectShelf(shelf.id)}
							>
								<span class={[shelf.icon, 'size-4 shrink-0']}></span>
								{#if maskOn && selectedShelf?.id !== shelf.id}
									<span class="inline-block h-3 w-28 rounded-full bg-base-content/20"></span>
								{:else}
									<span class="truncate">{shelf.name}</span>
								{/if}
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
					{#if shelves.length === 0 && !journal.loading}
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
							aria-label={maskOn && selectedShelf?.id !== shelf.id ? 'Shelf' : shelf.name}
							onclick={() => selectShelf(shelf.id)}
						>
							<span class={[shelf.icon, 'size-4']}></span>
							{#if maskOn && selectedShelf?.id !== shelf.id}
								<span class="inline-block h-3 w-16 rounded-full bg-base-content/20"></span>
							{:else}
								{shelf.name}
							{/if}
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
					<div class="flex shrink-0 items-center gap-1">
						<button
							type="button"
							class="btn btn-circle shrink-0 btn-ghost btn-sm"
							aria-label={journal.entryOrder === 'desc' ? 'Sort descending' : 'Sort ascending'}
							onclick={toggleEntryOrder}
						>
							{#if journal.entryOrder === 'desc'}
								<span class="icon-[lucide--arrow-down-wide-narrow] size-4"></span>
							{:else}
								<span class="icon-[lucide--arrow-up-narrow-wide] size-4"></span>
							{/if}
						</button>
						<button
							type="button"
							class="btn shrink-0 rounded-full btn-neutral btn-sm"
							onclick={() => void writeToday()}
							disabled={!selectedShelf || busy}
						>
							<span class="icon-[lucide--plus] size-4"></span>
							Note
						</button>
					</div>
				</div>
				<div class="flex-1 scrollbar-thin overflow-y-auto px-2 pb-4">
					{#each entries as entry (entry.id)}
						<button
							type="button"
							class={[
								'w-full truncate rounded-xl px-3 py-2.5 text-left text-sm font-medium transition-colors',
								selectedEntry?.id === entry.id ? 'bg-base-200' : 'hover:bg-base-200/70'
							]}
							aria-label={maskOn && selectedEntry?.id !== entry.id ? 'Note' : entry.title}
							onclick={() => selectEntry(entry.id)}
						>
							{#if maskOn && selectedEntry?.id !== entry.id}
								<span class="inline-block h-3 w-28 rounded-full bg-base-content/20"></span>
							{:else}
								{entry.title}
							{/if}
						</button>
					{:else}
						<p class="px-3 py-8 text-sm text-base-content/60">
							No entries yet. Press <kbd class="kbd kbd-sm">n</kbd> or Note.
						</p>
					{/each}
					{#if journal.hasMore}
						<div
							class="flex h-8 items-center justify-center"
							{@attach loadMoreSentinel(`${selectedShelf?.id ?? ''}:${entries.length}`)}
						>
							{#if journal.loadingMore}
								<p class="text-xs text-base-content/50">Loading…</p>
							{/if}
						</div>
					{/if}
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
						<button
							type="button"
							class="btn mt-4 rounded-full btn-neutral"
							onclick={() => void writeToday()}
							disabled={!selectedShelf || busy}
						>
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
		<button
			type="button"
			class="btn btn-neutral"
			onclick={() => void createShelf()}
			disabled={busy}
		>
			Create
		</button>
	{/snippet}
</Modal>

<Modal
	bind:open={editShelfOpen}
	title="Edit shelf"
	description="Shelf names are encrypted before they are stored."
>
	<form
		id="edit-shelf-form"
		class="flex flex-col gap-3"
		onsubmit={(event) => void saveShelf(event)}
	>
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
		<button type="submit" form="edit-shelf-form" class="btn btn-neutral" disabled={busy}
			>Save</button
		>
	{/snippet}
</Modal>

<Modal
	bind:open={deleteShelfOpen}
	title="Delete this shelf?"
	description="This deletes the shelf and every note in it from the server. Ciphertext only — the server never saw the words."
>
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (deleteShelfOpen = false)}>
			Cancel
		</button>
		<button
			type="button"
			class="btn btn-error"
			onclick={() => void confirmDeleteShelf()}
			disabled={busy}
		>
			Delete
		</button>
	{/snippet}
</Modal>

<Modal
	bind:open={deleteOpen}
	title="Delete this entry?"
	description="This deletes the encrypted note from the server. It cannot be undone."
>
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (deleteOpen = false)}>Cancel</button>
		<button
			type="button"
			class="btn btn-error"
			onclick={() => void confirmDelete()}
			disabled={busy}
		>
			Delete
		</button>
	{/snippet}
</Modal>

<WorkspaceEditModal
	bind:open={workspaceEditOpen}
	{workspace}
	bind:name={editName}
	bind:hint={editHint}
	bind:mask={editMask}
/>
