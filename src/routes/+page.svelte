<script lang="ts">
	import { open } from '@tauri-apps/api/dialog';
	import { onDestroy, onMount } from 'svelte';
	import { extractFileNameFromPath } from '$lib/parse';
	import { init } from './init';
	import { invoke } from '@tauri-apps/api/tauri';
	import { backups, clientConfig, clientDefaults, serverConfig } from '$lib/store';
	import { BaseDirectory, writeTextFile } from '@tauri-apps/api/fs';
	import { BACKUPS_FILE_NAME } from '$lib/app_files';
	import { sleep } from '$lib/concurrency';
	import { TauriEvent, emit, listen } from '@tauri-apps/api/event';
	import { onUpdaterEvent } from '@tauri-apps/api/updater';
	import { info, error as logError } from 'tauri-plugin-log-api';
	import { randomString } from '$lib/generate';
	import ArrowIcon from '~icons/ion/arrow-forward';
	import CaretDownIcon from '~icons/ion/caret-down';
	import Reload from '~icons/ion/reload';
	import TrashIcon from '~icons/ion/trash';
	import AddIcon from '~icons/ion/add';
	import UploadIcon from '~icons/ion/cloud-upload-outline';
	import Button from '$lib/ui/button.svelte';
	import Select from '$lib/ui/select.svelte';
	import Modal from '$lib/ui/modal.svelte';
	import JobStatusPopup from '$lib/ui/job_status/status.svelte';

	import type { Folder } from '../../src-tauri/bindings/Folder';
	import type { Backup } from '../../src-tauri/bindings/Backup';
	import type { Config } from '../../src-tauri/bindings/Config';
	import type { JobStatus } from '../../src-tauri/bindings/JobStatus';

	let server_home_folders: Folder[] = [];
	let incomingJob: App.BackupFolderJob | undefined;
	let jobs: App.Job[] = [];
	let completedJobs: App.Job[] = [];
	let failedJobs: App.Job[] = [];
	let target_server_folder: string | undefined;
	let button_states: { [key: string]: ButtonState } = {};
	let error: App.Error | undefined;
	let initError: App.Error | undefined;
	let use_client_directory = false;
	let selectMenuOpen = false;
	let outsideClickListener: (event: MouseEvent) => void;
	let hovering = false;
	let selectServerFolderModalOpen = false;

	$: selectItems = server_home_folders.map((folder) => ({
		title: folder.name,
		value: folder.name
	}));

	$: selectServerFolderModalOpen = incomingJob !== undefined;

	$: $backups.length > 0 &&
		writeTextFile(BACKUPS_FILE_NAME, JSON.stringify($backups), {
			dir: BaseDirectory.AppData
		}).catch((e) => {
			console.error(e);
			error = { message: 'Failed to write backups file' };
		});

	$: if ($backups.length > 0 && $serverConfig?.allow_background_backup) {
		$backups.map((backup) =>
			invoke('backup_on_change', { backup }).catch((e) => {
				console.error(e);
				error = {
					message: `Failed to start background update for ${backup.client_location.entity_name}`
				};
			})
		);
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

	const unlistenUpdater = onUpdaterEvent(async ({ error: updaterErrorMessage, status }) => {
		switch (status) {
			case 'ERROR':
				updaterErrorMessage && logError(updaterErrorMessage);
				error = {
					message: `Failed to update app\n${updaterErrorMessage}`
				};
				break;
			case 'PENDING':
				info('Checking for updates');
				break;
			case 'DONE':
				info('App updated to latest version');
				break;
			case 'UPTODATE':
				info('App is already up to date');
				break;
		}
	});

	const triggerServerFolderModalAndAwaitSelection = async () => {
		selectServerFolderModalOpen = true;
		while (selectServerFolderModalOpen) {
			await sleep(100);
		}
	};

	const unlistenFileDropEvent = listen<TauriEvent>(TauriEvent.WINDOW_FILE_DROP, async (event) => {
		hovering = false;
		const files: string[] = (event as any).payload;
		if (files.length === 0) return;

		await triggerServerFolderModalAndAwaitSelection();

		if (!target_server_folder) return;

		const target = target_server_folder;

		files.map(async (file) => {
			const isDirectory = await invoke<boolean>('is_directory', { path: file });
			let job: App.BackupFolderJob = {
				__type: isDirectory ? 'folder' : 'file',
				__frequency: 'one-time',
				id: randomString(16),
				state: 'loading',
				from: {
					name: extractFileNameFromPath(file),
					path: file,
					size: null
				}
			};

			try {
				addNewJob(job, target);
			} catch (e: any) {
				job.state = 'error';
				failedJobs = [...failedJobs, job];
				error = { message: e };
			}
		});
	});

	const unlistenFileHoverEvent = listen<TauriEvent>(TauriEvent.WINDOW_FILE_DROP_HOVER, (_) => {
		hovering = true;
	});

	const unlistenFileCancelEvent = listen(TauriEvent.WINDOW_FILE_DROP_CANCELLED, (_) => {
		hovering = false;
	});

	const backupEntity = async (backup: Backup): Promise<boolean> => {
		const buttonStateKey = `${backup.client_location.entity_name}_${backup.server_location.entity_name}`;
		button_states[buttonStateKey] = 'loading';
		try {
			let jobId = await invoke<string>('backup_entity', { backup });
			let status = await invoke<JobStatus>('check_job_status', { id: jobId });

			while (status === 'Running') {
				await sleep(3000);
				status = await invoke<JobStatus>('check_job_status', { id: jobId });
			}

			if (status === 'Completed') {
				button_states[buttonStateKey] = 'success';
				return true;
			} else {
				button_states[buttonStateKey] = 'error';
				return false;
			}
		} catch (e) {
			console.error(e);
			button_states[buttonStateKey] = 'error';
			return false;
		}
	};

	const addNewJob = async (incoming?: App.BackupFolderJob, target?: string) => {
		if (!incoming?.from?.path) {
			selectServerFolderModalOpen = false;
			error = {
				message: `Cannot add backup with the provided parameter: Local folder - ${incoming?.from?.path}`
			};
			return;
		}

		const server_folder = server_home_folders.find((folder) => folder.name === target);

		if (!server_folder) {
			selectServerFolderModalOpen = false;
			error = { message: `Server folder not found: ${target}` };
			return;
		}

		incoming.to = server_folder;

		if (
			jobs.some(
				(job: App.BackupFolderJob) =>
					job.from?.path === incoming?.from?.path && job.to?.path === incoming?.to?.path
			) ||
			$backups.some(
				(b) =>
					b.client_location.path === incoming?.from?.path &&
					b.server_location.path === server_folder.path
			)
		) {
			error = {
				message: `Backup already exists: ${incoming.from.path}`
			};
			incomingJob = undefined;
			target_server_folder = undefined;
			use_client_directory = false;
			return;
		}

		const backup: Backup = {
			client_location: {
				entity_name: incoming.from.name,
				path: incoming.from.path
			},
			server_location: { entity_name: server_folder.name, path: server_folder.path },
			latest_run: null,
			options: {
				use_client_directory
			}
		};

		const task = async (): Promise<void> => {
			const job = incoming as App.BackupFolderJob;
			if (!(await backupEntity(backup))) {
				failedJobs = [...failedJobs, job];
				error = {
					message: `Failed to backup ${backup.client_location.path}`
				};
			} else {
				completedJobs = [...completedJobs, job];
			}

			jobs = jobs.filter((j) => j.id !== job.id);
		};

		incoming.task = task();
		jobs = [...jobs, incoming];

		if (incoming.__frequency == 'recurring') {
			backups.update((currentState) => [...currentState, backup]);
			emit('backups-updated', $backups);
		}

		incomingJob = undefined;
		target_server_folder = undefined;
		use_client_directory = false;
	};

	const deleteBackup = async (backup: Backup) => {
		// HACK: Must type confirm as any because typescript doesn't type it as a promise
		const answer: Promise<boolean> = await (confirm as any)(
			`Are you sure you want to stop backing up ${backup.client_location.entity_name}?\n\nYour data will still exists on the server, that has to be deleted seperately.`
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

	const selectNewFolderToBackup = async (frequency: BackupFrequency) => {
		const local_entity_path = await open({
			multiple: false,
			title: 'Select a file',
		});

		if (!local_entity_path || Array.isArray(local_entity_path)) return;

		incomingJob = {
			__type: await invoke('is_directory', { path: local_entity_path }) ? 'folder' : 'file',
			__frequency: frequency,
			id: randomString(16),
			state: 'loading',
			from: {
				name: extractFileNameFromPath(local_entity_path),
				path: local_entity_path,
				size: null
			}
		};
	};

	const loadConfig = async () => {
		init()
			.then((data) => {
				server_home_folders = data || [];
			})
			.catch(() => {
				initError = { message: 'Failed to load config' };
			});
	};

	const toggleSelectMenuOpen = () => {
		selectMenuOpen = !selectMenuOpen;
		if (selectMenuOpen) {
			window.addEventListener('click', outsideClickListener);
		} else {
			window.removeEventListener('click', outsideClickListener);
		}
	};

	onMount(() => {
		outsideClickListener = (event: MouseEvent) => {
			const target = event.target as HTMLElement;
			if (!target.closest('.select') || target.closest('.menu')) {
				selectMenuOpen = false;
				window.removeEventListener('click', outsideClickListener);
			}
		};
		loadConfig();
	});
	onDestroy(async () => {
		(await unlistenReset)();
		(await unlistenRefreshServerConfig)();
		(await unlistenUpdater)();
		(await unlistenFileDropEvent)();
		(await unlistenFileHoverEvent)();
		(await unlistenFileCancelEvent)();
		outsideClickListener = () => {};
	});
</script>

{#if !initError && $serverConfig?.server_address}
	<div class={`file_drop_zone ${$clientConfig.theme}`} class:hovering>
		<div class="feedback">
			<span>
				<UploadIcon style="font-size: 6rem" />
				<p>Drop folder here to backup</p>
			</span>
		</div>
	</div>
	<div class={$clientConfig.theme}>
		<Modal
			bind:open={selectServerFolderModalOpen}
			onClickOutside={() => (target_server_folder = undefined)}
		>
			<div class="modal">
				<div class="form_group">
					<label for="server_home_folders">Select target folder on the server</label>
					<Select items={selectItems} bind:value={target_server_folder} />
				</div>
				<div class="form_group">
					<label for="use_client_directory">Use client directory on server</label>
					<input
						id="use_client_directory"
						type="checkbox"
						checked={use_client_directory}
						on:change={() => (use_client_directory = !use_client_directory)}
					/>
				</div>
				<Button
					type="secondary"
					onClick={() =>
						!incomingJob
							? (selectServerFolderModalOpen = false)
							: addNewJob(incomingJob, target_server_folder)}
				>
					Backup
				</Button>
			</div>
		</Modal>
		<div class="heading">
			<h1>Your backups</h1>
			<div class="select">
				<Button type="primary" onClick={toggleSelectMenuOpen} style="min-width: 8rem;">
					<div class="button-text">New</div>
					<AddIcon slot="icon" />
					<CaretDownIcon
						slot="secondary-icon"
						style={`transform: rotate(${selectMenuOpen ? 180 : 0}deg);`}
					/>
				</Button>
				{#if selectMenuOpen}
					<div class="menu">
						<button on:click={() => selectNewFolderToBackup('recurring')}>
							<Reload />Reacurring
						</button>
						<button on:click={() => selectNewFolderToBackup('one-time')}
							><ArrowIcon />One time</button
						>
					</div>
				{/if}
			</div>
		</div>
		{#if error}
			<p class="error">{@html error.message.replace(/\n/g, '<br>')}</p>
		{/if}
		{#if $backups.length > 0}
			<div class="backups">
				<div class="grid-heading">
					<div>
						<div>Local folder</div>
					</div>
					<div>
						<div>Server folder</div>
					</div>
				</div>
				{#each $backups as backup}
					{@const backupKey = `${backup.client_location.entity_name}_${backup.server_location.entity_name}`}
					<div class="backup grid">
						<div class="folder">
							<div>
								<Button type="icon" onClick={() => deleteBackup(backup)} style="padding-left: 0">
									<TrashIcon color="var(--clr-danger)" />
								</Button>
								<span>
									{backup.client_location.entity_name}
								</span>
							</div>
						</div>
						<!-- TODO: add a "backup up to date" state -->
						<Button
							type="icon-with_background"
							onClick={() => backupEntity(backup)}
							style="align-self: center;"
							state={button_states[backupKey] || 'idle'}
						>
							<ArrowIcon slot="icon" color="white" />
						</Button>
						<div class="folder">
							<div>{backup.server_location.entity_name}</div>
						</div>
					</div>
				{/each}
			</div>
		{:else}
			<div>No backups</div>
		{/if}
		<JobStatusPopup
			{jobs}
			{completedJobs}
			{failedJobs}
			onClear={() => {
				jobs = [];
				completedJobs = [];
				failedJobs = [];
			}}
		/>
	</div>
{:else if initError}
	<h1>Internal server error</h1>
	<p>{initError.message}</p>
{/if}

<style lang="scss">
	.heading {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.file_drop_zone {
		position: absolute;
		top: 0;
		left: 0;
		width: 100vw;
		height: 100vh;
		backdrop-filter: blur(10px);
		background-color: rgba(130, 130, 130, 0.5);
		z-index: 10;
		display: none;
		justify-content: center;
		align-items: center;

		&.hovering {
			display: flex;
		}

		.feedback,
		.feedback span {
			display: flex;
			flex-direction: column;
			align-items: center;
			justify-content: center;
		}

		.feedback {
			@include box;
			background: $clr-background;
			padding: 1rem;
			width: 75vw;
			height: 30vh;
			gap: 0.5rem;

			span {
				border: 1px dashed $slate-400;
				width: 100%;
				height: 100%;
				border-radius: 0.5rem;
			}
		}
	}

	.select {
		position: relative;

		.button-text {
			margin-left: 0.5rem;
		}

		.menu {
			@include box;
			position: absolute;
			bottom: -5.5rem;
			left: 0;
			display: flex;
			flex-direction: column;
			gap: 0.125rem;
			background-color: $clr-foreground;
			padding: 0.5rem;
			z-index: 0;

			button {
				border: none;
				background-color: transparent;
				text-align: left;
				display: flex;
				align-items: center;
				gap: 0.5rem;
				padding: 0.225rem;
				cursor: pointer;
				&:hover {
					background-color: $slate-300;
				}
			}
		}
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
		display: grid;
		grid-template-columns: 0.9fr 1fr;
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
		.feedback {
			background: $clr-background_dark;
			span {
				border: 1px dashed $slate-500;
			}
		}
		.backup {
			border-color: $clr-border_dark;
		}
		.select {
			.menu {
				background-color: $clr-secondary-action_dark;

				button {
					color: $clr-text_light;
					&:hover {
						background-color: $clr-secondary-action_hover_dark;
					}
				}
			}
		}
	}
</style>
