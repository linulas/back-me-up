<script lang="ts">
	import { backups, clientConfig } from '$lib/store';
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow } from '@tauri-apps/api/window';
	import { onMount } from 'svelte';

	$: if ($backups.length > 0) {
		$backups.map((backup) => invoke('backup_on_change', { backup }));
	}

	onMount(() => {
		appWindow
			.theme()
			.then((theme) => {
				theme && clientConfig.set({ theme });
			})
			.catch((e) => {
				console.error(e);
			});

		appWindow.listen('tauri://theme-changed', ({ payload }: any) => {
			console.log({ payload });
			clientConfig.update(() => ({ theme: payload }));
		});
	});
</script>

<svelte:head>
	{#if $clientConfig.theme === 'dark'}
		<style lang="scss">
			body {
				background: $clr-background_dark;
				color: $clr-text_light;
			}
		</style>
	{:else}
		<style lang="scss">
			body {
				background: $clr-background;
				color: $clr-text_dark;
			}
		</style>
	{/if}
</svelte:head>

<main>
	<slot />
</main>

<style lang="scss">
	@import '../lib/style/mixins.scss';

	:global(body) {
		font-family: $font-sans-serif;
		@include text-sm;
		@media screen and (min-width: $media-sm) {
			@include text-base;
		}
	}
	:global(p, span, div, button, a, li, label) {
		@include text-sm;
		@media screen and (min-width: $media-sm) {
			@include text-base;
		}
	}

	main {
		width: calc(100% - 2rem);
		margin: auto;
	}
</style>
