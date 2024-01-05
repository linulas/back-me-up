<script lang="ts">
	import ArrowIcon from '~icons/ion/arrow-forward';
	import { clientConfig } from '$lib/store';
	export let job: App.BackupFolderJob;
</script>

<div class="item">
	{#if job.__type === 'single'}
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
