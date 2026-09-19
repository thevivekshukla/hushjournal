<script lang="ts">
	import { Dialog } from 'bits-ui';
	import type { Snippet } from 'svelte';

	let {
		open = $bindable(false),
		title,
		description,
		nested = false,
		theme,
		children,
		footer
	}: {
		open?: boolean;
		title: string;
		description?: string;
		nested?: boolean;
		theme?: string;
		children?: Snippet;
		footer?: Snippet;
	} = $props();
</script>

<Dialog.Root bind:open>
	<Dialog.Portal>
		<Dialog.Overlay
			class={[
				'fixed inset-0 bg-base-300/70 backdrop-blur-[2px] data-[state=open]:opacity-100',
				nested ? 'z-[60]' : 'z-50'
			]}
			data-theme={theme || undefined}
		/>
		<Dialog.Content
			class={[
				'fixed top-1/2 left-1/2 flex max-h-[calc(100dvh-2rem)] w-[calc(100%-2rem)] max-w-md -translate-x-1/2 -translate-y-1/2 flex-col overflow-hidden rounded-2xl border border-base-300 bg-base-100 p-6 shadow-xl',
				nested ? 'z-[60]' : 'z-50'
			]}
			data-theme={theme || undefined}
		>
			<Dialog.Title class="shrink-0 pr-10 font-serif text-xl font-semibold tracking-tight"
				>{title}</Dialog.Title
			>
			{#if description}
				<Dialog.Description class="mt-1 shrink-0 text-sm text-base-content/70"
					>{description}</Dialog.Description
				>
			{/if}
			{#if children}
				<div class="mt-5 min-h-0 overflow-y-auto overscroll-contain">
					<div class="flex flex-col gap-3">
						{@render children()}
					</div>
				</div>
			{/if}
			{#if footer}
				<div class="mt-6 flex shrink-0 justify-end gap-2">
					{@render footer()}
				</div>
			{/if}
			<Dialog.Close
				class="btn absolute top-3 right-3 btn-circle btn-ghost btn-sm"
				aria-label="Close"
				type="button"
			>
				<span class="icon-[lucide--x] size-4"></span>
			</Dialog.Close>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
