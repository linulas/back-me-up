import { error, redirect } from '@sveltejs/kit';
import { invoke } from '@tauri-apps/api/tauri';
import { exists, createDir, readTextFile, BaseDirectory, writeTextFile } from '@tauri-apps/api/fs';
import { appConfigDir } from '@tauri-apps/api/path';
import { backups, serverConfig } from '$lib/store';
import { BACKUPS_FILE_NAME, SERVER_CONFIG_FILE_NAME } from '$lib/app_files';

import type { Redirect } from '@sveltejs/kit';
import type { Folder } from '../../src-tauri/bindings/Folder';
import type { Config } from '../../src-tauri/bindings/Config';

export const isRedirect = (someError: any): someError is Redirect => someError?.status === 302;

const getServerHomeFolders = async () => {
	try {
		return await invoke<Folder[]>('list_home_folders');
	} catch (e) {
		console.error(e);
		const appError: App.Error = { message: "Couldn't get server home folders" };
		throw error(500, appError);
	}
};

const setStateOnServer = async (config: Config) => {
	try {
		await invoke('set_state', { config });
	} catch (e) {
		const appError: App.Error = {
			message: "Couldn't establish a server connection based on your config"
		};
		console.error(e);
		throw error(500, appError);
	}
};

const createConfigDirectory = async () => {
	try {
		const appConfigPath = await appConfigDir();
		await createDir(appConfigPath);
	} catch (e) {
		console.error(e);
		throw error(500, { message: "Couldn't create config directory" });
	}
};

const appConfigDirectoryExists = async () => {
	try {
		return await exists(await appConfigDir());
	} catch (e) {
		console.error(e);
		return false;
	}
};

const configFileExist = async () => {
	const options = { dir: BaseDirectory.AppConfig };
	try {
		return await exists(SERVER_CONFIG_FILE_NAME, options);
	} catch (e) {
		console.error(e);
		throw error(500, { message: 'Error checking if config file exists' });
	}
};

export const loadStoredBackupsAndSetToState = async () => {
	const options = { dir: BaseDirectory.AppData };
	try {
		if (!(await exists(BACKUPS_FILE_NAME, options))) {
			writeTextFile(BACKUPS_FILE_NAME, JSON.stringify([]), options);
		}

		backups.set(JSON.parse(await readTextFile(BACKUPS_FILE_NAME, options)));
	} catch (e) {
		console.error(e);
	}
};

export const loadStoredConfigAndSetToState = async () => {
	const options = { dir: BaseDirectory.AppConfig };
	const stored_config: Config = JSON.parse(await readTextFile(SERVER_CONFIG_FILE_NAME, options));
	serverConfig.set(stored_config);
	return stored_config;
};

export const init = async () => {
	try {
		if (!(await appConfigDirectoryExists())) {
			await createConfigDirectory();
		}

		if (!(await configFileExist())) {
			throw redirect(302, '/setup');
		}

		loadStoredBackupsAndSetToState();
		const config = await loadStoredConfigAndSetToState();
		await setStateOnServer(config);
		const server_home_folders = await getServerHomeFolders();

		return server_home_folders;
	} catch (e) {
		if (isRedirect(e)) throw e;
		console.error(e);
		throw error(500, { message: "Couldn't load config" });
	}
};
