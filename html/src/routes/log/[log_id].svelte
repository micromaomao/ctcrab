<script lang="ts" context="module">
	import Topbar from "../../components/Topbar.svelte";
	import type { CtLog, Sth } from "../../backend-api";
	import { log as get_log, sth as get_sth, APIError } from "../../backend-api";
	import Heading from "../../components/Heading.svelte";
	import { rough_duration } from "../../humantime";
	import { onMount } from "svelte";

	export async function preload({ params: { log_id } }: { params: {log_id: string} }): Promise<any> {
		try {
			let log = await get_log(this.fetch, log_id);
			let prefetched_sths: Record<string, Sth> = {};
			if (log.latest_sth !== null) {
				prefetched_sths[log.latest_sth.toString()] = await get_sth(this.fetch, log_id, log.latest_sth);
			}
			return {
				log_id,
				log,
				prefetched_sths
			};
		} catch (e) {
			if (e instanceof APIError && e.status === 404) {
				this.error(404, "Log not found.");
				return;
			}
			throw e;
		}
	}
</script>

<script lang="ts">import type { monitorEventLoopDelay } from "perf_hooks";

	export let log_id: string;
	export let log: CtLog;
	export let prefetched_sths: Record<string, Sth>;
	$: latest_sth = log && log.latest_sth !== null ? prefetched_sths[log.latest_sth.toString()] : null;

	async function lookup_sth(sth_id: number): Promise<Sth> {
		if (prefetched_sths.hasOwnProperty(sth_id.toString())) {
			return prefetched_sths[sth_id.toString()];
		} else {
			return await get_sth(fetch, log_id, sth_id);
		}
	}

	$: latency_ms = latest_sth ? Date.now() - Math.min(latest_sth.received_time, latest_sth.sth_timestamp) : null;

	onMount(() => {
		let updateInterval = setInterval(() => {
			if (latest_sth) {
				latest_sth = latest_sth;
			}
		}, 500);
		return () => clearInterval(updateInterval);
	});

	onMount(() => {
		let umounted = false;
		let nextTimeout: number | null = null;
		let fn = async () => {
			nextTimeout = null;
			if (umounted) return;
			try {
				let new_log_info = await get_log(fetch, log_id);
				if (umounted) return;
				let new_latest_sth: Sth | null = null;
				if (new_log_info.latest_sth !== null && !prefetched_sths.hasOwnProperty(new_log_info.latest_sth.toString())) {
					new_latest_sth = await get_sth(fetch, log_id, new_log_info.latest_sth);
				}
				if (umounted) return;
				if (new_latest_sth !== null) {
					prefetched_sths[new_latest_sth.id.toString()] = new_latest_sth;
				}
				log = new_log_info;
				nextTimeout = setTimeout(fn, 5000);
			} catch (e) {
				if (umounted) return;
				nextTimeout = setTimeout(fn, 5000);
			}
		};
		nextTimeout = setTimeout(fn, 5000);
		return () => {
			umounted = true;
			if (nextTimeout !== null) {
				clearTimeout(nextTimeout);
			}
		}
	});
</script>

<Topbar home={true}>
	<span slot="title">{log.name}</span>
	<div slot="desc">
		<a href="{log.endpoint_url}" class="log-endpoint-link" target="_blank" rel="noopener">{log.endpoint_url.replace(/^https:\/\//, "")}</a>
	</div>
</Topbar>

<Heading heading="CT log" apilink="/log/{log_id}" />
<div class="loginfo">
	{#if log.monitoring}
		<span class="green">
			<span class="icon">checkmark</span> monitoring
		</span>
	{:else}
		<span class="red">
			<span class="icon">pause</span> retired
		</span>
	{/if}
	<span class="sep">&centerdot;</span>
	<a href="data:application/octet-stream;base64,{log.public_key}" download="{log.name.replace(/ /g, "-")}.der">
		<span class="icon">key</span> public key
	</a>
	{#if latest_sth}
	<span class="sep">&centerdot;</span>
	<span class="icon">tree</span> {latest_sth.tree_size}
	<span class="sep">&centerdot;</span>
	<span class="icon">latency</span> {rough_duration(latency_ms)}
	{/if}
</div>

{#if log.last_sth_error}
	<div class="sth-error">
		<b><span class="icon">error</span> Can't fetch latest STH</b><br>
		{log.last_sth_error}
	</div>
{/if}

{#if log.monitoring && latency_ms !== null && latency_ms > 1000*60*60*24}
	<div class="latency-warning">
		<b><span class="icon">error</span> Last signed tree head was produced {rough_duration(latency_ms)} ago.</b><br>
		Usually logs have a maximum delay limit of 24h.
	</div>
{/if}

{#if !log.monitoring}
	<div class="retired-warning">This log is not being monitored because it was marked as "retired".</div>
{/if}

<style>
	.log-endpoint-link {
		border-bottom: dashed 1px var(--color-bg);
	}

	.loginfo {
		text-align: center;
		padding: 5px;
	}

	span.green {
		color: var(--color-green);
	}
	span.red {
		color: var(--color-red);
	}

	span.sep {
		display: inline-block;
		margin: 0 5px;
	}

	.sth-error, .latency-warning {
		margin: 5px auto;
		max-width: 800px;
		padding: 10px;
	}
	.sth-error {
		background-color: rgba(150,16,27, 0.1);
		border: solid 2px var(--color-red);
	}
	.latency-warning {
		background-color: rgba(241,237,20, 0.1);
		border: solid 2px var(--color-yellow);
	}

	.sth-error b, .latency-warning b {
		font-size: 1.1em;
		line-height: 1.3;
	}
	.sth-error b {
		color: var(--color-red);
	}
	.latency-warning b {
		color: var(--color-accent);
	}

	.retired-warning {
		color: var(--color-accent);
		text-align: center;
		padding: 5px;
		font-weight: bold;
	}
</style>
