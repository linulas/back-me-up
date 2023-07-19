<script lang="ts">
	import { BACKUPS_FILE_NAME, SERVER_CONFIG_FILE_NAME } from '$lib/app_files';
	import Button from '$lib/button.svelte';
	import { backups, clientConfig, serverConfig } from '$lib/store';
	import { BaseDirectory, removeFile, writeTextFile } from '@tauri-apps/api/fs';
	import { invoke } from '@tauri-apps/api/tauri';
	import { WebviewWindow, appWindow } from '@tauri-apps/api/window';
	import { onDestroy, onMount } from 'svelte';
	import { emit, listen } from '@tauri-apps/api/event';
	import { loadStoredBackupsAndSetToState, loadStoredConfigAndSetToState } from '../init';
	import { onUpdaterEvent } from '@tauri-apps/api/updater';
	import { info, error as logError } from 'tauri-plugin-log-api';
	import { checkForUpdate } from '$lib/update';
	import { sleep } from '$lib/concurrency';

	import type { Config } from '../../../src-tauri/bindings/Config';
	import type { Backup } from '../../../src-tauri/bindings/Backup';

	let error: App.Error | undefined = undefined;
	let disconnected = false;
	let loading = false;
	let updateStatus: ButtonState = 'idle';

	$: if (disconnected) {
		appWindow.close();
	}

	$: if ($serverConfig) {
		emit('server-config-updated', $serverConfig).catch((e) => console.error(e));
	}

	$: if (updateStatus === 'loading') {
		// NOTE: if user aborts the update, the promise will never resolve, this ensures that the loading spinner stops
		sleep(10000)
			.then(() => (updateStatus = 'idle'))
			.catch();
	}

	const unlistenBackupsUpdate = listen<Backup[]>('backups-updated', ({ payload }) => {
		payload && backups.update(() => payload);
	});

	const unlistenUpdater = onUpdaterEvent(async ({ error: updaterErrorMessage, status }) => {
		switch (status) {
			case 'ERROR':
				updateStatus = 'error';
				updaterErrorMessage && logError(updaterErrorMessage);
				error = {
					message: `Failed to update app\n${updaterErrorMessage}`
				};
				break;
			case 'PENDING':
				updateStatus = 'loading';
				info('Checking for updates');
				break;
			case 'DONE':
				updateStatus = 'success';
				info('App updated to latest version');
				break;
			case 'UPTODATE':
				updateStatus = 'idle';
				info('App is already up to date');
				break;
		}
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
			await loadStoredConfigAndSetToState();
			await loadStoredBackupsAndSetToState();
		} catch (e) {
			console.error(e);
			error = { message: "Couldn't read config files" };
		}
	});

	onDestroy(async () => {
		(await unlistenBackupsUpdate)();
		(await unlistenUpdater)();
	});
</script>

<div class={`settings ${$clientConfig.theme}`}>
	<div class="heading">
		<h1>Settings</h1>
		<Button type="danger" onClick={reset}>Disconnect</Button>
	</div>

	{#if error}
		<div class="error">{@html error.message.replace(/\n/g, '<br>')}</div>
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

	<div class="update">
		<Button
			type="icon"
			onClick={() => {
				updateStatus = 'loading';
				error = undefined;
				checkForUpdate();
			}}
			state={updateStatus}
			loadingColor={$clientConfig.theme === 'dark' ? 'white' : 'black'}
		>
			<span class="button_text">Check for updates</span>
			<span slot="icon" />
		</Button>
	</div>
</div>

<style lang="scss">
	.settings {
		height: 100vh;
	}

	.heading {
		display: flex;
		justify-content: space-between;
	}

	.error {
		color: $clr-danger;
	}

	.update {
		position: fixed;
		bottom: 0;
		left: 1rem;
		.button_text {
			padding: 0.5rem;
			@include text-xs;
			margin-right: 0.125rem;
			text-decoration: underline;
		}
	}

	.dark {
		.button_text {
			color: $clr-text_light;
		}
	}
</style>
