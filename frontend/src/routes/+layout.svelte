<script lang="ts">
	import { onMount } from 'svelte';
	import favicon from '$lib/assets/favicon.svg';
	import { session } from '$lib/session.svelte';
	import './layout.css';

	let { children } = $props();

	onMount(() => {
		void session.hydrate();
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>HushJournal</title>
</svelte:head>

<div class="min-h-dvh bg-base-100 font-sans text-base-content">
	{#if !session.ready}
		<div class="flex min-h-dvh items-center justify-center text-sm text-base-content/50">
			Loading…
		</div>
	{:else}
		{@render children()}
	{/if}
</div>
