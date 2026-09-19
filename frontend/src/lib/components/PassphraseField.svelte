<script lang="ts">
	let {
		value = $bindable(''),
		id,
		name,
		label,
		autocomplete,
		describedby,
		onkeydown,
		autofocus = false
	}: {
		value?: string;
		id: string;
		name: string;
		label: string;
		autocomplete: 'current-password' | 'new-password';
		describedby?: string;
		onkeydown?: (event: KeyboardEvent) => void;
		autofocus?: boolean;
	} = $props();

	let visible = $state(false);
</script>

<div class="w-full">
	<div class="mb-1 flex items-center justify-between">
		<label class="text-sm" for={id}>{label}</label>
		<button
			type="button"
			class="text-xs text-base-content/50"
			aria-pressed={visible}
			aria-controls={id}
			onclick={() => (visible = !visible)}
		>
			{visible ? 'Hide' : 'Show'}
		</button>
	</div>
	<!-- svelte-ignore a11y_autofocus -->
	<input
		{id}
		{name}
		class="input w-full"
		type={visible ? 'text' : 'password'}
		{autocomplete}
		aria-describedby={describedby}
		autocapitalize="none"
		spellcheck={false}
		bind:value
		{onkeydown}
		{autofocus}
	/>
</div>
