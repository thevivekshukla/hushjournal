<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import './layout.css';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>e2ejournal</title>
</svelte:head>

<div class="min-h-dvh bg-base-100 font-sans text-base-content">
	{@render children()}
</div>
