<script lang="ts">
	import { open } from '@tauri-apps/api/dialog';
	import { onMount } from 'svelte';
	import { extractFileNameFromPath } from '$lib/parse';
	import Modal from '$lib/modal.svelte';
	import { init, isRedirect } from './init';
	import { goto } from '$app/navigation';
	import { invoke } from '@tauri-apps/api/tauri';
	import { backups } from '$lib/store';
	import { BaseDirectory, writeTextFile } from '@tauri-apps/api/fs';
	import { BACKUPS_FILE_NAME } from '$lib/app_files';

	import type { Folder } from '../../src-tauri/bindings/Folder';
	import type { Backup } from '../../src-tauri/bindings/Backup';

	let server_home_folders: Folder[] = [];
	let new_folder_to_backup: Folder | undefined;
	let target_server_folder: String | undefined;

	$: $backups.length > 0 &&
		writeTextFile(BACKUPS_FILE_NAME, JSON.stringify($backups), {
			dir: BaseDirectory.AppData
		});

	const backupDirectory = async (backup: Backup) => {
		try {
			await invoke('backup_directory', { backup });
			return true;
		} catch (e) {
			console.error(e);
			// TODO: handle error
			return false;
		}
	};

	const addNewBackup = async () => {
		const server_folder = server_home_folders.find(
			(folder) => folder.name === target_server_folder
		);
		if (!server_folder) return; // TODO: handle error
		const backup: Backup = {
			client_folder: new_folder_to_backup!,
			server_folder,
			latest_run: null
		};

		if (await backupDirectory(backup)) {
			backups.update((currentState) => [...currentState, backup]);
			new_folder_to_backup = undefined;
			target_server_folder = undefined;
		}
	};

	const deleteBackup = async (backup: Backup) => {
		// HACK: Must type confirm as any because typescript doesn't type it as a promise
		const answer: Promise<boolean> = await (confirm as any)(
			'Are you sure you want to delete this backup'
		);
		if (!answer) return;
		backups.update((currentState) => currentState.filter((b) => b !== backup));

		// Reactive data write won't run if length is 0, so we have to run manually in that case
		if ($backups.length === 0) {
			writeTextFile(BACKUPS_FILE_NAME, JSON.stringify([]), { dir: BaseDirectory.AppData });
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

	const refresh = () => {
		init()
			.then((data) => {
				server_home_folders = data;
			})
			.catch((err) => {
				console.log({ err });
				if (isRedirect(err)) goto(err.location);
			});
	};

	onMount(refresh);
</script>

<div>
	<Modal open={new_folder_to_backup !== undefined}>
		<label for="server_home_folders">Select target folder on the server</label>
		<select id="server_home_folders" bind:value={target_server_folder}>
			{#each server_home_folders as folder}
				<option value={folder.name}>{folder.name}</option>
			{/each}
		</select>
		<button on:click={addNewBackup}>Add</button>
	</Modal>
	<button on:click={refresh}>Refresh</button>
	<div class="heading">
		<h1>Your backups</h1>
		<div>
			<button on:click={selectNewFolderToBackup}> Add backup </button>
		</div>
	</div>
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
				<div class="backup grid">
					<div class="folder">
						<div>{backup.client_folder.name}</div>
					</div>
					<!-- TODO: add a "backup up to date" state -->
					<button class="arrow" on:click={() => backupDirectory(backup)}>></button>
					<div class="folder">
						<div>{backup.server_folder.name}</div>
						<button on:click={() => deleteBackup(backup)}>Del</button>
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
	}

	.folder {
		display: flex;
		justify-content: space-between;
		align-items: center;
		border-radius: 5px;
		padding: 1rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.26);
	}

	.arrow {
		display: flex;
		justify-content: center;
		align-items: center;
	}
</style>
