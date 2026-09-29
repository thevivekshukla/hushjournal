import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import * as api from '$lib/api';
import { session } from '$lib/session.svelte';

export const load: PageLoad = async ({ fetch }) => {
	await session.hydrate(fetch);
	if (session.user) redirect(307, api.journals());
};
