<script lang="ts">
	import Modal from '$lib/components/Modal.svelte';
	import WorkspaceChangePassphraseModal from '$lib/components/WorkspaceChangePassphraseModal.svelte';
	import { journal, type Workspace } from '$lib/journal.svelte';

	const NAME_MAX = 255;
	const HINT_MAX = 255;

	let {
		open = $bindable(false),
		workspace,
		name = $bindable(''),
		hint = $bindable(''),
		mask = $bindable(false)
	}: {
		open?: boolean;
		workspace?: Workspace;
		name?: string;
		hint?: string;
		mask?: boolean;
	} = $props();

	let error = $state('');
	let busy = $state(false);
	let changeOpen = $state(false);

	async function save() {
		if (!workspace) return;
		const trimmed = name.trim();
		if (!trimmed) {
			error = 'Name the workspace.';
			return;
		}
		busy = true;
		error = '';
		try {
			await journal.updateWorkspace(workspace.id, {
				name: trimmed,
				passphraseHint: hint,
				mask
			});
			open = false;
		} catch (cause) {
			error = cause instanceof Error ? cause.message : 'Could not save the workspace.';
		} finally {
			busy = false;
		}
	}
</script>

<Modal bind:open title="Edit workspace" description="Name, hint, and mask are stored as plaintext.">
	<label class="w-full" for="workspace-edit-name">
		<span class="mb-1 block text-sm">Name</span>
		<input
			id="workspace-edit-name"
			name="name"
			class="input w-full"
			type="text"
			autocomplete="off"
			maxlength={NAME_MAX}
			bind:value={name}
			onkeydown={(event) => event.key === 'Enter' && !busy && void save()}
		/>
	</label>
	<label class="w-full" for="workspace-edit-passphrase-hint">
		<span class="mb-1 block text-sm">Passphrase hint</span>
		<input
			id="workspace-edit-passphrase-hint"
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
	<label class="flex cursor-pointer items-start justify-between gap-3" for="workspace-edit-mask">
		<span>
			<span class="block text-sm">Mask</span>
			<span class="mt-1 block text-xs text-base-content/50">
				Hide inactive shelf names and note titles. Icons and the open note stay visible.
			</span>
		</span>
		<input
			id="workspace-edit-mask"
			name="mask"
			class="toggle shrink-0"
			type="checkbox"
			bind:checked={mask}
		/>
	</label>
	<button
		type="button"
		class="btn self-start btn-outline btn-sm"
		onclick={() => (changeOpen = true)}
	>
		Change passphrase
	</button>
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

<WorkspaceChangePassphraseModal bind:open={changeOpen} {workspace} />
