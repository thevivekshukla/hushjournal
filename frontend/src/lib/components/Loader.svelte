<script lang="ts">
	let { label = 'Loading', size = 'md' }: { label?: string; size?: 'sm' | 'md' } = $props();
</script>

<span
	class={[
		'loading aspect-square overflow-visible loading-spinner leading-none text-base-content/40',
		size === 'sm' ? 'loading-sm' : 'loading-md'
	]}
	role="progressbar"
	aria-label={label}
></span>
