import { writable } from 'svelte/store';
import type { Config } from '../../src-tauri/bindings/Config';
import type { Backup } from '../../src-tauri/bindings/Backup';

export const config = writable<Config | undefined>(undefined);
export const backups = writable<Backup[]>([]);
