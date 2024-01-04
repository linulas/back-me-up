<script lang="ts">
	import ArrowIcon from '~icons/ion/arrow-forward';
	import { clientConfig } from '$lib/store';
	export let job: App.Job;

	const isBackupFolderJob = (job: App.Job): job is App.BackupFolderJob => {
		return (job as any).from !== undefined && (job as any).to !== undefined;
	};
</script>

<div class="item">
	{#if isBackupFolderJob(job)}
		<span>
			<slot />
		</span>
		<span>
			{job.from?.name}
		</span>
		<span>
			<ArrowIcon color={$clientConfig.theme === 'dark' ? 'white' : 'black'} />
		</span>
		<span>
			{job.to?.name}
		</span>
	{/if}
</div>

<style lang="scss">
	.item {
		span {
			display: flex;
		}
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
