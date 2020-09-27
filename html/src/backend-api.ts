type FetchFn = typeof fetch;

import backend_config from "./backend-config.json";

export class APIError extends Error {
	status: number;
	constructor(status: number, text: string) {
		super(`Server responded with ${status}: ${text}`);
		this.status = status;
	}
}
export class NetworkError extends Error {
	inner: Error;
	constructor(inner: Error) {
		if (typeof navigator === "undefined" || navigator.onLine) {
			super(`Can not communicate with backend server: ${inner.message}`);
		} else {
			super("You're offline.");
		}
		this.inner = inner;
	}
}

export async function get_json<T>(path: string, fetch: FetchFn): Promise<T> {
	try {
		let res = await fetch(backend_config.url + path);
		if (res.status !== 200) {
			let text = await res.text();
			throw new APIError(res.status, text);
		}
		return (await res.json()) as T;
	} catch (e) {
		if (e instanceof APIError) {
			throw e;
		}
		throw new NetworkError(e);
	}
}

export type Stats = {
	nb_logs_active: number,
	nb_logs_total: number
};

export function stats(fetch: FetchFn): Promise<Stats> {
	return get_json<Stats>("/stats", fetch);
}

export type CtLogs = Array<BasicCtLogInfo>;
export type BasicCtLogInfo = {
	log_id: string,
	name: string,
	monitoring: boolean,
	endpoint_url: string,
	latest_sth: BasicSthInfo | null,
	last_sth_error: string | null
};
export type BasicSthInfo = {
	id: number,
	tree_size: number,
	tree_hash: string,
	received_time: number,
	sth_timestamp: number
}

export function ctlogs(fetch: FetchFn, include_retired: boolean): Promise<CtLogs> {
	return get_json<CtLogs>(`/ctlogs?include_retired=${include_retired}`, fetch);
}

export type CtLog = {
	log_id: string,
	endpoint_url: string,
	name: string,
	public_key: string, // base64
	monitoring: boolean,
	latest_sth: number | null,
	last_sth_error: string | null
}

export function log(fetch: FetchFn, id: string): Promise<CtLog> {
	return get_json<CtLog>("/log/" + encodeURIComponent(id), fetch);
}

export type Sth = {
	id: number,
	log_id: string,
	tree_hash: string,
	tree_size: number,
	sth_timestamp: number,
	received_time: number,
	signature: string,
	checked_consistent_with_latest: boolean
}

export function sth(fetch: FetchFn, log_id: string, sth_id: number): Promise<Sth> {
	return get_json<Sth>(`/log/${encodeURIComponent(log_id)}/sth/${sth_id}`, fetch);
}
