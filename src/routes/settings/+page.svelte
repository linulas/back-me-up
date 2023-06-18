<script lang="ts">
	import { BACKUPS_FILE_NAME, SERVER_CONFIG_FILE_NAME } from '$lib/app_files';
	import Button from '$lib/button.svelte';
	import { serverConfig } from '$lib/store';
	import { BaseDirectory, readTextFile, removeFile, writeTextFile } from '@tauri-apps/api/fs';
	import { invoke } from '@tauri-apps/api/tauri';
	import { WebviewWindow, appWindow } from '@tauri-apps/api/window';
	import { onMount } from 'svelte';
	import type { Config } from '../../../src-tauri/bindings/Config';
	import { emit } from '@tauri-apps/api/event';

	// let allowBackgroundBackups = true;
	let error: App.Error | undefined = undefined;
	let resetSucceded = false;

	$: if (resetSucceded) {
		appWindow.close();
	}

	$: console.log($serverConfig);

	// const toggleBackgroundBackups = () => {
	// 	allowBackgroundBackups = !allowBackgroundBackups;
	// };

	const reset = async () => {
		// HACK: Must type confirm as any because typescript doesn't type it as a promise
		const answer: Promise<boolean> = await (confirm as any)(
			`Are you sure you?\n\nYou have to begin a new setup.\n\nYour backups will still remain on the server, that has to be deleted seperately.`
		);
		if (!answer) return;
		try {
			await invoke('reset');
		} catch (e) {
			console.error(e);
			error = { message: "Couldn't reset config state on server" };
			return;
		}

		try {
			await writeTextFile(BACKUPS_FILE_NAME, JSON.stringify([]), {
				dir: BaseDirectory.AppData
			});
			await removeFile(SERVER_CONFIG_FILE_NAME, {
				dir: BaseDirectory.AppConfig
			});
		} catch (e) {
			console.error(e);
			error = { message: "Couldn't reset config files" };
			return;
		}

		error = undefined;
		serverConfig.set(undefined);
		await emit('reload');
		const mainWindow = WebviewWindow.getByLabel('main');
		await mainWindow?.show();
		resetSucceded = true;
	};

	onMount(async () => {
		const stored_config: Config = JSON.parse(
			await readTextFile(SERVER_CONFIG_FILE_NAME, {
				dir: BaseDirectory.AppConfig
			})
		);

		serverConfig.set(stored_config);
	});
</script>

<div class="settings">
	<div class="heading">
		<h1>Settings</h1>
		<Button type="danger" onClick={reset}>Disconnect</Button>
	</div>

	{#if error}
		<div class="error">{error.message}</div>
	{/if}

	<!-- <div class="options"> -->
	<!-- 	<div class="option checkbox"> -->
	<!-- 		<input -->
	<!-- 			id="allow-background-backups" -->
	<!-- 			type="checkbox" -->
	<!-- 			checked={allowBackgroundBackups} -->
	<!-- 			on:change={toggleBackgroundBackups} -->
	<!-- 		/> -->
	<!-- 		<label for="allow-background-backups">Allow background backups</label> -->
	<!-- 	</div> -->
	<!-- </div> -->
</div>

<style lang="scss">
	.heading {
		display: flex;
		justify-content: space-between;
	}

	.error {
		color: $clr-danger;
	}
</style>
