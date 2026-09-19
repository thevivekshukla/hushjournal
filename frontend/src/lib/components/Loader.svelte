<script lang="ts">
	let { label = 'Loading', size = 'md' }: { label?: string; size?: 'sm' | 'md' } = $props();
</script>

<progress
	class={[
		'loading appearance-none border-0 loading-spinner text-base-content/40',
		size === 'sm' ? 'loading-sm' : 'loading-md'
	]}
	aria-label={label}
></progress>

<style>
	progress::-webkit-progress-bar,
	progress::-webkit-progress-value,
	progress::-moz-progress-bar {
		background: none;
		display: none;
	}
</style>
