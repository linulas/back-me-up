import type { Folder } from '../src-tauri/bindings/Folder';
import type { File } from '../src-tauri/bindings/File';
// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
	type ButtonType = 'primary' | 'secondary' | 'danger' | 'icon' | 'icon-with_background';
	type ButtonState = 'idle' | 'loading' | 'success' | 'error';
	type BackupJobType = 'file' | 'folder';
  type BackupFrequency = 'one-time' | 'recurring';

	namespace App {
		type Theme = 'light' | 'dark';
		interface Config {
			theme: Theme;
		}

		interface Job {
			__type: BackupJobType;
      __frequency: BackupFrequency;
      id: string;
			state: ButtonState;
      task?: Promise<void>;
		}

		interface BackupFolderJob extends Job {
			from?: Folder;
      to?: Folder;
		}

		interface BackupFileJob extends Job {
			from?: File;
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
