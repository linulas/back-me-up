<script lang="ts">
	import TadpoleIcon from '~icons/svg-spinners/tadpole';
	import ErrorIcon from '~icons/ion/alert-circle';
	import CheckmarkIcon from '~icons/ion/checkmark';

	export let state: ButtonState = 'idle';
	export let loadingColor: string = 'white';
</script>

{#if state === 'loading'}
  <TadpoleIcon color={loadingColor} />
{:else if state === 'error'}
  <ErrorIcon color="#ef4444" />
{:else if state === 'success'}
  <CheckmarkIcon color="#10b981" />
{:else}
  <slot name="icon" />
  <slot name="secondary-icon" />
{/if}
