<script lang="ts" context="module">
	import Topbar from "../components/Topbar.svelte";
	import Loglist from "../components/Loglist.svelte";
	import { stats as get_stats, ctlogs as get_ctlogs } from "../backend-api";
	import type { Stats, CtLogs } from "../backend-api";
	import { onMount } from "svelte";

	export async function preload(): Promise<any> {
		let stats = await get_stats(this.fetch);
		let ctlogs = await get_ctlogs(this.fetch);
		return { stats, ctlogs };
	}
</script>

<script lang="ts">
	export let stats: Stats;
	export let ctlogs: CtLogs;

	onMount(() => {
		let timeoutHandle: number | null = null;
		let umounted: boolean = false;
		let fn = async () => {
			if (umounted) return;
			let new_props;
			try {
				new_props = await (preload.call({fetch}));
			} catch (e) {
				if (umounted) return;
				timeoutHandle = setTimeout(fn, 1000);
				return;
			}
			if (umounted) return;
			stats = new_props.stats;
			ctlogs = new_props.ctlogs;
			timeoutHandle = setTimeout(fn, 1000);
		};
		timeoutHandle = setTimeout(fn, 1000);
		return () => {
			clearTimeout(timeoutHandle);
			umounted = true;
		};
	});
</script>

<Topbar home={false}>
	<span slot="title">CtCrab</span>
	<div slot="desc">
		{stats.nb_logs_active} active logs ({stats.nb_logs_total - stats.nb_logs_active} retired), TODO certificates
	</div>
</Topbar>

<Loglist {ctlogs} />
