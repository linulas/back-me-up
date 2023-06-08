<script lang="ts">
	import TadpoleIcon from '~icons/svg-spinners/tadpole';
	import ErrorIcon from '~icons/ion/alert-circle';
	import CheckmarkIcon from '~icons/ion/checkmark';

	export let type: ButtonType = 'primary';
	export let state: ButtonState = 'idle';

	export let onClick: () => void;
  export let style = "";
</script>

<button class={`${type} ${state}`} on:click={onClick} {style}>
	<slot />
	{#if $$slots.icon}
		<span>
			{#if state === 'loading'}
				<TadpoleIcon color="white" />
			{:else if state === 'error'}
				<ErrorIcon color="#ef4444" />
			{:else if state === 'success'}
				<CheckmarkIcon color="#10b981" />
			{:else}
				<slot name="icon" class="icon_slot" />
			{/if}
		</span>
	{/if}
</button>

<style lang="scss">
	@import './style/mixins.scss';
	button {
		@include box;
		padding: 0.5rem;
		border: none;
		cursor: pointer;
		transition: all 0.1s ease-in-out;
		display: flex;
		align-items: center;
		justify-content: center;

		&:hover {
			transform: scale(1.05);
			background: $clr-primary-action_hover;
			-webkit-box-shadow: -1px 1px 4px 2px rgba(0, 0, 0, 1);
			-moz-box-shadow: -1px 1px 4px 2px rgba(0, 0, 0, 1);
			box-shadow: -1px 1px 4px 2px rgba(0, 0, 0, 1);
		}
		&:active {
			transform: scale(1.1);
		}
	}
	span {
		margin-top: 6px;
	}

	.primary {
		background: $clr-primary-action;
		color: $clr-text_light;
		&:hover {
			background: $clr-primary-action_hover;
		}
	}

	.secondary {
		background: $clr-secondary-action;
		color: $clr-text_light;
		&:hover {
			background: $clr-secondary-action_hover;
		}
	}

	.icon {
		background: transparent;
		box-shadow: none;

		&:hover {
			background: none;
			box-shadow: none;
		}
	}

	.icon-with_background {
		background: $clr-secondary-action;
    width: 2rem;
    height: 2rem;
    border-radius: 50%;

		&:hover {
			background: $clr-secondary-action_hover;
		}
	}
</style>
