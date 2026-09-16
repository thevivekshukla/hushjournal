<script lang="ts">
	import { Dialog } from 'bits-ui';
	import type { Snippet } from 'svelte';

	let {
		open = $bindable(false),
		title,
		description,
		children,
		footer
	}: {
		open?: boolean;
		title: string;
		description?: string;
		children?: Snippet;
		footer?: Snippet;
	} = $props();
</script>

<Dialog.Root bind:open>
	<Dialog.Portal>
		<Dialog.Overlay
			class="fixed inset-0 z-50 bg-base-300/70 backdrop-blur-[2px] data-[state=open]:opacity-100"
		/>
		<Dialog.Content
			class="fixed top-1/2 left-1/2 z-50 w-[calc(100%-2rem)] max-w-md -translate-x-1/2 -translate-y-1/2 rounded-2xl border border-base-300 bg-base-100 p-6 shadow-xl"
		>
			<Dialog.Title class="font-serif text-xl font-semibold tracking-tight">{title}</Dialog.Title>
			{#if description}
				<Dialog.Description class="mt-1 text-sm text-base-content/70"
					>{description}</Dialog.Description
				>
			{/if}
			{#if children}
				<div class="mt-5 flex flex-col gap-3">
					{@render children()}
				</div>
			{/if}
			{#if footer}
				<div class="mt-6 flex justify-end gap-2">
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
