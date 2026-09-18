<script lang="ts">
	import { formatEntryDate, journal, parseIsoDate, type Entry } from '$lib/journal.svelte';

	let { entry, onDelete }: { entry: Entry; onDelete: () => void } = $props();

	const status = $derived(
		!entry.contentLoaded
			? 'Loading…'
			: entry.saveStatus === 'saving'
				? 'Saving…'
				: entry.saveStatus === 'error'
					? 'Couldn’t save'
					: 'Saved'
	);

	const dateLabel = $derived(formatEntryDate(parseIsoDate(entry.entryDate)));

	const stats = $derived.by(() => {
		const content = entry.content;
		return {
			words: content.trim() === '' ? 0 : content.trim().split(/\s+/).length,
			chars: content.length,
			lines: content === '' ? 0 : content.split('\n').length
		};
	});

	let dateInput: HTMLInputElement | undefined;

	function dateRef(node: HTMLInputElement) {
		dateInput = node;
		return () => {
			dateInput = undefined;
		};
	}

	function updateTitle(event: Event) {
		const title = (event.currentTarget as HTMLInputElement).value;
		journal.updateEntry(entry.id, { title });
	}

	function updateContent(event: Event) {
		const content = (event.currentTarget as HTMLTextAreaElement).value;
		journal.updateEntry(entry.id, { content });
	}

	function openCalendar() {
		if (!dateInput) return;
		try {
			dateInput.showPicker();
		} catch {
			dateInput.focus();
		}
	}

	function updateDate(event: Event) {
		const value = (event.currentTarget as HTMLInputElement).value;
		if (!value) return;
		void journal.updateEntryDate(entry.id, value);
	}
</script>

<div class="flex h-full min-h-0 flex-col">
	<div class="flex items-center justify-between gap-3 px-4 pt-3 pb-1 md:px-10">
		<div class="flex min-w-0 items-center gap-3">
			<div class="relative inline-flex">
				<button
					type="button"
					class="cursor-pointer text-xs tracking-wide text-base-content/50"
					onclick={openCalendar}
					aria-label="Entry date, {dateLabel}"
				>
					{dateLabel}
				</button>
				<input
					{@attach dateRef}
					class="pointer-events-none absolute inset-0 opacity-0"
					type="date"
					value={entry.entryDate}
					onchange={updateDate}
					tabindex="-1"
					aria-hidden="true"
				/>
			</div>
			<p class="text-xs tracking-wide text-base-content/50 uppercase">{status}</p>
		</div>
		<button type="button" class="btn btn-ghost text-error btn-sm" onclick={onDelete}>
			<span class="icon-[lucide--trash-2] size-4"></span>
			<span class="hidden sm:inline">Delete</span>
		</button>
	</div>
	<input
		id="entry-title"
		name="title"
		class="journal-title w-full border-0 bg-transparent px-4 font-serif text-3xl font-semibold tracking-tight outline-none md:px-10"
		value={entry.title}
		oninput={updateTitle}
		placeholder="Title"
		aria-label="Entry title"
		autocomplete="off"
		disabled={!entry.contentLoaded}
	/>
	<textarea
		id="entry-content"
		name="content"
		class="journal-body min-h-0! w-full flex-1 resize-none border-0 bg-transparent px-4 pt-2 pb-4 font-serif text-lg leading-8 outline-none md:px-10"
		value={entry.content}
		oninput={updateContent}
		placeholder="Start writing…"
		aria-label="Entry content"
		disabled={!entry.contentLoaded}></textarea>
	<p
		class="shrink-0 px-4 py-2 text-xs tracking-wide text-base-content/50 tabular-nums md:px-10"
		aria-live="polite"
	>
		{stats.words}
		{stats.words === 1 ? 'word' : 'words'}
		<span aria-hidden="true"> · </span>
		{stats.chars}
		{stats.chars === 1 ? 'char' : 'chars'}
		<span aria-hidden="true"> · </span>
		{stats.lines}
		{stats.lines === 1 ? 'line' : 'lines'}
	</p>
</div>
