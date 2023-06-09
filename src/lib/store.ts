import { writable } from 'svelte/store';
import type { Config } from '../../src-tauri/bindings/Config';
import type { Backup } from '../../src-tauri/bindings/Backup';

export const serverConfig = writable<Config | undefined>(undefined);
export const clientConfig = writable<App.Config>({ theme: 'light' });
export const backups = writable<Backup[]>([]);
