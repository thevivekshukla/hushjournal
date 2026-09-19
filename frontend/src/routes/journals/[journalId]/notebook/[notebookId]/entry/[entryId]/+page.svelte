<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import * as api from '$lib/api';
	import EntryEditor from '$lib/components/EntryEditor.svelte';
	import Modal from '$lib/components/Modal.svelte';
	import { journal } from '$lib/journal.svelte';
	import { untrack } from 'svelte';

	let deleteOpen = $state(false);
	let busy = $state(false);

	const journalId = $derived(page.params.journalId ?? '');
	const notebookId = $derived(page.params.notebookId ?? '');
	const entryId = $derived(page.params.entryId ?? '');
	const entry = $derived(entryId ? (journal.entry(entryId) ?? null) : null);

	$effect(() => {
		const id = entryId;
		const notebook = notebookId;
		const journalKey = journalId;
		const notebooksReady = journal.notebooksLoadedFor === journalKey;
		if (!id || !notebook || !journalKey) return;
		untrack(() => {
			void (async () => {
				try {
					const loaded = await journal.loadEntry(id);
					if (page.params.entryId !== id) return;
					if (!loaded) {
						void goto(api.journalNotebook(journalKey, notebook), { replaceState: true });
						return;
					}
					if (loaded.notebookId === notebook) return;
					if (!notebooksReady) return;
					const owner = journal
						.notebooksFor(journalKey)
						.find((item) => item.id === loaded.notebookId);
					void goto(owner ? api.journalEntry(journalKey, owner.id, id) : api.journal(journalKey), {
						replaceState: true
					});
				} catch {
					// loadEntry already records journal.error
				}
			})();
		});
	});

	async function confirmDelete() {
		if (!entry) return;
		busy = true;
		try {
			await journal.deleteEntry(entry.id);
			deleteOpen = false;
			await goto(api.journalNotebook(journalId, notebookId), { replaceState: true });
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not delete the note.';
		} finally {
			busy = false;
		}
	}
</script>

{#if entry}
	{#key entry.id}
		<EntryEditor {entry} onDelete={() => (deleteOpen = true)} />
	{/key}
{:else}
	<div class="flex flex-1 items-center justify-center text-sm text-base-content/50">Loading…</div>
{/if}

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
