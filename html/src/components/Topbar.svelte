<script lang="ts">
	export let home: boolean = false;
	import { stores } from "@sapper/app";
	const { preloading } = stores();
</script>
<div class="topbar" class:loading={$preloading}>
	{#if home}
		<div class="back-contain">
			<a href="/" class="back icon">home</a>
		</div>
	{/if}
	<h1>
		<slot name="title">???</slot>
	</h1>
	<slot name="desc"></slot>
</div>

<style>
	.topbar {
		background-color: var(--color-accent);
		color: var(--color-bg);
		text-align: center;
		padding: 10px;
	}

	h1 {
		margin: 0;
		padding: 0;
		font-weight: bold;
		font-size: 1.5em;
	}

	.back-contain {
		width: 0;
		height: 30px;
		overflow: visible;
		float: left;
		padding: 5px;
	}

	a.back {
		font-size: 20px;
		line-height: 20px;
	}

	.topbar.loading {
		animation: 0.5s ease-in-out 0s infinite alternate loading-anim;
	}

	@keyframes loading-anim {
		from {
			background-color: #235789;
		}
		to {
			background-color: #4E78A0;
		}
	}
</style>
