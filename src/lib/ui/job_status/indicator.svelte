<script lang="ts">
	import { fly } from 'svelte/transition';
	import { clientConfig } from '$lib/store';
	import LoadingState from '../loading_state.svelte';
	import Button from '../button.svelte';
	import CloseIcon from '~icons/ion/close-outline';
	import ErrorIcon from '~icons/ion/alert-circle';
	import CheckmarkIcon from '~icons/ion/checkmark';
	import TadpoleIcon from '~icons/svg-spinners/tadpole';
	import Item from './item.svelte';

	export let jobs: App.Job[] = [];
	export let completedJobs: App.Job[] = [];
	export let failedJobs: App.Job[] = [];
	export let onClear: () => void = () => {};

	let show = jobs.length > 0 || completedJobs.length > 0 || failedJobs.length > 0;
	let state: ButtonState = 'idle';
	let expanded = false;

	$: iconColor = $clientConfig.theme === 'dark' ? 'white' : 'black';
	$: jobs.length > 0 && (state = 'loading');
	$: state === 'idle' ? (show = false) : (show = true);
	$: if (jobs.length === 0) {
		if (completedJobs.length > 0 && failedJobs.length == 0) {
			state = 'success';
		} else if (failedJobs.length > 0) {
			state = 'error';
		}
	}

	$: if (state === 'success') {
		setTimeout(() => {
			clear();
		}, 5000);
	}

	const clear = () => {
		expanded = false;
		onClear();
		state = 'idle';
	};
</script>

{#if show}
	<div class="job-status" transition:fly={{ x: 200, duration: 500 }}>
		{#if expanded}
			<div
				class={`info ${$clientConfig.theme}`}
				style={`bottom: ${jobs.length > 0 ? '6rem' : '7.5rem'}`}
			>
				{#each jobs as job}
					<Item {job}>
						<TadpoleIcon color={iconColor} />
					</Item>
				{/each}
				{#each failedJobs as job}
					<Item {job}>
						<ErrorIcon color="#ef4444" />
					</Item>
				{/each}
				{#each completedJobs as job}
					<Item {job}>
						<CheckmarkIcon color="#10b981" />
					</Item>
				{/each}
			</div>
		{/if}
		<!-- svelte-ignore a11y-click-events-have-key-events -->
		<div on:click={() => (expanded = !expanded)} class={`indicator ${$clientConfig.theme}`}>
			<span class="icon">
				<LoadingState {state} loadingColor={iconColor} />
			</span>
			<span class="text">
				{#if jobs.length > 0}
					Running jobs: {jobs.length}
				{:else}
					<span>
						Failed jobs: {failedJobs.length}
					</span>
					<span>
						Completed jobs: {completedJobs.length}
					</span>
				{/if}
			</span>
			<Button type="icon" onClick={clear} disabled={state === 'loading'} style="padding: 0;">
				<CloseIcon color={iconColor} />
			</Button>
		</div>
	</div>
{/if}

<style lang="scss">
	.job-status {
		position: fixed;
		bottom: 0;
		width: 100vw;
	}

	.info,
	.indicator {
		@include box;
		background: $clr-foreground;
		padding: 1rem;
		width: 12rem;
	}

	.info {
		position: fixed;
		bottom: 6rem;
		right: 2rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.indicator {
		position: fixed;
		bottom: 2rem;
		right: 2rem;
		display: flex;
		align-items: center;
		justify-content: space-between;
		cursor: pointer;

		.text {
			display: flex;
			flex-direction: column;
		}
	}

	.icon {
		display: flex;
	}

	.dark {
		background: $clr-foreground_dark;
		color: $clr-text_light;
	}
</style>
