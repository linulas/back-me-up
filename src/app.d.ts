import type { Folder } from '../src-tauri/bindings/Folder';
// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
	type ButtonType = 'primary' | 'secondary' | 'danger' | 'icon' | 'icon-with_background';
	type ButtonState = 'idle' | 'loading' | 'success' | 'error';
	type BackupFolderJobType = 'single' | 'reacurring';

	namespace App {
		type Theme = 'light' | 'dark';
		interface Config {
			theme: Theme;
		}

		interface Job {
			__type: BackupFolderJobType;
      id: string;
			state: ButtonState;
      task?: Promise<void>;
		}

		interface BackupFolderJob extends Job {
			from?: Folder;
      to?: Folder;
		}
	}
	// HACK: types not working for unplugin-icons
	declare module '~icons/*' {
		const content: any;
		export default content;
	}
}

export {};
