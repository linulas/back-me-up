<script lang="ts">
	import { onMount } from 'svelte';
	import { clientConfig } from './store';

	interface Item {
		title: string;
		value: string;
	}

	export let items: Item[] = [];
	export let value: string | undefined;

	let isOpen = false;

	function selectItem(item: Item) {
		value = item.value;
		isOpen = false;
	}

	let outsideClickListener: (event: MouseEvent) => void;

	onMount(() => {
		value = items[0]?.value;
		outsideClickListener = (event: MouseEvent) => {
			const target = event.target as HTMLElement;
			if (!target.closest('.select')) {
				isOpen = false;
				window.removeEventListener('click', outsideClickListener);
			}
		};
	});

	function toggleOpen() {
		isOpen = !isOpen;
		if (isOpen) {
			window.addEventListener('click', outsideClickListener);
		} else {
			window.removeEventListener('click', outsideClickListener);
		}
	}
</script>

<div class={`select ${$clientConfig.theme}`}>
	<div
		class="toggle"
		on:click={toggleOpen}
		on:keydown={(e) => e.key === 'Enter' && toggleOpen()}
	>
		{#each items as item}
			{#if value === item.value}
				<span>{item.title}</span>
			{/if}
		{/each}
		<span>{isOpen ? '▲' : '▼'}</span>
	</div>

	{#if isOpen}
		{#if items.length > 0}
			<ul class="options">
				{#each items as item}
					<li
						class="option"
						on:click={() => selectItem(item)}
						on:keydown={(e) => e.key === 'Enter' && selectItem(item)}
					>
						{item.title}
					</li>
				{/each}
			</ul>
		{:else}
			<p>No items to choose from</p>
		{/if}
	{/if}
</div>

<style lang="scss">
	.select {
		position: relative;
		display: inline-block;
	}

	.toggle {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.5rem 1rem;
		background-color: $clr-foreground;
		border: 1px solid $clr-border;
		cursor: pointer;
	}

	.options {
		position: absolute;
		top: 100%;
		left: 0;
		width: 100%;
		list-style: none;
		background-color: $clr-foreground;
		border: 1px solid $clr-border;
		padding: 0;
		margin: 0;
		max-height: 150px;
		overflow-y: auto;
	}

	.option {
		padding: 0.5rem 1rem;
		cursor: pointer;

		&:hover {
			background-color: $slate-300;
		}
	}

  .dark {
    .toggle, .options {
      background-color: $clr-secondary-action_dark;
      border-color: $clr-border_dark;
    }

    .options:hover {
      background-color: $clr-secondary-action_hover_dark;
    }
  }
</style>
