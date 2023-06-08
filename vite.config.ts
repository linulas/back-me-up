import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import Icons from 'unplugin-icons/vite';

export default defineConfig({
	plugins: [
		sveltekit(),
		Icons({
			compiler: 'svelte'
		})
	],

	css: {
		preprocessorOptions: {
			scss: {
				additionalData: `@import "$lib/style/theme.scss";`
			}
		}
	}
});
