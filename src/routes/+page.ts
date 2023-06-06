// import { error, redirect } from '@sveltejs/kit';
// import { invoke } from '@tauri-apps/api/tauri';
// import { exists, createDir, readTextFile, BaseDirectory } from '@tauri-apps/api/fs';
// import { configDir, appConfigDir } from '@tauri-apps/api/path';
// import { config } from '$lib/store';
//
// import type { Redirect } from '@sveltejs/kit';
// import type { PageLoad } from './$types';
// import type { Folder } from '../../src-tauri/bindings/Folder';
// import type { Config } from '../../src-tauri/bindings/Config';
//
// const CONFIG = `app.conf`;
//
// async function getServerHomeFolders() {
// 	try {
// 		return await invoke<Folder[]>('list_home_folders');
// 	} catch (e) {
// 		console.error(e);
// 		const appError: App.Error = { message: "Couldn't get server home folders" };
// 		throw error(500, appError);
// 	}
// }
//
// async function setStateOnServer(config: Config) {
// 	try {
// 		await invoke('set_state', { config });
// 	} catch (e) {
// 		const appError: App.Error = {
// 			message: "Couldn't establish a server connection based on your config"
// 		};
// 		console.error(e);
// 		throw error(500, appError);
// 	}
// }
//
// const createConfigDirectory = async () => {
// 	try {
// 		const configPath = await configDir();
// 		if (!(await exists(configPath))) {
// 			console.log('Creating root config directory...');
// 			await createDir(configPath);
// 		}
//
// 		const appConfigPath = await appConfigDir();
// 		if (!(await exists(appConfigPath))) {
// 			await createDir(appConfigPath);
// 		}
// 		console.log({ configPath, appConfigPath });
// 	} catch (e) {
// 		console.error(e);
// 		throw error(500, { message: "Couldn't create config directory" });
// 	}
// };
//
// const isRedirect = (someError: any): someError is Redirect => someError?.status === 302;
//
// const configExist = async () => {
// 	const options = { dir: BaseDirectory.AppConfig };
// 	try {
// 		return await exists(CONFIG, options);
// 	} catch (e) {
// 		console.error(e);
// 		// We probably got error because config directory doesn't exists
// 		// Create it and try again
// 		await createConfigDirectory();
// 		return await exists(CONFIG, options);
// 	}
// };
//
// export const load = (async () => {
// 	try {
// 		if (!(await configExist())) {
// 			console.log('No config found, redirecting to setup...');
// 			throw redirect(302, '/setup');
// 		}
//
// 		const options = { dir: BaseDirectory.AppConfig };
// 		const stored_config: Config = JSON.parse(await readTextFile('app.conf', options));
// 		config.set(stored_config);
// 		await setStateOnServer(stored_config);
// 		const server_home_folders = await getServerHomeFolders();
//
// 		return { server_home_folders };
// 	} catch (e) {
// 		if (isRedirect(e)) throw e;
// 		console.error(e);
// 		throw error(500, { message: "Couldn't load config" });
// 	}
// }) satisfies PageLoad;
