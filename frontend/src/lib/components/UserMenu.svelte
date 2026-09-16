<script lang="ts">
	import { goto } from '$app/navigation';
	import { DropdownMenu } from 'bits-ui';
	import { session } from '$lib/session.svelte';

	let { compact = false }: { compact?: boolean } = $props();

	const user = $derived(session.user);
	const initial = $derived(user?.name?.charAt(0).toUpperCase() ?? '?');

	function goWorkspaces() {
		session.lock();
		void goto('/workspaces');
	}

	function logout() {
		session.logout();
		void goto('/');
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
				class="flex size-8 items-center justify-center rounded-full bg-primary text-sm font-medium text-primary-content"
			>
				{initial}
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
					onSelect={logout}
				>
					<span class="icon-[lucide--log-out] size-4"></span>
					Sign out
				</DropdownMenu.Item>
			</DropdownMenu.Content>
		</DropdownMenu.Portal>
	</DropdownMenu.Root>
{/if}
