import { writable } from 'svelte/store';
import type { Config } from '../../src-tauri/bindings/Config';

export const config = writable<Config | undefined>(undefined);
