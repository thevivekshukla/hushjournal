<script lang="ts">
	const themes = { light: 'silk', dark: 'dim' } as const;

	let dark = $state(
		typeof document !== 'undefined' &&
			document.documentElement.getAttribute('data-theme') === themes.dark
	);

	function toggle() {
		dark = !dark;
		const theme = dark ? themes.dark : themes.light;
		document.documentElement.setAttribute('data-theme', theme);
		localStorage.setItem('theme', theme);
	}
</script>

<button
	type="button"
	class="btn btn-circle btn-ghost btn-sm"
	onclick={toggle}
	aria-label={dark ? 'Switch to light theme' : 'Switch to dark theme'}
>
	{#if dark}
		<span class="icon-[lucide--sun] size-4"></span>
	{:else}
		<span class="icon-[lucide--moon] size-4"></span>
	{/if}
</button>
