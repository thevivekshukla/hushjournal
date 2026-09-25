<script lang="ts">
	import { JOURNAL_THEMES, type JournalTheme } from '$lib/theme';

	let { value = $bindable('' as JournalTheme) }: { value?: JournalTheme } = $props();
</script>

<fieldset class="w-full min-w-0 border-0 p-0">
	<div class="grid grid-cols-2 gap-2 sm:grid-cols-3" role="radiogroup" aria-label="Theme">
		{#each JOURNAL_THEMES as item (item.id)}
			<button
				type="button"
				role="radio"
				class={[
					'rounded-xl border bg-base-100 p-2 text-left transition-colors',
					value === item.id ? 'border-primary ring-1 ring-primary' : 'border-base-300'
				]}
				data-theme={item.id || undefined}
				aria-label={item.label}
				aria-checked={value === item.id}
				onclick={() => (value = item.id)}
			>
				<span class="block truncate text-xs font-medium text-base-content">{item.label}</span>
				<span class="mt-2 flex gap-1">
					<span class="size-3 rounded-full bg-primary"></span>
					<span class="size-3 rounded-full bg-secondary"></span>
					<span class="size-3 rounded-full bg-accent"></span>
					<span class="size-3 rounded-full bg-neutral"></span>
				</span>
			</button>
		{/each}
	</div>
	<p class="mt-1 text-xs text-base-content/50">Colors change; corners stay silk and dim.</p>
</fieldset>
