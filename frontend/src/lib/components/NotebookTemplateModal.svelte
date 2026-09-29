<script lang="ts">
	import Modal from '$lib/components/Modal.svelte';
	import { journal, type Notebook } from '$lib/journal.svelte';

	let {
		open = $bindable(false),
		draft = $bindable(''),
		notebook,
		theme
	}: {
		open?: boolean;
		draft?: string;
		notebook?: Notebook;
		theme?: string;
	} = $props();

	let error = $state('');
	let busy = $state(false);

	async function save() {
		if (!notebook) return;
		busy = true;
		error = '';
		try {
			const text = draft;
			await journal.updateNotebook(notebook.id, {
				templateEntryContent: text.length > 0 ? text : null
			});
			open = false;
		} catch (cause) {
			error = cause instanceof Error ? cause.message : 'Could not save the template.';
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
	title="Template"
	description="New notes in this notebook start with this text. Stored encrypted. Leave empty to start blank."
	{theme}
>
	<label class="w-full" for="notebook-template">
		<span class="sr-only">Template</span>
		<textarea
			id="notebook-template"
			name="template"
			class="textarea min-h-48 w-full font-serif"
			bind:value={draft}
			placeholder="Start new notes with…"></textarea>
	</label>
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
