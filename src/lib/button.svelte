<script lang="ts">
	import TadpoleIcon from '~icons/svg-spinners/tadpole';
	import ErrorIcon from '~icons/ion/alert-circle';
	import CheckmarkIcon from '~icons/ion/checkmark';
	import { clientConfig } from './store';

	export let type: ButtonType = 'primary';
	export let state: ButtonState = 'idle';

	export let onClick: () => void;
	export let style = '';
  export let loadingColor: string = "white";
</script>

<button class={`${type} ${state} ${$clientConfig.theme}`} on:click={onClick} {style}>
	<slot />
	{#if $$slots.icon}
		<span>
			{#if state === 'loading'}
				<TadpoleIcon color={loadingColor} />
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
			-webkit-box-shadow: -1px 1px 2px 1px rgba(0, 0, 0, 0.5);
			-moz-box-shadow: -1px 1px 2px 1px rgba(0, 0, 0, 0.5);
			box-shadow: -1px 1px 2px 1px rgba(0, 0, 0, 0.5);
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
		span {
			margin-left: 0.5rem;
		}
	}

	.secondary {
		background: $clr-secondary-action;
		color: $clr-text_light;
		&:hover {
			background: $clr-secondary-action_hover;
		}
		span {
			margin-left: 0.5rem;
		}
	}

	.danger {
		background: $clr-danger;
		color: $clr-text_light;
		&:hover {
			background: $red-600;
		}
		span {
			margin-left: 0.5rem;
		}
	}

	.icon {
		background: transparent;
		box-shadow: none;

		&:hover {
			transform: none;
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

	.dark {
		&.icon-with_background {
			background: $clr-secondary-action_dark;

			&:hover {
				background: $clr-secondary-action_hover_dark;
			}
		}
		&.secondary {
			background: $clr-secondary-action_dark;
			color: $clr-text_light;
			&:hover {
				background: $clr-secondary-action_hover_dark;
			}
		}
	}
</style>
