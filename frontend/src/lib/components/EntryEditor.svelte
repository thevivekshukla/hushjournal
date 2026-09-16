<script lang="ts">
	import { journal, type Entry } from '$lib/journal.svelte';

	let { entry, onDelete }: { entry: Entry; onDelete: () => void } = $props();
	let status = $state('Saved');

	function updateTitle(event: Event) {
		const title = (event.currentTarget as HTMLInputElement).value;
		status = 'Saved';
		journal.updateEntry(entry.id, { title });
	}

	function updateContent(event: Event) {
		const content = (event.currentTarget as HTMLTextAreaElement).value;
		status = 'Saved';
		journal.updateEntry(entry.id, { content });
	}
</script>

<div class="flex h-full min-h-0 flex-col">
	<div class="flex items-center justify-between gap-3 px-4 pt-3 pb-1 md:px-10">
		<p class="text-xs tracking-wide text-base-content/50 uppercase">{status}</p>
		<button type="button" class="btn btn-ghost text-error btn-sm" onclick={onDelete}>
			<span class="icon-[lucide--trash-2] size-4"></span>
			<span class="hidden sm:inline">Delete</span>
		</button>
	</div>
	<input
		class="journal-title w-full border-0 bg-transparent px-4 font-serif text-3xl font-semibold tracking-tight outline-none md:px-10"
		value={entry.title}
		oninput={updateTitle}
		placeholder="Title"
		aria-label="Entry title"
	/>
	<textarea
		class="journal-body min-h-0 w-full flex-1 resize-none border-0 bg-transparent px-4 pt-2 pb-16 font-serif text-lg leading-8 outline-none md:px-10"
		value={entry.content}
		oninput={updateContent}
		placeholder="Start writing…"
		aria-label="Entry content"></textarea>
</div>
