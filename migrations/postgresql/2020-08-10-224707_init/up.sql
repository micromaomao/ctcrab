CREATE TABLE ctlogs (
    "log_id" bytea UNIQUE NOT NULL PRIMARY KEY,
    "endpoint_url" text NOT NULL,
    "name" text NOT NULL,
    "public_key" bytea NOT NULL,
    "monitoring" boolean NOT NULL,
    "latest_sth" bigint DEFAULT NULL,
    "last_sth_error" text DEFAULT NULL
);

CREATE TABLE sth (
    "id" bigserial UNIQUE NOT NULL PRIMARY KEY,
    "log_id" bytea NOT NULL REFERENCES ctlogs("log_id"),
    "tree_hash" bytea NOT NULL,
    "tree_size" bigint NOT NULL,
    "sth_timestamp" bigint NOT NULL, -- ms
    "received_time" timestamp with time zone NOT NULL DEFAULT now(),
    "signature" bytea NOT NULL,
    "checked_consistent_with_latest" boolean NOT NULL DEFAULT false
);

ALTER TABLE ctlogs ADD FOREIGN KEY ("latest_sth") REFERENCES sth("id");

CREATE UNIQUE INDEX sth_i ON sth ("log_id", "tree_size", "tree_hash", "sth_timestamp");
CREATE INDEX sth_unchecked ON sth ("log_id", "tree_size") WHERE "checked_consistent_with_latest" = false;

CREATE TABLE consistency_check_errors (
    "id" bigserial UNIQUE NOT NULL PRIMARY KEY,
    "log_id" bytea NOT NULL REFERENCES ctlogs("log_id"),
    "from_sth_id" bigint NOT NULL REFERENCES sth("id"),
    "to_sth_id" bigint NOT NULL REFERENCES sth("id"),
    "discovery_time" timestamp with time zone NOT NULL DEFAULT now(),
    "last_check_time" timestamp with time zone NOT NULL DEFAULT now(),
    "last_check_error" text NOT NULL
);

CREATE UNIQUE INDEX consistency_check_errors_i ON consistency_check_errors ("log_id", "to_sth_id", "from_sth_id");

CREATE TABLE cert_fetch_errors (
    "id" bigserial UNIQUE NOT NULL PRIMARY KEY,
    "log_id" bytea NOT NULL REFERENCES ctlogs("log_id"),
    "from_tree_size" bigint NOT NULL,
    "to_tree_size" bigint NOT NULL,
    "error_time" timestamp with time zone NOT NULL DEFAULT now(),
    "error_msg" text NOT NULL
);

CREATE TABLE certificates (
    "fingerprint" bytea UNIQUE NOT NULL PRIMARY KEY, -- sha256
    "x509" bytea NOT NULL -- der blob
);

CREATE TABLE certificate_chain (
    "__pk" bigserial UNIQUE NOT NULL PRIMARY KEY,
    "certificate_fingerprint" bytea NOT NULL REFERENCES certificates("fingerprint"),
    "chain" bytea[] NOT NULL -- der blobs of parents
);

-- CREATE UNIQUE INDEX certificate_chain_dup_check ON certificate_chain ("certificate_fingerprint", "chain");
-- FIXME: the above index will fail if chain gets too large.

CREATE TABLE certificate_appears_in_leaf (
    "__pk" bigserial UNIQUE NOT NULL PRIMARY KEY,
    "leaf_hash" bytea NOT NULL,
    "cert_fp" bytea NOT NULL REFERENCES "certificates"("fingerprint"),
    "log_id" bytea NOT NULL REFERENCES "ctlogs"("log_id"),
    "leaf_index" bigint NOT NULL
);

CREATE INDEX certificate_appears_in_leaf_by_cert ON certificate_appears_in_leaf ("cert_fp", "log_id");
CREATE UNIQUE INDEX certificate_appears_in_leaf_by_leaf ON certificate_appears_in_leaf ("log_id", "leaf_index");
CREATE UNIQUE INDEX certificate_appears_in_leaf_by_leaf_hash ON certificate_appears_in_leaf ("log_id", "leaf_hash");

CREATE TABLE retired_log_changed_error (
    "log_id" bytea UNIQUE NOT NULL PRIMARY KEY REFERENCES ctlogs("log_id"),
    "latest_sth" bigint NOT NULL REFERENCES sth("id")
);

CREATE TABLE certificate_dns_names (
    "__pk" bigserial UNIQUE NOT NULL PRIMARY KEY,
    "cert_fp" bytea NOT NULL REFERENCES "certificates"("fingerprint"),
    "dns_name" text NOT NULL
);

CREATE INDEX certificate_dns_names_suffix_ind ON certificate_dns_names (reverse("dns_name"));
CREATE UNIQUE INDEX certificate_dns_names_dup_check ON certificate_dns_names ("cert_fp", "dns_name");
