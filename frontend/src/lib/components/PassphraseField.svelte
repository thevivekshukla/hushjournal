<script lang="ts">
	let {
		value = $bindable(''),
		id,
		name,
		label,
		autocomplete,
		describedby,
		onkeydown,
		autofocus = false,
		minlength
	}: {
		value?: string;
		id: string;
		name: string;
		label: string;
		autocomplete: 'current-password' | 'new-password';
		describedby?: string;
		onkeydown?: (event: KeyboardEvent) => void;
		autofocus?: boolean;
		minlength?: number;
	} = $props();

	let visible = $state(false);
</script>

<div class="grid w-full grid-cols-[1fr_auto] items-center gap-x-2">
	<label class="mb-1 text-sm" for={id}>{label}</label>
	<!-- svelte-ignore a11y_autofocus -->
	<input
		{id}
		{name}
		class="input col-span-2 w-full"
		type={visible ? 'text' : 'password'}
		{autocomplete}
		aria-describedby={describedby}
		autocapitalize="none"
		spellcheck={false}
		bind:value
		{onkeydown}
		{autofocus}
		{minlength}
	/>
	<button
		type="button"
		class="col-start-2 row-start-1 mb-1 text-xs text-base-content/50"
		aria-pressed={visible}
		aria-controls={id}
		onclick={() => (visible = !visible)}
	>
		{visible ? 'Hide' : 'Show'}
	</button>
</div>
