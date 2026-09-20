<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import * as api from '$lib/api';
</script>

<svelte:head>
	<title>Not found · HushJournal</title>
</svelte:head>

<div class="flex min-h-dvh flex-col items-center justify-center px-6 text-center">
	<p class="font-serif text-3xl">{page.status}</p>
	<p class="mt-2 text-base-content/70">{page.error?.message ?? 'Something went missing.'}</p>
	<button type="button" class="btn mt-6 rounded-full btn-neutral" onclick={() => goto(api.login())}>
		Back home
	</button>
</div>
