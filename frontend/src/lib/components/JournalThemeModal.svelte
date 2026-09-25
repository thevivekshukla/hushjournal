<script lang="ts">
	import Modal from '$lib/components/Modal.svelte';
	import ThemePicker from '$lib/components/ThemePicker.svelte';
	import { journal, type Journal } from '$lib/journal.svelte';
	import type { JournalTheme } from '$lib/theme';

	let {
		open = $bindable(false),
		draft = $bindable('' as JournalTheme),
		currentJournal
	}: {
		open?: boolean;
		draft?: JournalTheme;
		currentJournal?: Journal;
	} = $props();

	let error = $state('');
	let busy = $state(false);

	async function save() {
		if (!currentJournal) return;
		busy = true;
		error = '';
		try {
			await journal.updateJournal(currentJournal.id, { theme: draft });
			open = false;
		} catch (cause) {
			error = cause instanceof Error ? cause.message : 'Could not save the theme.';
		} finally {
			busy = false;
		}
	}

	function close() {
		error = '';
		open = false;
	}
</script>

<Modal
	bind:open
	nested
	title="Theme"
	description="Colors for this journal. Stored as plaintext."
	theme={draft || undefined}
>
	<ThemePicker bind:value={draft} />
	{#if error}
		<p class="text-sm text-error">{error}</p>
	{/if}
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={close}>Cancel</button>
		<button type="button" class="btn btn-neutral" onclick={() => void save()} disabled={busy}>
			{busy ? 'Saving…' : 'Save'}
		</button>
	{/snippet}
</Modal>
