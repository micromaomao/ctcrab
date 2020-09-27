<script lang="ts" context="module">
	import type { BasicCtLogInfo } from "../backend-api";
	import { rough_duration } from "../humantime";
	import { onMount } from "svelte";
</script>
<script lang="ts">
	export let log: BasicCtLogInfo;
	$: latest_sth = log.latest_sth;
	$: lsth_effective_timestamp = latest_sth ? new Date(Math.min(latest_sth.received_time, latest_sth.sth_timestamp)) : null;
	$: latency_ms = lsth_effective_timestamp ? (Date.now() - lsth_effective_timestamp.getTime()) : null;
	$: late = lsth_effective_timestamp ? (latency_ms > (1000*60*60*24)) : null

	onMount(() => {
		// update latency display
		let updateInterval = setInterval(() => {
			log = log;
		}, 500);
		return () => {
			clearInterval(updateInterval);
		}
	});
</script>

<li class:retired={!log.monitoring}>
	<a class="name" href="/log/{log.log_id}">{log.name}</a>
	{#if latest_sth}
		<span>
			<span class="icon">tree</span> {latest_sth.tree_size}
		</span>
		<span class:late={late}>
			<span class="icon">latency</span> <time datetime={lsth_effective_timestamp.toISOString()}>{rough_duration(latency_ms)}</time>
		</span>
	{/if}
	{#if !log.monitoring}
	<span>
		<span class="icon">pause</span> retired
	</span>
	{/if}
	<a class="endpoint" href="{log.endpoint_url}" target="_blank" rel="noopener">
		<span class="icon">link</span> {log.endpoint_url.replace(/^https:\/\//, "")}
	</a>
	{#if log.last_sth_error}
		<div class="error">
			<span class="icon">error</span> {log.last_sth_error}
		</div>
	{/if}
</li>

<style>
	li {
		margin: 5px 0;
		list-style-type: none;
	}
	li.retired {
		opacity: 0.7;
	}
	a.name {
		display: inline-block;
		padding: 5px;
		margin-left: -5px;
		font-size: 1.1em;
		color: var(--color-accent);
		margin-right: 30px;
	}
	li > a, li > span {
		margin-right: 20px;
	}
	a.endpoint {
		color: rgba(72,67,73, 0.8)
	}
	span.late {
		color: var(--color-red);
	}
	.error {
		color: var(--color-red);
	}
	.error > span.icon {
		display: inline-block;
		margin-right: 5px;
	}
</style>
