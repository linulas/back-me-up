// import { error } from '@sveltejs/kit';
// import { appWindow } from '@tauri-apps/api/window';
// import { clientConfig } from '$lib/store';
// import type { PageLoad } from './$types';
//
// export const load = (async () => {
// 	try {
// 		let theme = await appWindow.theme() || 'light';
//     clientConfig.set({theme});
// 		return { theme };
// 	} catch (e) {
// 		console.error(e);
// 		throw error(500, { message: "Couldn't load config" });
// 	}
// }) satisfies PageLoad;
