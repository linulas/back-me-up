import { error, redirect } from '@sveltejs/kit';
import { invoke } from '@tauri-apps/api/tauri';
import { exists, createDir, readTextFile, BaseDirectory } from '@tauri-apps/api/fs';
import { configDir, appConfigDir } from '@tauri-apps/api/path';
import { config } from '$lib/store';
import { CONFIG_FILE_NAME } from '$lib/app_files';

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
		console.log(`Creating app config directory ${appConfigPath}`);
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
		return await exists(CONFIG_FILE_NAME, options);
	} catch (e) {
		console.error(e);
		throw error(500, { message: 'Error checking if config file exists' });
	}
};

export const init = async () => {
	try {
		if (!(await appConfigDirectoryExists())) {
			await createConfigDirectory();
		}

		if (!(await configFileExist())) {
			console.log('No config found, redirecting to setup...');
			throw redirect(302, '/setup');
		}

		const options = { dir: BaseDirectory.AppConfig };
		const stored_config: Config = JSON.parse(await readTextFile(CONFIG_FILE_NAME, options));
		config.set(stored_config); // Client app state
		await setStateOnServer(stored_config);
		const server_home_folders = await getServerHomeFolders();

		return server_home_folders;
	} catch (e) {
		if (isRedirect(e)) throw e;
		console.error(e);
		throw error(500, { message: "Couldn't load config" });
	}
};
