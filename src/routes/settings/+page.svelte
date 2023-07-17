<script lang="ts">
	import { BACKUPS_FILE_NAME, SERVER_CONFIG_FILE_NAME } from '$lib/app_files';
	import Button from '$lib/button.svelte';
	import { backups, serverConfig } from '$lib/store';
	import { BaseDirectory, removeFile, writeTextFile } from '@tauri-apps/api/fs';
	import { invoke } from '@tauri-apps/api/tauri';
	import { WebviewWindow, appWindow } from '@tauri-apps/api/window';
	import { onDestroy, onMount } from 'svelte';
	import { emit, listen } from '@tauri-apps/api/event';
	import { loadStoredBackupsAndSetToState, loadStoredConfigAndSetToState } from '../init';

	import type { Config } from '../../../src-tauri/bindings/Config';
	import type { Backup } from '../../../src-tauri/bindings/Backup';

	let error: App.Error | undefined = undefined;
	let disconnected = false;
	let loading = false;

	$: if (disconnected) {
		appWindow.close();
	}

	$: if ($serverConfig) {
		emit('server-config-updated', $serverConfig).catch((e) => console.error(e));
	}

	const unlistenBackupsUpdate = listen<Backup[]>('backups-updated', ({ payload }) => {
		payload && backups.update(() => payload);
	});

	const handleConfigUpdate = async (config: Config) => {
		loading = true;

		try {
			await invoke('set_config', { config });

			if (!config.allow_background_backup) {
				await invoke('terminate_all_background_jobs');
			}
		} catch (e) {
			console.error(e);
			error = { message: "Couldn't update server state with new config" };
		}

		try {
			await writeTextFile(SERVER_CONFIG_FILE_NAME, JSON.stringify(config), {
				dir: BaseDirectory.AppConfig
			});
		} catch (e) {
			console.error(e);
			error = { message: "Couldn't save server config" };
		}

		loading = false;
	};

	const toggleBackgroundBackups = async () => {
		serverConfig.update((state) => {
			if (!state) return state;
			state.allow_background_backup = !state?.allow_background_backup;
			handleConfigUpdate(state);
			return state;
		});
	};

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
		await emit('reset');
		const mainWindow = WebviewWindow.getByLabel('main');
		await mainWindow?.show();
		disconnected = true;
	};

	onMount(async () => {
		try {
			loadStoredConfigAndSetToState();
			loadStoredBackupsAndSetToState();
		} catch (e) {
			console.error(e);
			error = { message: "Couldn't read config files" };
		}
	});

	onDestroy(async () => {
		(await unlistenBackupsUpdate)();
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

	<div class="options">
		<div class="option checkbox">
			<input
				disabled={loading}
				id="allow-background-backups"
				type="checkbox"
				checked={$serverConfig?.allow_background_backup}
				on:change={toggleBackgroundBackups}
			/>
			<label for="allow-background-backups">Allow background backups</label>
		</div>
	</div>
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
