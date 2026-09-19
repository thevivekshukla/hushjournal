<script lang="ts">
	import { beforeNavigate, goto } from '$app/navigation';
	import { page } from '$app/state';
	import * as api from '$lib/api';
	import Loader from '$lib/components/Loader.svelte';
	import Modal from '$lib/components/Modal.svelte';
	import NoIndex from '$lib/components/NoIndex.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import UserMenu from '$lib/components/UserMenu.svelte';
	import JournalEditModal from '$lib/components/JournalEditModal.svelte';
	import {
		CryptoError,
		NOTEBOOK_ICONS,
		formatBytes,
		formatEntryDate,
		journal,
		type Notebook
	} from '$lib/journal.svelte';
	import { session } from '$lib/session.svelte';
	import { restoreAppTheme, type JournalTheme } from '$lib/theme';
	import { untrack } from 'svelte';
	import type { Attachment } from 'svelte/attachments';

	let { children } = $props();

	let notebookOpen = $state(false);
	let newNotebookName = $state('');
	let newNotebookIcon = $state<string>(NOTEBOOK_ICONS[0]);
	let editNotebookOpen = $state(false);
	let editingNotebookId = $state<string | null>(null);
	let editNotebookName = $state('');
	let editNotebookIcon = $state<string>(NOTEBOOK_ICONS[0]);
	let deleteNotebookOpen = $state(false);
	let journalEditOpen = $state(false);
	let editName = $state('');
	let editHint = $state('');
	let editMask = $state(false);
	let editTheme = $state<JournalTheme>('');
	let busy = $state(false);
	let passphrase = $state('');
	let unlockError = $state('');
	let unlockBusy = $state(false);

	const journalId = $derived(page.params.journalId ?? '');
	const notebookId = $derived(page.params.notebookId ?? '');
	const entryId = $derived(page.params.entryId ?? '');
	const activeJournal = $derived(journal.getJournal(journalId));
	const notebooks = $derived(journal.notebooksFor(journalId));
	const editingNotebook = $derived(
		notebooks.find((notebook) => notebook.id === editingNotebookId) ?? null
	);
	const selectedNotebook = $derived(
		notebookId ? (notebooks.find((notebook) => notebook.id === notebookId) ?? null) : null
	);
	const entries = $derived(selectedNotebook ? journal.entriesFor(selectedNotebook.id) : []);
	const maskOn = $derived(activeJournal?.mask === true);
	const journalThemeId = $derived(activeJournal?.theme || undefined);
	const unlocked = $derived(
		Boolean(session.user && activeJournal && session.unlockedJournalId === journalId)
	);

	beforeNavigate(({ from, to }) => {
		const fromEntry = from?.params?.entryId;
		const toEntry = to?.params?.entryId;
		if (fromEntry && fromEntry !== toEntry) {
			void journal.flush(fromEntry);
			return;
		}
		const fromJournal = from?.params?.journalId;
		const toJournal = to?.params?.journalId;
		if (fromJournal && fromJournal !== toJournal) {
			void journal.flush();
		}
	});

	$effect(() => {
		if (!session.user) {
			void goto(api.login());
			return;
		}
		if (session.unlockedJournalId === journalId) return;
		const id = journalId;
		untrack(() => {
			if (session.unlockedJournalId) {
				void journal.flush();
				session.lock();
				journal.clearJournal();
			}
			void journal
				.loadJournal(id)
				.then((loaded) => {
					if (!loaded && page.params.journalId === id) {
						void goto(api.journals(), { replaceState: true });
					}
				})
				.catch(() => {
					// loadJournal already records journal.error
				});
		});
	});

	$effect(() => {
		if (session.unlockedJournalId !== journalId) return;
		const id = journalId;
		untrack(() => {
			void journal.loadNotebooks(id);
		});
	});

	$effect(() => {
		const id = selectedNotebook?.id;
		if (!id) return;
		untrack(() => {
			void journal.loadEntries(id);
		});
	});

	$effect(() => {
		if (session.unlockedJournalId !== journalId) return;
		const id = notebookId;
		if (!id) return;
		if (journal.notebooksLoadedFor !== journalId) return;
		if (notebooks.some((notebook) => notebook.id === id)) return;
		void goto(api.journal(journalId), { replaceState: true });
	});

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

	function fetchEntry(id: string) {
		void journal.loadEntry(id, { force: true });
	}

	function onEntryPointerDown(event: PointerEvent, id: string) {
		if (event.button !== 0) return;
		fetchEntry(id);
	}

	function onEntryClick(event: MouseEvent, id: string) {
		if (event.detail !== 0) return;
		fetchEntry(id);
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

	async function writeToday() {
		if (!selectedNotebook) return;
		busy = true;
		try {
			await journal.flush();
			const entry = await journal.createEntry(selectedNotebook.id, formatEntryDate());
			await goto(api.journalEntry(journalId, selectedNotebook.id, entry.id));
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
			newNotebookName = '';
			notebookOpen = false;
			await goto(api.journalNotebook(journalId, notebook.id));
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not create the notebook.';
		} finally {
			busy = false;
		}
	}

	function openEditNotebook(notebook: Notebook, event?: MouseEvent) {
		event?.preventDefault();
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
		const deletedId = editingNotebookId;
		busy = true;
		try {
			await journal.deleteNotebook(deletedId);
			deleteNotebookOpen = false;
			if (notebookId === deletedId) {
				await goto(api.journal(journalId), { replaceState: true });
			}
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not delete the notebook.';
		} finally {
			busy = false;
		}
	}

	function onKeydown(event: KeyboardEvent) {
		const target = event.target as HTMLElement | null;
		if (target && ['INPUT', 'TEXTAREA'].includes(target.tagName)) return;
		if (!unlocked) return;
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
		editTheme = activeJournal.theme;
		journalEditOpen = true;
	}

	function lockJournal() {
		void journal.flush();
		session.lock();
		journal.clearJournal();
		void goto(api.journals());
	}

	async function unlockJournal() {
		if (!passphrase.trim() || !activeJournal) {
			unlockError = 'Enter the journal passphrase.';
			return;
		}
		unlockBusy = true;
		unlockError = '';
		try {
			await journal.unlockJournal(activeJournal, passphrase);
			passphrase = '';
		} catch (cause) {
			unlockError = cause instanceof CryptoError ? cause.message : 'Could not unlock this journal.';
		} finally {
			unlockBusy = false;
		}
	}

	function onPageHide() {
		void journal.flush();
	}

	$effect(() => {
		const theme = journalThemeId;
		if (typeof document === 'undefined') return;
		if (theme) document.documentElement.setAttribute('data-theme', theme);
		else restoreAppTheme();
		return () => restoreAppTheme();
	});
</script>

<svelte:window onkeydown={onKeydown} onpagehide={onPageHide} />

<svelte:head>
	<title>{activeJournal?.name ?? 'Journal'} · e2ejournal</title>
</svelte:head>

<NoIndex />

{#if session.user && !unlocked}
	<div
		class="mx-auto flex min-h-dvh w-full max-w-md flex-col bg-base-100 px-5 py-6 text-base-content"
		data-theme={journalThemeId}
	>
		<header class="flex items-center justify-between gap-3">
			<button
				type="button"
				class="btn gap-2 btn-ghost px-2 btn-sm"
				onclick={() => goto(api.journals())}
			>
				<span class="icon-[lucide--arrow-left] size-4"></span>
				<span>Journals</span>
			</button>
			<div class="flex items-center gap-1">
				{#if !journalThemeId}
					<ThemeToggle />
				{/if}
				<UserMenu compact />
			</div>
		</header>
		<div class="flex flex-1 flex-col justify-center pb-16">
			{#if journal.loading && !activeJournal}
				<p class="text-center text-sm text-base-content/60">Loading journal…</p>
			{:else if journal.error && !activeJournal}
				<p class="text-center text-sm text-error">{journal.error}</p>
			{:else if activeJournal}
				<h1 class="font-serif text-3xl tracking-tight">{activeJournal.name}</h1>
				<p class="mt-2 text-sm text-base-content/60">
					Enter the passphrase to unlock this journal. It stays on this device.
				</p>
				<form
					class="mt-8 flex flex-col gap-4"
					onsubmit={(event) => {
						event.preventDefault();
						void unlockJournal();
					}}
				>
					{#if activeJournal.passphraseHint}
						<p class="text-sm text-base-content/70">Hint: {activeJournal.passphraseHint}</p>
					{/if}
					<label class="w-full" for="deep-journal-unlock-passphrase">
						<span class="mb-1 block text-sm">Passphrase</span>
						<!-- svelte-ignore a11y_autofocus -->
						<input
							id="deep-journal-unlock-passphrase"
							name="passphrase"
							class="input w-full"
							type="password"
							autocomplete="current-password"
							bind:value={passphrase}
							autofocus
						/>
					</label>
					{#if unlockError}
						<p class="text-sm text-error">{unlockError}</p>
					{/if}
					<button type="submit" class="btn rounded-full btn-neutral" disabled={unlockBusy}>
						{unlockBusy ? 'Unlocking…' : 'Unlock'}
					</button>
				</form>
			{/if}
		</div>
	</div>
{:else if unlocked && activeJournal}
	<div
		class="flex h-dvh min-h-0 flex-col bg-base-100 text-base-content"
		data-theme={journalThemeId}
	>
		<header class="flex h-14 shrink-0 items-center gap-2 border-b border-base-300 px-3">
			{#if entryId && notebookId}
				<a
					href={api.journalNotebook(journalId, notebookId)}
					data-sveltekit-noscroll
					class="btn btn-circle btn-ghost btn-sm md:hidden"
					aria-label="Back to entries"
				>
					<span class="icon-[lucide--arrow-left] size-4"></span>
				</a>
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
				{#if !journalThemeId}
					<ThemeToggle />
				{/if}
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
							<a
								href={api.journalNotebook(journalId, notebook.id)}
								data-sveltekit-noscroll
								class={[
									'flex min-w-0 flex-1 items-center gap-2 rounded-xl px-3 py-2 text-left text-sm transition-colors',
									selectedNotebook?.id === notebook.id ? 'font-medium' : 'text-base-content/80'
								]}
								aria-current={selectedNotebook?.id === notebook.id ? 'page' : undefined}
								aria-label={maskOn && selectedNotebook?.id !== notebook.id
									? 'Notebook'
									: notebook.name}
							>
								<span class={[notebook.icon, 'size-4 shrink-0']}></span>
								{#if maskOn && selectedNotebook?.id !== notebook.id}
									<span class="inline-block h-3 w-28 rounded-full bg-base-content/20"></span>
								{:else}
									<span class="truncate">{notebook.name}</span>
								{/if}
							</a>
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
					entryId ? 'hidden md:flex' : 'flex'
				]}
			>
				<div class="flex gap-2 overflow-x-auto px-3 pt-3 md:hidden">
					{#each notebooks as notebook (notebook.id)}
						<a
							href={api.journalNotebook(journalId, notebook.id)}
							data-sveltekit-noscroll
							class={[
								'btn rounded-full btn-sm',
								selectedNotebook?.id === notebook.id ? 'btn-neutral' : 'btn-ghost'
							]}
							aria-current={selectedNotebook?.id === notebook.id ? 'page' : undefined}
							aria-label={maskOn && selectedNotebook?.id !== notebook.id
								? 'Notebook'
								: notebook.name}
						>
							<span class={[notebook.icon, 'size-4']}></span>
							{#if maskOn && selectedNotebook?.id !== notebook.id}
								<span class="inline-block h-3 w-16 rounded-full bg-base-content/20"></span>
							{:else}
								{notebook.name}
							{/if}
						</a>
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
					{#if journal.loadingEntries && entries.length === 0}
						<div class="flex h-full items-center justify-center py-16">
							<Loader label="Loading notes" />
						</div>
					{:else}
						{#each entries as entry (entry.id)}
							<a
								href={selectedNotebook
									? api.journalEntry(journalId, selectedNotebook.id, entry.id)
									: api.journal(journalId)}
								data-sveltekit-noscroll
								class={[
									'block w-full truncate rounded-xl px-3 py-2.5 text-left text-sm font-medium transition-colors',
									entryId === entry.id ? 'bg-base-200' : 'hover:bg-base-200/70'
								]}
								aria-current={entryId === entry.id ? 'page' : undefined}
								aria-label={maskOn && entryId !== entry.id ? 'Note' : entry.title}
								onpointerdown={(event) => onEntryPointerDown(event, entry.id)}
								onclick={(event) => onEntryClick(event, entry.id)}
							>
								{#if maskOn && entryId !== entry.id}
									<span class="inline-block h-3 w-28 rounded-full bg-base-content/20"></span>
								{:else}
									{entry.title}
								{/if}
							</a>
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
									<Loader label="Loading more notes" size="sm" />
								{/if}
							</div>
						{/if}
					{/if}
				</div>
			</aside>

			<main class={['flex min-h-0 min-w-0 flex-1 flex-col', entryId ? 'flex' : 'hidden md:flex']}>
				{@render children()}
			</main>
		</div>
	</div>
{/if}

<Modal
	bind:open={notebookOpen}
	title="New notebook"
	description="Notebook names are encrypted before they are stored."
	theme={journalThemeId}
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
	theme={journalThemeId}
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
	theme={journalThemeId}
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

<JournalEditModal
	bind:open={journalEditOpen}
	currentJournal={activeJournal}
	bind:name={editName}
	bind:hint={editHint}
	bind:mask={editMask}
	bind:theme={editTheme}
/>
