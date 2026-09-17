import { redirect } from '@sveltejs/kit';
import * as api from '$lib/api';

export const load = () => {
	redirect(307, api.login());
};
