<script lang="ts">
	import { open } from '@tauri-apps/api/dialog';
	import { onDestroy, onMount } from 'svelte';
	import { extractFileNameFromPath } from '$lib/parse';
	import { init, isRedirect } from './init';
	import { goto } from '$app/navigation';
	import { invoke } from '@tauri-apps/api/tauri';
	import { backups, clientConfig, clientDefaults, serverConfig } from '$lib/store';
	import { BaseDirectory, writeTextFile } from '@tauri-apps/api/fs';
	import { BACKUPS_FILE_NAME } from '$lib/app_files';
	import ArrowIcon from '~icons/ion/arrow-forward';
	import TrashIcon from '~icons/ion/trash';
	import AddIcon from '~icons/ion/add';
	import Button from '$lib/button.svelte';
	import Select from '$lib/select.svelte';
	import Modal from '$lib/modal.svelte';
	import { emit, listen } from '@tauri-apps/api/event';

	import type { Folder } from '../../src-tauri/bindings/Folder';
	import type { Backup } from '../../src-tauri/bindings/Backup';
	import type { Config } from '../../src-tauri/bindings/Config';

	let server_home_folders: Folder[] = [];
	let new_folder_to_backup: Folder | undefined;
	let target_server_folder: string | undefined;
	let button_states: { [key: string]: ButtonState } = {};
	let error: App.Error | undefined;

	$: selectItems = server_home_folders.map((folder) => ({
		title: folder.name,
		value: folder.name
	}));

	$: $backups.length > 0 &&
		writeTextFile(BACKUPS_FILE_NAME, JSON.stringify($backups), {
			dir: BaseDirectory.AppData
		}).catch((e) => {
			console.error(e);
			error = { message: 'Failed to write backups file' };
		});

	$: if ($backups.length > 0 && $serverConfig?.allow_background_backup) {
		$backups.map((backup) => invoke('backup_on_change', { backup }).catch((e) => console.error(e)));
	}

	$: Object.keys(button_states).map((key) => {
		if (button_states[key] === 'success') {
			setTimeout(() => {
				button_states[key] = 'idle';
			}, 5000);
		}
	});

	const unlistenReset = listen<string>('reset', () => {
		backups.set([]);
		clientConfig.set(clientDefaults);
		serverConfig.set(undefined);
		loadConfig();
	});

	const unlistenRefreshServerConfig = listen<Config>('server-config-updated', ({ payload }) => {
		payload && serverConfig.update(() => payload);
	});

	const backupDirectory = async (backup: Backup) => {
		const buttonStateKey = `${backup.client_folder.name}_${backup.server_folder.name}`;
		button_states[buttonStateKey] = 'loading';
		try {
			await invoke('backup_directory', { backup });
			button_states[buttonStateKey] = 'success';
			return true;
		} catch (e) {
			console.error(e);
			button_states[buttonStateKey] = 'error';
			return false;
		}
	};

	const addNewBackup = async () => {
		const server_folder = server_home_folders.find(
			(folder) => folder.name === target_server_folder
		);

		if (!server_folder) {
			error = {
				message: 'Server folder not found'
			};
			return;
		}

		const client_folder_copy = JSON.stringify(new_folder_to_backup);
		const server_folder_copy = JSON.stringify(target_server_folder);

		const backup: Backup = {
			client_folder: new_folder_to_backup!,
			server_folder,
			latest_run: null
		};

		new_folder_to_backup = undefined;
		target_server_folder = undefined;

		backups.update((currentState) => [...currentState, backup]);
		emit('backups-updated', $backups);

		if (!(await backupDirectory(backup))) {
			new_folder_to_backup = JSON.parse(client_folder_copy);
			target_server_folder = JSON.parse(server_folder_copy);
			return;
		}
	};

	const deleteBackup = async (backup: Backup) => {
		// HACK: Must type confirm as any because typescript doesn't type it as a promise
		const answer: Promise<boolean> = await (confirm as any)(
			`Are you sure you want to stop backing up ${backup.client_folder.name}?\n\nYour data will still exists on the server, that has to be deleted seperately.`
		);
		if (!answer) return;

		try {
			$serverConfig?.allow_background_backup &&
				(await invoke('terminate_background_backup', { backup }));
			backups.update((currentState) => currentState.filter((b) => b !== backup));
			emit('backups-updated', $backups);

			// Reactive data write won't run if length is 0, so we have to run manually in that case
			if ($backups.length === 0) {
				writeTextFile(BACKUPS_FILE_NAME, JSON.stringify([]), { dir: BaseDirectory.AppData });
			}
		} catch (e) {
			console.error(e);
		}
	};

	const selectNewFolderToBackup = async () => {
		const local_folder_path = await open({
			multiple: false,
			title: 'Select a folder',
			directory: true
		});

		if (!local_folder_path || Array.isArray(local_folder_path)) return;

		new_folder_to_backup = {
			name: extractFileNameFromPath(local_folder_path),
			path: local_folder_path,
			size: null
		};
	};

	const loadConfig = async () => {
		init()
			.then((data) => {
				server_home_folders = data;
			})
			.catch((err) => {
				if (isRedirect(err)) goto(err.location);
			});
	};

	onMount(loadConfig);
	onDestroy(async () => {
		(await unlistenReset)();
		(await unlistenRefreshServerConfig)();
	});
