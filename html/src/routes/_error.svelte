<script lang="ts" context="module">
	import Topbar from "../components/Topbar.svelte";
</script>
<script lang="ts">
	export let error: Error;
	export let status: number;
</script>

<Topbar>
	<span slot="title">Whoops!</span>
	<div slot="desc">{error.message}</div>
</Topbar>

{#if status === 404}
	<p>The page you're requesting does not exist. <a href="/">Return home</a></p>
{:else}
	<pre>{error.stack}</pre>
{/if}

<style>
	pre {
		padding: 10px 20px;
		overflow-x: auto;
	}
	p {
		padding: 20px;
		max-width: 1000px;
		margin: 0 auto;
	}
	a {
		color: var(--color-accent);
	}
</style>
