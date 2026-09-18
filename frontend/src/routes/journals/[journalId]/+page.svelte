<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import * as api from '$lib/api';
	import EntryEditor from '$lib/components/EntryEditor.svelte';
	import Modal from '$lib/components/Modal.svelte';
	import NoIndex from '$lib/components/NoIndex.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import UserMenu from '$lib/components/UserMenu.svelte';
	import JournalEditModal from '$lib/components/JournalEditModal.svelte';
	import {
		NOTEBOOK_ICONS,
		formatBytes,
		formatEntryDate,
		journal,
		type Notebook
	} from '$lib/journal.svelte';
	import { session } from '$lib/session.svelte';
	import { untrack } from 'svelte';
	import type { Attachment } from 'svelte/attachments';

	let selectedNotebookId = $state<string | null>(null);
	let selectedEntryId = $state<string | null>(null);
	let mobilePane = $state<'nav' | 'editor'>('nav');
	let notebookOpen = $state(false);
	let newNotebookName = $state('');
	let newNotebookIcon = $state<string>(NOTEBOOK_ICONS[0]);
	let editNotebookOpen = $state(false);
	let editingNotebookId = $state<string | null>(null);
	let editNotebookName = $state('');
	let editNotebookIcon = $state<string>(NOTEBOOK_ICONS[0]);
	let deleteOpen = $state(false);
	let deleteNotebookOpen = $state(false);
	let journalEditOpen = $state(false);
	let editName = $state('');
	let editHint = $state('');
	let editMask = $state(false);
	let busy = $state(false);

	const journalId = $derived(page.params.journalId ?? '');
	const activeJournal = $derived(journal.getJournal(journalId));
	const notebooks = $derived(journal.notebooksFor(journalId));
	const editingNotebook = $derived(
		notebooks.find((notebook) => notebook.id === editingNotebookId) ?? null
	);
	const selectedNotebook = $derived(
		notebooks.find((notebook) => notebook.id === selectedNotebookId) ?? null
	);
	const entries = $derived(selectedNotebook ? journal.entriesFor(selectedNotebook.id) : []);
	const selectedEntry = $derived(
		selectedEntryId ? (entries.find((entry) => entry.id === selectedEntryId) ?? null) : null
	);
	const maskOn = $derived(activeJournal?.mask === true);

	$effect(() => {
		if (!session.user) {
			void goto(api.login());
			return;
		}
		if (session.unlockedJournalId !== journalId) {
			void goto(api.journals());
		}
	});

	$effect(() => {
		if (session.unlockedJournalId !== journalId) return;
		const id = journalId;
		untrack(() => {
			void journal.loadNotebooks(id);
		});
	});

	$effect(() => {
		const notebookId = selectedNotebook?.id;
		if (!notebookId) return;
		untrack(() => {
			void journal.loadEntries(notebookId);
		});
	});

	$effect(() => {
		const id = selectedEntry?.id;
		if (!id) return;
		untrack(() => {
			void journal.ensureContent(id);
		});
	});

	function selectNotebook(id: string) {
		if (selectedNotebookId === id) return;
		void journal.flush();
		selectedNotebookId = id;
		selectedEntryId = null;
		mobilePane = 'nav';
	}

	function setEntryOrder(order: api.EntryOrder) {
		if (journal.entryOrder === order) return;
		journal.entryOrder = order;
		if (selectedNotebook) void journal.loadEntries(selectedNotebook.id);
	}

	function toggleEntryOrder() {
		setEntryOrder(journal.entryOrder === 'desc' ? 'asc' : 'desc');
	}

	async function loadMore() {
		if (!selectedNotebook || journal.loadingMore || busy) return;
		try {
			await journal.loadMore(selectedNotebook.id);
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
		if (!selectedNotebook) return;
		busy = true;
		try {
			await journal.flush();
			const entry = await journal.createEntry(selectedNotebook.id, formatEntryDate());
			selectedEntryId = entry.id;
			mobilePane = 'editor';
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not create a note.';
		} finally {
			busy = false;
		}
	}

	async function createNotebook() {
		const name = newNotebookName.trim();
		if (!name) return;
		busy = true;
		try {
			const notebook = await journal.createNotebook(journalId, name, newNotebookIcon);
			selectedNotebookId = notebook.id;
			selectedEntryId = null;
			newNotebookName = '';
			notebookOpen = false;
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not create the notebook.';
		} finally {
			busy = false;
		}
	}

	function openEditNotebook(notebook: Notebook, event?: MouseEvent) {
		event?.stopPropagation();
		editingNotebookId = notebook.id;
		editNotebookName = notebook.name;
		editNotebookIcon = notebook.icon;
		editNotebookOpen = true;
	}

	async function saveNotebook(event: SubmitEvent) {
		event.preventDefault();
		const name = editNotebookName.trim();
		if (!name || !editingNotebookId) return;
		busy = true;
		try {
			await journal.updateNotebook(editingNotebookId, { name, icon: editNotebookIcon });
			editNotebookOpen = false;
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not save the notebook.';
		} finally {
			busy = false;
		}
	}

	function requestDeleteNotebook() {
		editNotebookOpen = false;
		deleteNotebookOpen = true;
	}

	async function confirmDeleteNotebook() {
		if (!editingNotebookId) return;
		busy = true;
		try {
			if (selectedNotebookId === editingNotebookId) {
				selectedNotebookId = null;
				selectedEntryId = null;
			}
			await journal.deleteNotebook(editingNotebookId);
			deleteNotebookOpen = false;
			mobilePane = 'nav';
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not delete the notebook.';
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

	function openJournalEdit() {
		if (!activeJournal) return;
		editName = activeJournal.name;
		editHint = activeJournal.passphraseHint ?? '';
		editMask = activeJournal.mask;
		journalEditOpen = true;
	}

	function lockJournal() {
		void journal.flush();
		session.lock();
		journal.clearJournal();
		void goto(api.journals());
	}

	function onPageHide() {
		void journal.flush();
	}
</script>

<svelte:window onkeydown={onKeydown} onpagehide={onPageHide} />

<svelte:head>
	<title>{activeJournal?.name ?? 'Journal'} · e2ejournal</title>
</svelte:head>

<NoIndex />

{#if session.user && activeJournal && session.unlockedJournalId === journalId}
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
			<button type="button" class="btn gap-2 btn-ghost px-2 btn-sm" onclick={lockJournal}>
				<span class="icon-[lucide--lock-keyhole] size-4"></span>
				<span class="font-serif text-base tracking-tight">{activeJournal.name}</span>
			</button>
			<button
				type="button"
				class="btn btn-circle btn-ghost btn-sm"
				aria-label="Edit journal"
				onclick={openJournalEdit}
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
					<p class="text-xs tracking-wide text-base-content/50 uppercase">Notebooks</p>
					<button
						type="button"
						class="btn btn-circle btn-ghost btn-xs"
						aria-label="New notebook"
						onclick={() => (notebookOpen = true)}
					>
						<span class="icon-[lucide--plus] size-4"></span>
					</button>
				</div>
				<nav class="flex-1 scrollbar-thin overflow-y-auto px-2 pb-4">
					{#each notebooks as notebook (notebook.id)}
						<div
							class={[
								'group flex w-full items-center rounded-xl',
								selectedNotebook?.id === notebook.id ? 'bg-base-200' : 'hover:bg-base-200/70'
							]}
						>
							<button
								type="button"
								class={[
									'flex min-w-0 flex-1 items-center gap-2 rounded-xl px-3 py-2 text-left text-sm transition-colors',
									selectedNotebook?.id === notebook.id ? 'font-medium' : 'text-base-content/80'
								]}
								aria-label={maskOn && selectedNotebook?.id !== notebook.id
									? 'Notebook'
									: notebook.name}
								onclick={() => selectNotebook(notebook.id)}
							>
								<span class={[notebook.icon, 'size-4 shrink-0']}></span>
								{#if maskOn && selectedNotebook?.id !== notebook.id}
									<span class="inline-block h-3 w-28 rounded-full bg-base-content/20"></span>
								{:else}
									<span class="truncate">{notebook.name}</span>
								{/if}
							</button>
							<button
								type="button"
								class="btn mr-1 btn-circle btn-ghost opacity-0 btn-xs group-hover:opacity-100 focus-visible:opacity-100"
								aria-label="Edit notebook"
								onclick={(event) => openEditNotebook(notebook, event)}
							>
								<span class="icon-[lucide--pencil] size-3.5"></span>
							</button>
						</div>
					{/each}
					{#if notebooks.length === 0 && !journal.loading}
						<p class="px-3 py-6 text-sm text-base-content/60">
							Create a notebook to start writing.
						</p>
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
					{#each notebooks as notebook (notebook.id)}
						<button
							type="button"
							class={[
								'btn rounded-full btn-sm',
								selectedNotebook?.id === notebook.id ? 'btn-neutral' : 'btn-ghost'
							]}
							aria-label={maskOn && selectedNotebook?.id !== notebook.id
								? 'Notebook'
								: notebook.name}
							onclick={() => selectNotebook(notebook.id)}
						>
							<span class={[notebook.icon, 'size-4']}></span>
							{#if maskOn && selectedNotebook?.id !== notebook.id}
								<span class="inline-block h-3 w-16 rounded-full bg-base-content/20"></span>
							{:else}
								{notebook.name}
							{/if}
						</button>
					{/each}
					<button
						type="button"
						class="btn btn-circle btn-ghost btn-sm"
						aria-label="New notebook"
						onclick={() => (notebookOpen = true)}
					>
						<span class="icon-[lucide--plus] size-4"></span>
					</button>
				</div>
				<div class="flex items-center justify-between gap-2 px-3 pt-4 pb-2">
					<div class="flex min-w-0 flex-1 items-center gap-1">
						<p class="truncate text-sm font-medium">{selectedNotebook?.name ?? 'Entries'}</p>
						{#if selectedNotebook}
							<button
								type="button"
								class="btn btn-circle shrink-0 btn-ghost btn-xs"
								aria-label="Edit notebook"
								onclick={() => openEditNotebook(selectedNotebook)}
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
							disabled={!selectedNotebook || busy}
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
							{#if selectedNotebook}
								No entries yet. Press <kbd class="kbd kbd-sm">n</kbd> or Note.
							{:else}
								Pick a notebook to see notes.
							{/if}
						</p>
					{/each}
					{#if selectedNotebook && journal.hasMore}
						<div
							class="flex h-8 items-center justify-center"
							{@attach loadMoreSentinel(`${selectedNotebook?.id ?? ''}:${entries.length}`)}
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
						{#if selectedNotebook}
							<p class="mt-3 font-serif text-xl">Pick an entry, or start today’s page.</p>
							<button
								type="button"
								class="btn mt-4 rounded-full btn-neutral"
								onclick={() => void writeToday()}
								disabled={busy}
							>
								Write today
							</button>
						{:else}
							<p class="mt-3 font-serif text-xl">Pick a notebook to see notes.</p>
						{/if}
					</div>
				{/if}
			</main>
		</div>
	</div>
{/if}

<Modal
	bind:open={notebookOpen}
	title="New notebook"
	description="Notebook names are encrypted before they are stored."
>
	<label class="w-full" for="new-notebook-name">
		<span class="mb-1 block text-sm">Name</span>
		<input
			id="new-notebook-name"
			name="name"
			class="input w-full"
			type="text"
			autocomplete="off"
			bind:value={newNotebookName}
		/>
	</label>
	<div>
		<p class="mb-2 text-sm">Icon</p>
		<div class="flex flex-wrap gap-2">
			{#each NOTEBOOK_ICONS as icon (icon)}
				<button
					type="button"
					class={['btn btn-square btn-sm', newNotebookIcon === icon ? 'btn-neutral' : 'btn-ghost']}
					aria-label="Notebook icon"
					onclick={() => (newNotebookIcon = icon)}
				>
					<span class={[icon, 'size-4']}></span>
				</button>
			{/each}
		</div>
	</div>
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (notebookOpen = false)}
			>Cancel</button
		>
		<button
			type="button"
			class="btn btn-neutral"
			onclick={() => void createNotebook()}
			disabled={busy}
		>
			Create
		</button>
	{/snippet}
</Modal>

<Modal
	bind:open={editNotebookOpen}
	title="Edit notebook"
	description="Notebook names are encrypted before they are stored."
>
	<form
		id="edit-notebook-form"
		class="flex flex-col gap-3"
		onsubmit={(event) => void saveNotebook(event)}
	>
		<label class="w-full" for="edit-notebook-name">
			<span class="mb-1 block text-sm">Name</span>
			<!-- svelte-ignore a11y_autofocus -->
			<input
				id="edit-notebook-name"
				name="name"
				class="input w-full"
				type="text"
				placeholder="Name"
				autocomplete="off"
				bind:value={editNotebookName}
				autofocus
			/>
		</label>
		<div>
			<p class="mb-2 text-sm">Icon</p>
			<div class="flex flex-wrap gap-2">
				{#each NOTEBOOK_ICONS as icon (icon)}
					<button
						type="button"
						class={[
							'btn btn-square btn-sm',
							editNotebookIcon === icon ? 'btn-neutral' : 'btn-ghost'
						]}
						aria-label="Notebook icon"
						onclick={() => (editNotebookIcon = icon)}
					>
						<span class={[icon, 'size-4']}></span>
					</button>
				{/each}
			</div>
		</div>
	</form>
	{#if editingNotebook}
		<p class="text-sm text-base-content/70">
			<span class="mb-1 block text-sm text-base-content">Stored size</span>
			{#if editingNotebook.sizeLastCalculatedAt}
				{formatBytes(editingNotebook.totalSize)}
			{:else}
				Not calculated yet
			{/if}
		</p>
	{/if}
	{#snippet footer()}
		<button type="button" class="btn mr-auto btn-error" onclick={requestDeleteNotebook}>
			Delete
		</button>
		<button type="button" class="btn btn-ghost" onclick={() => (editNotebookOpen = false)}>
			Cancel
		</button>
		<button type="submit" form="edit-notebook-form" class="btn btn-neutral" disabled={busy}
			>Save</button
		>
	{/snippet}
</Modal>

<Modal
	bind:open={deleteNotebookOpen}
	title="Delete this notebook?"
	description="This deletes the notebook and every note in it from the server. Ciphertext only — the server never saw the words."
>
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (deleteNotebookOpen = false)}>
			Cancel
		</button>
		<button
			type="button"
			class="btn btn-error"
			onclick={() => void confirmDeleteNotebook()}
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

<JournalEditModal
	bind:open={journalEditOpen}
	currentJournal={activeJournal}
	bind:name={editName}
	bind:hint={editHint}
	bind:mask={editMask}
/>
