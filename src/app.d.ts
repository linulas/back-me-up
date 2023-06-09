// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {

	type ButtonType = 'primary' | 'secondary' | 'icon' | 'icon-with_background';
	type ButtonState = 'idle' | 'loading' | 'success' | 'error';

	namespace App {
    type Theme = 'light' | 'dark';
    interface Config {
      theme: Theme;
    }
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface Platform {}
	}
	// HACK: types not working for unplugin-icons
	declare module '~icons/*' {
		const content: any;
		export default content;
	}
}

export {};
