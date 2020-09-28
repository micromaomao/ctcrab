<script lang="ts">
	export let heading: string;
	import { url as backend_url } from "../backend-config.json";
	export let apilink: string | null;
</script>

<h2>
	{heading}
	{#if typeof apilink == "string"}
		<a class="apilink" href="{backend_url}{apilink}" target="_blank">
			<span class="icon">code</span> json
		</a>
	{/if}
</h2>

<style>
	h2 {
		font-size: 1.5em;
		font-weight: normal;
		margin: 5px 10px;
		padding: 5px 0 5px 20px;
		color: var(--color-accent);
		border-bottom: rgba(35,87,137, 0.5) solid 1px;
	}

	a.apilink {
		color: rgba(72,67,73, 0.7);
		text-decoration: none;
		margin-left: 10px;
		font-size: 0.6em;
	}
</style>
