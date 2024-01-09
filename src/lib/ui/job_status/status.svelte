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

	let show = false;
	let state: ButtonState = 'idle';
	let expanded = false;

	const filter = (j: App.Job) => j.__type === 'single';

	$: iconColor = $clientConfig.theme === 'dark' ? 'var(--clr-text_light)' : 'var(--clr-text_dark)';
	$: jobs.filter(filter).length > 0 && (state = 'loading');
	$: state === 'idle' ? (show = false) : (show = true);
	$: if (jobs.filter(filter).length === 0) {
		if (completedJobs.some(filter) && failedJobs.length == 0) {
			state = 'success';
		} else if (failedJobs.some(filter)) {
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
						<ErrorIcon color="var(--clr-danger)" />
					</Item>
				{/each}
				{#each completedJobs as job}
					<Item {job}>
						<CheckmarkIcon color="var(--clr-success)" />
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
					Running jobs: {jobs.filter(filter).length}
				{:else}
					<span>
						Failed jobs: {failedJobs.filter(filter).length}
					</span>
					<span>
						Completed jobs: {completedJobs.filter(filter).length}
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
