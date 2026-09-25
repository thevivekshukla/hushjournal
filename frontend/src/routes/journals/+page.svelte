<script lang="ts">
	import { goto } from '$app/navigation';
	import * as api from '$lib/api';
	import Loader from '$lib/components/Loader.svelte';
	import Modal from '$lib/components/Modal.svelte';
	import PassphraseField from '$lib/components/PassphraseField.svelte';
	import PassphraseStrength from '$lib/components/PassphraseStrength.svelte';
	import NoIndex from '$lib/components/NoIndex.svelte';
	import SiteName from '$lib/components/SiteName.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import UserMenu from '$lib/components/UserMenu.svelte';
	import { CryptoError, journal } from '$lib/journal.svelte';
	import { MIN_PASSPHRASE_LEN, minPassphraseLengthError } from '$lib/passphrase';
	import { session } from '$lib/session.svelte';
	import { onMount } from 'svelte';

	const HINT_MAX = 255;

	let createOpen = $state(false);
	let unlockOpen = $state(false);
	let pendingJournalId = $state<string | null>(null);
	let newName = $state('');
	let passphrase = $state('');
	let passphraseConfirm = $state('');
	let hint = $state('');
	let error = $state('');
	let busy = $state(false);
	let listReady = $state(false);

	const pendingJournal = $derived(
		pendingJournalId ? journal.getJournal(pendingJournalId) : undefined
	);

	onMount(() => {
		if (!session.user) {
			void goto(api.login());
			return;
		}
		let active = true;
		void journal.loadJournals().then(
			() => {
				if (active) listReady = true;
			},
			() => {
				if (active) listReady = false;
			}
		);
		return () => {
			active = false;
		};
	});

	function openUnlock(id: string) {
		pendingJournalId = id;
		passphrase = '';
		error = '';
		unlockOpen = true;
	}

	async function unlock() {
		if (!passphrase.trim() || !pendingJournal) {
			error = 'Enter the journal passphrase.';
			return;
		}
		busy = true;
		error = '';
		try {
			await journal.unlockJournal(pendingJournal, passphrase);
			unlockOpen = false;
			passphrase = '';
			void goto(api.journal(pendingJournal.id));
		} catch (cause) {
			error = cause instanceof CryptoError ? cause.message : 'Could not unlock this journal.';
		} finally {
			busy = false;
		}
	}

	async function create() {
		const name = newName.trim();
		if (!name) {
			error = 'Name the journal.';
			return;
		}
		if (!passphrase.trim()) {
			error = 'Choose a passphrase. It never leaves this device.';
			return;
		}
		if (passphrase.length < MIN_PASSPHRASE_LEN) {
			error = minPassphraseLengthError('Passphrase');
			return;
		}
		if (passphrase !== passphraseConfirm) {
			error = 'Passphrases do not match.';
			return;
		}
		busy = true;
		error = '';
		try {
			const created = await journal.createJournal(name, passphrase, hint);
			createOpen = false;
			newName = '';
			passphrase = '';
			passphraseConfirm = '';
			hint = '';
			void goto(api.journal(created.id));
		} catch (cause) {
			error = cause instanceof Error ? cause.message : 'Could not create the journal.';
		} finally {
			busy = false;
		}
	}

	function openCreate() {
		error = '';
		newName = '';
		passphrase = '';
		passphraseConfirm = '';
		hint = '';
		createOpen = true;
	}
</script>

<svelte:head>
	<title>Journals · HushJournal</title>
</svelte:head>

<NoIndex />

