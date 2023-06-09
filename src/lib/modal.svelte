<script lang="ts">
  import { onMount } from 'svelte';
	import { clientConfig } from './store';
  
  export let open: boolean;
  function closeModal() {
    open = false;
  }
  onMount(() => {
    document.addEventListener('keydown', (event) => {
      if (event.key === 'Escape') {
        closeModal();
      }
    });
  });
</script>
{#if open}
  <div class={`modal ${$clientConfig.theme}`}>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <div class="overlay" on:click={closeModal}></div>
    <div class="content">
      <slot />
    </div>
  </div>
{/if}
<style lang="scss">
  .modal {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 9999;
  }
  .overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
  }
  .content {
    position: relative;
    background-color: $clr-background;
    padding: 1rem;
    border-radius: 0.5rem;
    box-shadow: 0 0.5rem 1;
    min-height: 25vh;
  }

  .dark {
    .content {
      background-color: $clr-background_dark;
    }
  }
</style>
