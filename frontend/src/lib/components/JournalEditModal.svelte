<script lang="ts">
	import Modal from '$lib/components/Modal.svelte';
	import ThemePicker from '$lib/components/ThemePicker.svelte';
	import JournalChangePassphraseModal from '$lib/components/JournalChangePassphraseModal.svelte';
	import { formatBytes, journal, type Journal } from '$lib/journal.svelte';
	import type { JournalTheme } from '$lib/theme';

	const NAME_MAX = 255;
	const HINT_MAX = 255;

	let {
		open = $bindable(false),
		currentJournal,
		name = $bindable(''),
		hint = $bindable(''),
		mask = $bindable(false),
		theme = $bindable('' as JournalTheme)
	}: {
		open?: boolean;
		currentJournal?: Journal;
		name?: string;
		hint?: string;
		mask?: boolean;
		theme?: JournalTheme;
	} = $props();

	let error = $state('');
	let busy = $state(false);
	let changeOpen = $state(false);

	async function save() {
		if (!currentJournal) return;
		const trimmed = name.trim();
		if (!trimmed) {
			error = 'Name the journal.';
			return;
		}
		busy = true;
		error = '';
		try {
			await journal.updateJournal(currentJournal.id, {
				name: trimmed,
				passphraseHint: hint,
				mask,
				theme
			});
			open = false;
		} catch (cause) {
			error = cause instanceof Error ? cause.message : 'Could not save the journal.';
		} finally {
			busy = false;
		}
	}
</script>

<Modal
	bind:open
	title="Edit journal"
	description="Name, hint, mask, and theme are stored as plaintext."
	theme={theme || undefined}
>
	<label class="w-full" for="journal-edit-name">
		<span class="mb-1 block text-sm">Name</span>
		<input
			id="journal-edit-name"
			name="name"
			class="input w-full"
			type="text"
			autocomplete="off"
			maxlength={NAME_MAX}
			bind:value={name}
			onkeydown={(event) => event.key === 'Enter' && !busy && void save()}
		/>
	</label>
	<label class="w-full" for="journal-edit-passphrase-hint">
		<span class="mb-1 block text-sm">Passphrase hint</span>
		<input
			id="journal-edit-passphrase-hint"
			name="passphrase-hint"
			class="input w-full"
			type="text"
			autocomplete="off"
			maxlength={HINT_MAX}
			bind:value={hint}
			onkeydown={(event) => event.key === 'Enter' && !busy && void save()}
		/>
		<span class="mt-1 block text-xs text-base-content/50">Leave empty to remove the hint.</span>
	</label>
	<label class="flex cursor-pointer items-start justify-between gap-3" for="journal-edit-mask">
		<span>
			<span class="block text-sm">Mask</span>
			<span class="mt-1 block text-xs text-base-content/50">
				Hide inactive notebook names and note titles. Icons and the open note stay visible.
			</span>
		</span>
		<input
			id="journal-edit-mask"
			name="mask"
			class="toggle shrink-0"
			type="checkbox"
			bind:checked={mask}
		/>
	</label>
	<ThemePicker bind:value={theme} />
	<button
		type="button"
		class="btn self-start btn-outline btn-sm"
		onclick={() => (changeOpen = true)}
	>
		Change passphrase
	</button>
	{#if currentJournal}
		<p class="text-sm text-base-content/70">
			<span class="mb-1 block text-sm text-base-content">Stored size</span>
			{#if currentJournal.sizeLastCalculatedAt}
				{formatBytes(currentJournal.totalSize)}
			{:else}
				Not calculated yet
			{/if}
		</p>
	{/if}
	{#if error}
		<p class="text-sm text-error">{error}</p>
	{/if}
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (open = false)}>Cancel</button>
		<button type="button" class="btn btn-neutral" onclick={() => void save()} disabled={busy}>
			{busy ? 'Saving…' : 'Save'}
		</button>
	{/snippet}
</Modal>

<JournalChangePassphraseModal bind:open={changeOpen} {currentJournal} />
