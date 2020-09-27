<script lang="ts" context="module">
	import type { CtLogs } from "../backend-api";
	import LoglistEntry from "./LoglistEntry.svelte";
</script>

<script lang="ts">
	export let ctlogs: CtLogs;
</script>

<ul>
	{#each ctlogs as l}
		<LoglistEntry log={l} />
	{/each}
</ul>
