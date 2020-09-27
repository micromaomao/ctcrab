type FetchFn = typeof fetch;

import backend_config from "./backend-config.json";

export class APIError extends Error {
	constructor(status: number, text: string) {
		super(`Server responded with ${status}: ${text}`);
	}
}
export class NetworkError extends Error {
	inner: Error;
	constructor(inner: Error) {
		super(`Can not communicate with backend server: ${inner.message}`);
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

export function ctlogs(fetch: FetchFn): Promise<CtLogs> {
	return get_json<CtLogs>("/ctlogs", fetch);
}

export type CtLogDetail = {
	log_id: string,
	endpoint_url: string,
	name: string,
	public_key: string, // base64
	monitoring: boolean,
	latest_sth: number | null,
	last_sth_error: string | null
}

export function log(fetch: FetchFn, id: string): Promise<CtLogDetail> {
	return get_json<CtLogDetail>("/log/" + encodeURIComponent(id), fetch);
}
