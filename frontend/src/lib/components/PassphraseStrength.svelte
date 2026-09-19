<script lang="ts">
	import { passphraseStrength, type PassphraseScore } from '$lib/passphrase';

	let { value, id }: { value: string; id: string } = $props();

	const strength = $derived(passphraseStrength(value));
	const bar = $derived(tone(strength.score, 'bar'));
	const text = $derived(tone(strength.score, 'text'));

	function tone(score: PassphraseScore, kind: 'bar' | 'text') {
		if (score <= 1) return kind === 'bar' ? 'bg-error' : 'text-error';
		if (score === 2) return kind === 'bar' ? 'bg-warning' : 'text-warning';
		if (score === 3) return kind === 'bar' ? 'bg-info' : 'text-info';
		return kind === 'bar' ? 'bg-success' : 'text-success';
	}
</script>

{#if value}
	<div class="mt-1.5">
		<div
			class="flex gap-1"
			role="meter"
			aria-labelledby={id}
			aria-valuemin={0}
			aria-valuemax={4}
			aria-valuenow={strength.score}
			aria-valuetext={strength.label}
		>
			{#each [1, 2, 3, 4] as step (step)}
				<span
					class={['h-1 min-w-0 flex-1 rounded-full', strength.score >= step ? bar : 'bg-base-300']}
				></span>
			{/each}
		</div>
		<p {id} class={['mt-1 text-xs', text]}>{strength.label}</p>
	</div>
{/if}
