<script lang="ts">
	import { isUrl, isNumber } from '$lib/validate';
	import { BaseDirectory, writeTextFile } from '@tauri-apps/api/fs';
	import { goto } from '$app/navigation';
	import { config } from '$lib/store';

	import type { Config } from '../../../src-tauri/bindings/Config';

	let username = '';
	let server_address = '';
	let server_port = 22;

	$: adressIsValid = isUrl(server_address);
	$: portIsValid = isNumber(server_port) && server_port > 0 && server_port < 65536;
	$: usernameIsValid = username.length > 0;
	$: if ($config != undefined) goto('/');

	const submit = async () => {
		if (!adressIsValid || !portIsValid || !usernameIsValid) return; // TODO: handle error

		const newConfig: Config = {
			username,
			server_address,
			server_port,
		};

		await writeTextFile('app.conf', JSON.stringify(newConfig), { dir: BaseDirectory.AppConfig })
			.then(() => {
				config.set(newConfig);
			})
			.catch((e) => {
				console.error(e); // TODO: handle error
			});
	};
</script>

<div>
	<h1>Setup</h1>

	<a href="/">Go home</a>

	<div class="input_group">
		<label for="username">Username</label>
		<input id="username" type="text" bind:value={username} />
	</div>

	<div class="input_group">
		<label for="server_adress">Server adress</label>
		<input id="server_adress" type="text" bind:value={server_address} />
	</div>

	<div class="input_group">
		<label for="server_port">Server port</label>
		<input id="server_port" type="number" bind:value={server_port} />
	</div>

	<button on:click={submit}>Submit</button>
</div>

<style lang="scss">
	.input_group {
		input {
			width: 100%;
			padding: 0.5rem;
		}
		margin-bottom: 1rem;
	}
</style>
