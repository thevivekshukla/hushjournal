<script lang="ts">
	import { parseDate, type DateValue } from '@internationalized/date';
	import { Calendar, Popover } from 'bits-ui';
	import { formatEntryDate, journal, parseIsoDate, type Entry } from '$lib/journal.svelte';

	let { entry, onDelete }: { entry: Entry; onDelete: () => void } = $props();

	let calendarOpen = $state(false);

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
	const selectedDate = $derived(parseDate(entry.entryDate));

	const stats = $derived.by(() => {
		const content = entry.content;
		return {
			words: content.trim() === '' ? 0 : content.trim().split(/\s+/).length,
			chars: content.length,
			lines: content === '' ? 0 : content.split('\n').length
		};
	});

	function updateTitle(event: Event) {
		const title = (event.currentTarget as HTMLInputElement).value;
		journal.updateEntry(entry.id, { title });
	}

	function updateContent(event: Event) {
		const content = (event.currentTarget as HTMLTextAreaElement).value;
		journal.updateEntry(entry.id, { content });
	}

	function selectDate(next: DateValue | undefined) {
		if (!next) return;
		calendarOpen = false;
		void journal.updateEntryDate(entry.id, next.toString());
	}
</script>

<div class="flex h-full min-h-0 flex-col">
	<div class="flex items-center justify-between gap-3 px-4 pt-3 pb-1 md:px-10">
		<div class="flex min-w-0 items-center gap-3">
			<Popover.Root bind:open={calendarOpen}>
				<Popover.Trigger
					type="button"
					class="cursor-pointer text-xs tracking-wide text-base-content/50"
					aria-label="Entry date, {dateLabel}"
				>
					{dateLabel}
				</Popover.Trigger>
				<Popover.Portal>
					<Popover.Content
						class="z-50 rounded-xl border border-base-300 bg-base-100 p-3 shadow-lg"
						sideOffset={8}
						align="start"
					>
						<Calendar.Root
							type="single"
							value={selectedDate}
							onValueChange={selectDate}
							weekdayFormat="short"
							fixedWeeks
							class="w-[19rem]"
						>
							{#snippet children({ months, weekdays })}
								<Calendar.Header class="flex items-center justify-between gap-1">
									<Calendar.PrevButton
										type="button"
										class="btn btn-ghost btn-square btn-sm"
										aria-label="Previous month"
									>
										<span class="icon-[lucide--chevron-left] size-4"></span>
									</Calendar.PrevButton>
									<div class="flex min-w-0 items-center justify-center gap-1">
										<Calendar.MonthSelect
											monthFormat="short"
											class="select select-ghost select-xs w-fit min-w-0"
										/>
										<Calendar.YearSelect class="select select-ghost select-xs w-fit min-w-0" />
									</div>
									<Calendar.NextButton
										type="button"
										class="btn btn-ghost btn-square btn-sm"
										aria-label="Next month"
									>
										<span class="icon-[lucide--chevron-right] size-4"></span>
									</Calendar.NextButton>
								</Calendar.Header>
								{#each months as month (month.value.toString())}
									<Calendar.Grid class="mt-2 w-full border-collapse select-none">
										<Calendar.GridHead>
											<Calendar.GridRow class="flex">
												{#each weekdays as day (day)}
													<Calendar.HeadCell
														class="w-8 pb-1 text-center text-[0.65rem] font-normal text-base-content/50"
													>
														{day.slice(0, 2)}
													</Calendar.HeadCell>
												{/each}
											</Calendar.GridRow>
										</Calendar.GridHead>
										<Calendar.GridBody>
											{#each month.weeks as week (week[0].toString())}
												<Calendar.GridRow class="flex">
													{#each week as date (date.toString())}
														<Calendar.Cell {date} month={month.value} class="p-0">
															<Calendar.Day
																class="inline-flex size-8 items-center justify-center rounded-lg text-sm hover:bg-base-200 data-disabled:pointer-events-none data-disabled:text-base-content/30 data-outside-month:text-base-content/30 data-selected:bg-neutral data-selected:text-neutral-content data-selected:hover:bg-neutral data-today:font-semibold"
															>
																{date.day}
															</Calendar.Day>
														</Calendar.Cell>
													{/each}
												</Calendar.GridRow>
											{/each}
										</Calendar.GridBody>
									</Calendar.Grid>
								{/each}
							{/snippet}
						</Calendar.Root>
					</Popover.Content>
				</Popover.Portal>
			</Popover.Root>
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
