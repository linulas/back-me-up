<script lang="ts">
	import TadpoleIcon from '~icons/svg-spinners/tadpole';
	import ErrorIcon from '~icons/ion/alert-circle';
	import CheckmarkIcon from '~icons/ion/checkmark';
	import { clientConfig } from '$lib/store';

	export let state: ButtonState = 'idle';
	export let loadingColor: string =
		$clientConfig.theme === 'dark' ? 'var(--clr-text_light)' : 'var(--clr-text_dark)';
</script>

{#if state === 'loading'}
	<TadpoleIcon color={loadingColor} />
{:else if state === 'error'}
	<ErrorIcon color="var(--clr-danger)" />
{:else if state === 'success'}
	<CheckmarkIcon color="var(--clr-success)" />
{:else}
	<slot name="icon" />
	<slot name="secondary-icon" />
{/if}