</script>

<div class={$clientConfig.theme}>
	<Modal open={new_folder_to_backup !== undefined}>
		<div class="modal">
			<div class="form_group">
				<label for="server_home_folders">Select target folder on the server</label>
				<Select items={selectItems} bind:value={target_server_folder} />
			</div>
			<Button type="secondary" onClick={addNewBackup}>Backup</Button>
		</div>
	</Modal>
	<div class="heading">
		<h1>Your backups</h1>
		<div>
			<Button type="primary" onClick={selectNewFolderToBackup}>
				New <AddIcon slot="icon" />
			</Button>
		</div>
	</div>
	{#if error}
		<p class="error">{error.message}</p>
	{/if}
	{#if $backups.length > 0}
		<div class="backups">
			<div class="grid grid-heading">
				<div>
					<div>Local folder</div>
				</div>
				<div />
				<div>
					<div>Server folder</div>
				</div>
			</div>
			{#each $backups as backup}
				{@const backupKey = `${backup.client_folder.name}_${backup.server_folder.name}`}
				<div class="backup grid">
					<div class="folder">
						<div>
							<Button type="icon" onClick={() => deleteBackup(backup)} style="padding-left: 0">
								<TrashIcon color="#ef4444" />
							</Button>
							<span>
								{backup.client_folder.name}
							</span>
						</div>
					</div>
					<!-- TODO: add a "backup up to date" state -->
					<Button
						type="icon-with_background"
						onClick={() => backupDirectory(backup)}
						style="align-self: center;"
						state={button_states[backupKey] || 'idle'}
					>
						<ArrowIcon slot="icon" color="white" />
					</Button>
					<div class="folder">
						<div>{backup.server_folder.name}</div>
					</div>
				</div>
			{/each}
		</div>
	{:else}
		<div>No backups</div>
	{/if}
</div>

<style lang="scss">
	.heading {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.error {
		color: $clr-danger;
	}

	.modal {
		.form_group {
			margin-bottom: 1rem;
			display: flex;
			flex-direction: column;
		}
	}

	.grid-heading {
		font-weight: bold;
		font-size: 1.2rem;
		margin-bottom: 0.5rem;
	}

	.grid {
		display: grid;
		grid-template-columns: 1fr 0.1fr 1fr;
		grid-template-rows: 1fr;
		grid-column-gap: 0.5rem;
		grid-row-gap: 0;
	}

	.backup {
		margin-bottom: 0.5rem;
		border-bottom: 1px solid $clr-border;
	}

	.folder {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem 1rem 1rem 0;

		div {
			display: flex;
			align-items: center;
		}
	}

	h1 {
		@include text-2xl;
		margin: $m-2xl;
		@media screen and (min-width: $media-sm) {
			@include text-3xl;
			margin: $m-3xl;
		}
	}

	.dark {
		.backup {
			border-color: $clr-border_dark;
		}
	}
</style>
