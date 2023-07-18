import { error, redirect } from '@sveltejs/kit';
import { invoke } from '@tauri-apps/api/tauri';
import { exists, createDir, readTextFile, BaseDirectory, writeTextFile } from '@tauri-apps/api/fs';
import { appConfigDir, appDataDir } from '@tauri-apps/api/path';
import { backups, serverConfig } from '$lib/store';
import { BACKUPS_FILE_NAME, SERVER_CONFIG_FILE_NAME } from '$lib/app_files';
import { info, error as logError } from 'tauri-plugin-log-api';

import type { Redirect } from '@sveltejs/kit';
import type { Folder } from '../../src-tauri/bindings/Folder';
import type { Config } from '../../src-tauri/bindings/Config';
import { goto } from '$app/navigation';

export const isRedirect = (someError: any): someError is Redirect => someError?.status === 302;

const getServerHomeFolders = async () => {
	try {
		return await invoke<Folder[]>('list_home_folders');
	} catch (e) {
		const appError: App.Error = { message: "Couldn't get server home folders" };
		logError(`${appError.message}: ${JSON.stringify(e)}`);
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
		logError(`${appError.message}: ${JSON.stringify(e)}`);
		throw error(500, appError);
	}
};

export const createConfigDirectory = async () => {
	try {
		const appConfigPath = await appConfigDir();
		await createDir(appConfigPath);
	} catch (e) {
		const message = `Couldn't create config directory`;
		logError(`${message}: ${JSON.stringify(e)}`);
		throw error(500, { message });
	}
};

export const appConfigDirectoryExists = async () => {
	try {
		return await exists(await appConfigDir());
	} catch (e) {
		logError(JSON.stringify(e));
		return false;
	}
};

const configFileExist = async () => {
	const options = { dir: BaseDirectory.AppConfig };
	try {
		return await exists(SERVER_CONFIG_FILE_NAME, options);
	} catch (e) {
		const message = `Error checking if config file exists`;
		logError(`${message}: ${JSON.stringify(e)}`);
		throw error(500, { message });
	}
};

const createDataDirectory = async () => {
	try {
		const appDataPath = await appDataDir();
		await createDir(appDataPath);
	} catch (e) {
		const message = `Couldn't create data directory`;
		logError(`${message}: ${JSON.stringify(e)}`);
		throw error(500, { message });
	}
};

const appDataDirectoryExists = async () => {
	try {
		return await exists(await appDataDir());
	} catch (e) {
		logError(JSON.stringify(e));
		return false;
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
		logError(JSON.stringify(e));
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

		if (!(await appDataDirectoryExists())) {
			await createDataDirectory();
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
		if (isRedirect(e)) {
			info('No config found, redirecting to setup');
			await goto(e.location);
		} else {
			const message = "Couldn't load config";
			logError(`${message}: ${JSON.stringify(e)}`);
			throw error(500, { message });
		}
	}
};
