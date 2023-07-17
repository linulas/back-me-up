<script lang="ts">
	import { isUrl, isNumber, isIp } from '$lib/validate';
	import { BaseDirectory, writeTextFile } from '@tauri-apps/api/fs';
	import { goto } from '$app/navigation';
	import { serverConfig, clientConfig } from '$lib/store';
	import PlugIcon from '~icons/mdi/plug';
	import Button from '$lib/button.svelte';
	import { SERVER_CONFIG_FILE_NAME } from '$lib/app_files';
	import { invoke } from '@tauri-apps/api/tauri';
	import { onMount } from 'svelte';
	import { randomString } from '$lib/generate';
	import { error as logError } from 'tauri-plugin-log-api';

	import type { Config } from '../../../src-tauri/bindings/Config';

	let username = '';
	let server_address = '';
	let client_name = `unknown_client_${randomString(10)}`;
	let server_port = 22;
	let state: ButtonState = 'idle';
	let error: App.Error | undefined;

	$: adressIsValid = isUrl(server_address) || isIp(server_address);
	$: portIsValid = isNumber(server_port) && server_port > 0 && server_port < 65536;
	$: usernameIsValid = username.length > 0;
	$: if ($serverConfig != undefined && !error) goto('/');

	const submit = async () => {
		error = undefined;
		if (!usernameIsValid) {
			error = { message: 'Invalid username' };
			return;
		}
		if (!adressIsValid) {
			error = { message: 'Invalid server adress' };
			return;
		}
		if (!portIsValid) {
			error = { message: 'Invalid server port' };
			return;
		}
		state = 'loading';

		const newConfig: Config = {
			client_name,
			username,
			server_address,
			server_port,
			allow_background_backup: true
		};

		// Test connection
		await invoke('set_state', { config: newConfig })
			.then(() => {
				state = 'success';
			})
			.catch((e) => {
				state = 'error';
				serverConfig.set(undefined);
				error = {
					message: "Couldn't establish a server connection based on your config"
				};
				logError(`Client log: ${error.message}: ${JSON.stringify(e)}`);
			});

		if (error) return;

		try {
			await writeTextFile(SERVER_CONFIG_FILE_NAME, JSON.stringify(newConfig), {
				dir: BaseDirectory.AppConfig
			});
			serverConfig.set(newConfig);
		} catch (e) {
			state = 'error';
			error = { message: 'Could not write config file' };
			logError(`Client log: ${error.message}: ${JSON.stringify(e)}`);
		}
	};

	onMount(async () => {
		try {
			client_name = await invoke('get_client_name');
		} catch (e) {
			error = { message: 'Could not get client name' };
			logError(`Client log: ${error.message}: ${JSON.stringify(e)}`);
		}
	});
</script>

<div class={$clientConfig.theme}>
	<h1>Setup</h1>

	{#if error}
		<p class="error">{error.message}</p>
	{/if}

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

	<Button type="primary" onClick={submit} {state}>
		Connect
		<PlugIcon slot="icon" />
	</Button>
</div>

<style lang="scss">
	@import '../../lib/style/mixins.scss';

	.error {
		color: $clr-danger;
	}

	.input_group {
		display: flex;
		flex-direction: column;
		margin-bottom: 1rem;

		input {
			width: 100%;
			max-width: 20rem;
			padding: 0.5rem;
			border: none;
			border-radius: 0.5rem;
			@include text-sm;
		}
	}

	.light {
		input {
			background-color: $clr-foreground;
		}
	}
</style>
