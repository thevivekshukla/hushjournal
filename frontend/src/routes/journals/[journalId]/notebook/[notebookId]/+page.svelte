<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import * as api from '$lib/api';
	import { formatEntryDate, journal } from '$lib/journal.svelte';

	let busy = $state(false);

	const journalId = $derived(page.params.journalId ?? '');
	const notebookId = $derived(page.params.notebookId ?? '');

	async function writeToday() {
		if (!notebookId) return;
		busy = true;
		try {
			await journal.flush();
			const entry = await journal.createEntry(notebookId, formatEntryDate());
			await goto(api.journalEntry(journalId, notebookId, entry.id));
		} catch (cause) {
			journal.error = cause instanceof Error ? cause.message : 'Could not create a note.';
		} finally {
			busy = false;
		}
	}
</script>

<div class="flex flex-1 flex-col items-center justify-center px-6 text-center">
	<span class="icon-[lucide--pen-line] size-8 text-base-content/30"></span>
	<p class="mt-3 font-serif text-xl">Pick an entry, or start today’s page.</p>
	<button
		type="button"
		class="btn mt-4 rounded-full btn-neutral"
		onclick={() => void writeToday()}
		disabled={busy}
	>
		Write today
	</button>
</div>
