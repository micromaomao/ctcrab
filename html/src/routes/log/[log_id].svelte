<script lang="ts" context="module">
	import Topbar from "../../components/Topbar.svelte";
	import type { CtLogDetail } from "../../backend-api";
	import { log as get_log } from "../../backend-api";

	export async function preload({ params }: { params: {log_id: string} }): Promise<any> {
		return {
			log_id: params.log_id,
			log: await get_log(this.fetch, params.log_id)
		};
	}
</script>

<script lang="ts">
	export let log_id: string;
	export let log: CtLogDetail;
</script>

<Topbar home={true}>
	<span slot="title">{log.name}</span>
	<div slot="desc">{log.endpoint_url.replace(/^https:\/\//, "")}</div>
</Topbar>
