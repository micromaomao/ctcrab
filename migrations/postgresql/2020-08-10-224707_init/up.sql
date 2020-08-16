CREATE TABLE ctlogs (
    "log_id" bytea UNIQUE NOT NULL PRIMARY KEY,
    "endpoint_url" text NOT NULL,
    "name" text NOT NULL,
    "public_key" bytea NOT NULL,
    "monitoring" boolean NOT NULL,
    "latest_tree_hash" bytea DEFAULT NULL,
    "latest_tree_size" bigint DEFAULT NULL,
    "backward_tree_hash" bytea DEFAULT NULL,
    "backward_tree_size" bigint DEFAULT NULL
);

CREATE TABLE sth (
    "id" bigserial UNIQUE NOT NULL PRIMARY KEY,
    "log_id" bytea NOT NULL REFERENCES ctlogs("log_id"),
    "tree_hash" bytea NOT NULL,
    "tree_size" bytea NOT NULL,
    "sth_timestamp" bigint NOT NULL, -- ms
    "received_time" timestamp with time zone NOT NULL DEFAULT 'now',
    "signature" bytea NOT NULL,
    "consistent_with_latest" boolean NOT NULL DEFAULT false
);

CREATE UNIQUE INDEX sth_i ON sth ("tree_size", "tree_hash", "sth_timestamp");

CREATE TABLE consistency_check_errors (
    "id" bigserial UNIQUE NOT NULL PRIMARY KEY,
    "log_id" bytea NOT NULL REFERENCES ctlogs("log_id"),
    "from_sth_id" bigint NOT NULL REFERENCES sth("id"),
    "to_sth_id" bigint NOT NULL REFERENCES sth("id"),
    "discovery_time" timestamp with time zone NOT NULL DEFAULT 'now',
    "last_check_time" timestamp with time zone NOT NULL,
    "last_check_error" text NOT NULL
);

CREATE TABLE cert_fetch_errors (
    "id" bigserial UNIQUE NOT NULL PRIMARY KEY,
    "log_id" bytea NOT NULL REFERENCES ctlogs("log_id"),
    "from_tree_size" bigint NOT NULL,
    "to_tree_size" bigint NOT NULL,
    "error_time" timestamp with time zone NOT NULL DEFAULT 'now',
    "resolved" boolean DEFAULT false
);

CREATE TABLE retired_log_changed_error (
    "log_id" bytea UNIQUE NOT NULL PRIMARY KEY REFERENCES ctlogs("log_id"),
    "latest_sth" bigint NOT NULL REFERENCES sth("id")
);
