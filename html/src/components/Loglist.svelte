<script lang="ts" context="module">
	import type { CtLogs } from "../backend-api";
	import LoglistEntry from "./LoglistEntry.svelte";
	import Heading from "../components/Heading.svelte";
</script>

<script lang="ts">
	export let ctlogs: CtLogs;
</script>

<Heading heading="Logs" apilink="/ctlogs?include_retired=true" />
<ul>
	{#each ctlogs as l}
		<LoglistEntry log={l} />
	{/each}
</ul>

<style>
	ul {
		margin: 10px 5px;
	}
</style>