{#if session.user}
	<div class="mx-auto flex min-h-dvh w-full max-w-3xl flex-col px-5 py-6">
		<header class="flex items-center justify-between gap-3">
			<div>
				<SiteName />
				<p class="mt-2 text-sm text-base-content/60">Choose a journal to unlock.</p>
			</div>
			<div class="flex items-center gap-1">
				<ThemeToggle />
				<UserMenu />
			</div>
		</header>

		{#if listReady}
			<section class="mt-10 grid gap-4 sm:grid-cols-2">
				{#each journal.journals as item (item.id)}
					<button
						type="button"
						class="card cursor-pointer border border-base-300 bg-base-100 text-left text-base-content transition-colors hover:bg-base-200"
						data-theme={item.theme || undefined}
						onclick={() => openUnlock(item.id)}
					>
						<div class="card-body gap-3 p-5">
							<div class="flex items-start justify-between">
								<h2 class="font-serif text-2xl font-semibold tracking-tight">{item.name}</h2>
								<span class="icon-[lucide--lock-keyhole] size-5 text-base-content/50"></span>
							</div>
							{#if item.passphraseHint}
								<p class="text-sm text-base-content/60">Hint: {item.passphraseHint}</p>
							{:else}
								<p class="text-sm text-base-content/60">Passphrase stays on this device.</p>
							{/if}
						</div>
					</button>
				{/each}

				<button
					type="button"
					class="card cursor-pointer border border-dashed border-base-300 text-left text-base-content/70 transition-colors hover:border-base-content/30 hover:text-base-content"
					onclick={openCreate}
				>
					<div class="card-body items-start justify-center gap-2 p-5">
						<span class="icon-[lucide--plus] size-5"></span>
						<p class="font-medium">New journal</p>
					</div>
				</button>
			</section>
		{:else if journal.error}
			<p class="mt-10 text-sm text-error">{journal.error}</p>
		{:else}
			<div class="flex flex-1 flex-col items-center justify-center gap-3">
				<Loader label="Loading journals" />
				<p class="text-sm text-base-content/60">Loading journals…</p>
			</div>
		{/if}
	</div>
{/if}

<Modal
	bind:open={unlockOpen}
	title="Unlock {pendingJournal?.name ?? 'journal'}"
	description="The passphrase stays on this device. The server never sees it."
	theme={pendingJournal?.theme || undefined}
>
	{#if pendingJournal?.passphraseHint}
		<p class="text-sm text-base-content/70">Hint: {pendingJournal.passphraseHint}</p>
	{/if}
	<PassphraseField
		id="journal-unlock-passphrase"
		name="passphrase"
		label="Passphrase"
		autocomplete="current-password"
		autofocus
		bind:value={passphrase}
		onkeydown={(event) => event.key === 'Enter' && !busy && void unlock()}
	/>
	{#if error}
		<p class="text-sm text-error">{error}</p>
	{/if}
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (unlockOpen = false)}>Cancel</button>
		<button type="button" class="btn btn-neutral" onclick={() => void unlock()} disabled={busy}>
			{busy ? 'Unlocking…' : 'Unlock'}
		</button>
	{/snippet}
</Modal>

<Modal
	bind:open={createOpen}
	title="New journal"
	description="Give it a name and a passphrase. Both encryption keys are derived on this device."
>
	<label class="w-full" for="journal-name">
		<span class="mb-1 block text-sm">Name</span>
		<input
			id="journal-name"
			name="name"
			class="input w-full"
			type="text"
			autocomplete="off"
			bind:value={newName}
		/>
	</label>
	<div class="w-full">
		<PassphraseField
			id="journal-passphrase"
			name="new-passphrase"
			label="Passphrase"
			autocomplete="new-password"
			describedby={passphrase ? 'journal-passphrase-strength' : undefined}
			bind:value={passphrase}
			minlength={MIN_PASSPHRASE_LEN}
		/>
		<PassphraseStrength id="journal-passphrase-strength" value={passphrase} />
	</div>
	<PassphraseField
		id="journal-passphrase-confirm"
		name="new-passphrase-confirm"
		label="Confirm passphrase"
		autocomplete="new-password"
		bind:value={passphraseConfirm}
		onkeydown={(event) => event.key === 'Enter' && !busy && void create()}
		minlength={MIN_PASSPHRASE_LEN}
	/>
	<p class="rounded-lg bg-warning/10 px-3 py-2 text-sm text-warning">
		If you forget this passphrase, the content of this journal cannot be recovered.
	</p>
	<label class="w-full" for="journal-passphrase-hint">
		<span class="mb-1 block text-sm">Passphrase hint (optional)</span>
		<input
			id="journal-passphrase-hint"
			name="passphrase-hint"
			class="input w-full"
			type="text"
			autocomplete="off"
			maxlength={HINT_MAX}
			bind:value={hint}
		/>
		<span class="mt-1 block text-xs text-base-content/50">Shown before unlock. Not encrypted.</span>
	</label>
	{#if error}
		<p class="text-sm text-error">{error}</p>
	{/if}
	{#snippet footer()}
		<button type="button" class="btn btn-ghost" onclick={() => (createOpen = false)}>Cancel</button>
		<button type="button" class="btn btn-neutral" onclick={() => void create()} disabled={busy}>
			{busy ? 'Creating…' : 'Create'}
		</button>
	{/snippet}
</Modal>
