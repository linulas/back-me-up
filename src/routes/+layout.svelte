<main>
	<slot />
</main>

<style lang="scss">
	@import '../lib/style/mixins.scss';

	:global(body) {
		background: $clr-background;
		color: $clr-text_light;
		font-family: $font-sans-serif;
		@include text-sm;
		@media screen and (min-width: $media-sm) {
			@include text-base;
		}
	}
	:global(p, span, div, button, a, li, label) {
		@include text-sm;
		@media screen and (min-width: $media-sm) {
			@include text-base;
		}
	}

	main {
		width: calc(100% - 2rem);
    margin: auto;
	}
</style>
