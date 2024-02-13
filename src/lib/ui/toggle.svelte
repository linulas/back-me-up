<script lang="ts">
	export let options: string[] = [];
	export let checked = false;
</script>

<div class="toggle">
	{#if options[0]}
		<span class:inactive={checked}>
			{options[0]}
		</span>
	{/if}

	<label class="switch">
		<input type="checkbox" bind:checked />
		<span class="slider round" />
	</label>

	{#if options[1]}
		<span class:inactive={!checked}>
			{options[1]}
		</span>
	{/if}
</div>

<style lang="scss">
	.toggle {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.inactive {
		color: $clr-text_muted;
	}
	/* The switch - the box around the slider */
	.switch {
		position: relative;
		display: inline-block;
		width: 30px;
		height: 17px;
	}

	/* Hide default HTML checkbox */
	input {
		opacity: 0;
		width: 0;
		height: 0;
	}

	/* The slider */
	.slider {
		position: absolute;
		cursor: pointer;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: $sky-500;
		-webkit-transition: 0.4s;
		transition: 0.4s;
	}

	.slider:before {
		position: absolute;
		content: '';
		height: 13px;
		width: 13px;
		left: 2px;
		bottom: 2px;
		background-color: white;
		-webkit-transition: 0.4s;
		transition: 0.4s;
	}

	input:checked + .slider {
		background-color: $fuchsia-500;
	}

	input:focus + .slider {
		box-shadow: 0 0 1px $fuchsia-500;
	}

	input:checked + .slider:before {
		-webkit-transform: translateX(13px);
		-ms-transform: translateX(13px);
		transform: translateX(13px);
	}

	/* Rounded sliders */
	.slider.round {
		border-radius: 34px;
	}

	.slider.round:before {
		border-radius: 50%;
	}
</style>
