<script lang="ts">
	import Modal from '$lib/components/Modal.svelte';
	import { CryptoError, journal, type Journal } from '$lib/journal.svelte';

	let {
		open = $bindable(false),
		currentJournal
	}: {
		open?: boolean;
		currentJournal?: Journal;
	} = $props();

	let currentPassphrase = $state('');
	let newPassphrase = $state('');
	let confirmPassphrase = $state('');
	let error = $state('');
	let busy = $state(false);

	$effect(() => {
		if (open) return;
		currentPassphrase = '';
		newPassphrase = '';
		confirmPassphrase = '';
		error = '';
	});

	async function save() {
		if (!currentJournal) return;
		if (!currentPassphrase) {
			error = 'Enter the current passphrase.';
			return;
		}
		if (!newPassphrase.trim()) {
			error = 'Choose a new passphrase. It never leaves this device.';
			return;
		}
		if (newPassphrase !== confirmPassphrase) {
			error = 'New passphrases do not match.';
			return;
		}
		if (newPassphrase === currentPassphrase) {
			error = 'Choose a different passphrase.';
			return;
		}
		busy = true;
		error = '';
		try {
			await journal.changeJournalPassphrase(currentJournal.id, currentPassphrase, newPassphrase);
			open = false;
		} catch (cause) {
			error =
				cause instanceof CryptoError ? cause.message : 'Could not change the journal passphrase.';
		} finally {
			busy = false;
		}
	}
</script>

<Modal
	bind:open
	nested
	title="Change passphrase"
	description="The passphrase stays on this device. Notes keep the same encryption key."
	theme={currentJournal?.theme || undefined}
>
	<label class="w-full" for="journal-change-current-passphrase">
		<span class="mb-1 block text-sm">Current passphrase</span>
		<input
			id="journal-change-current-passphrase"
			name="current-passphrase"
			class="input w-full"
			type="password"
			autocomplete="current-password"
			bind:value={currentPassphrase}
			onkeydown={(event) => event.key === 'Enter' && !busy && void save()}
		/>
	</label>
	<label class="w-full" for="journal-change-new-passphrase">
		<span class="mb-1 block text-sm">New passphrase</span>
		<input
			id="journal-change-new-passphrase"
			name="new-passphrase"
			class="input w-full"
			type="password"
			autocomplete="new-password"
			bind:value={newPassphrase}
			onkeydown={(event) => event.key === 'Enter' && !busy && void save()}
		/>
	</label>
	<label class="w-full" for="journal-change-confirm-passphrase">
		<span class="mb-1 block text-sm">Confirm new passphrase</span>
		<input
			id="journal-change-confirm-passphrase"
			name="new-passphrase-confirm"
			class="input w-full"
			type="password"
			autocomplete="new-password"
			bind:value={confirmPassphrase}
			onkeydown={(event) => event.key === 'Enter' && !busy && void save()}
		/>
	</label>
	<p class="rounded-lg bg-warning/10 px-3 py-2 text-sm text-warning">
		If you forget this passphrase, the content of this journal cannot be recovered.
	</p>
	{#if error}
		<p class="text-sm text-error">{error}</p>
	{/if}
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (open = false)}>Cancel</button>
		<button type="button" class="btn btn-neutral" onclick={() => void save()} disabled={busy}>
			{busy ? 'Changing…' : 'Change passphrase'}
		</button>
	{/snippet}
</Modal>
