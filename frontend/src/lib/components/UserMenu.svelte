<script lang="ts">
	import { goto } from '$app/navigation';
	import { DropdownMenu } from 'bits-ui';
	import * as api from '$lib/api';
	import { journal } from '$lib/journal.svelte';
	import { session } from '$lib/session.svelte';

	let { compact = false }: { compact?: boolean } = $props();

	const user = $derived(session.user);
	const initial = $derived(user?.name?.charAt(0).toUpperCase() ?? '?');

	function goWorkspaces() {
		void journal.flush();
		session.lock();
		journal.clearWorkspace();
		void goto(api.workspaces());
	}

	async function logout() {
		await journal.flush();
		await session.logout();
		journal.clearWorkspace();
		void goto(api.login());
	}
</script>

{#if user}
	<DropdownMenu.Root>
		<DropdownMenu.Trigger
			class={['btn gap-2 btn-ghost px-2', compact ? 'btn-circle btn-sm' : 'h-10 rounded-full pr-3']}
			type="button"
			aria-label="Account menu"
		>
			<span
				class="relative flex size-8 shrink-0 overflow-hidden rounded-full bg-primary text-sm font-medium text-primary-content"
			>
				<span class="flex size-8 items-center justify-center">{initial}</span>
				{#if user.avatarUrl}
					<img
						src={user.avatarUrl}
						alt=""
						class="absolute inset-0 size-8 object-cover"
						referrerpolicy="no-referrer"
						onerror={(event) => event.currentTarget.remove()}
					/>
				{/if}
			</span>
			{#if !compact}
				<span class="hidden sm:inline">{user.name}</span>
			{/if}
		</DropdownMenu.Trigger>
		<DropdownMenu.Portal>
			<DropdownMenu.Content
				class="z-50 min-w-48 rounded-xl border border-base-300 bg-base-100 p-1 shadow-lg"
				sideOffset={8}
				align="end"
			>
				<div class="px-3 py-2">
					<p class="text-sm font-medium">{user.name}</p>
					<p class="text-xs text-base-content/60">{user.email}</p>
				</div>
				<DropdownMenu.Separator class="my-1 h-px bg-base-300" />
				<DropdownMenu.Item
					class="flex cursor-pointer items-center gap-2 rounded-lg px-3 py-2 text-sm data-highlighted:bg-base-200"
					onSelect={goWorkspaces}
				>
					<span class="icon-[lucide--layout-grid] size-4"></span>
					Workspaces
				</DropdownMenu.Item>
				<DropdownMenu.Item
					class="flex cursor-pointer items-center gap-2 rounded-lg px-3 py-2 text-sm text-error data-highlighted:bg-base-200"
					onSelect={() => void logout()}
				>
					<span class="icon-[lucide--log-out] size-4"></span>
					Sign out
				</DropdownMenu.Item>
			</DropdownMenu.Content>
		</DropdownMenu.Portal>
	</DropdownMenu.Root>
{/if}
