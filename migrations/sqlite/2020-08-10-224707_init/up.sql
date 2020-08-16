-- Use bigint instead of integer to make diesel generate correct schema.

CREATE TABLE ctlogs (
    "log_id" BLOB UNIQUE NOT NULL,
    "endpoint_url" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "public_key" BLOB NOT NULL,
    "monitoring" BOOLEAN NOT NULL,
    "latest_tree_hash" BLOB DEFAULT NULL,
    "latest_tree_size" BIGINT DEFAULT NULL,
    "backward_tree_hash" BLOB DEFAULT NULL,
    "backward_tree_size" BIGINT DEFAULT NULL,
    PRIMARY KEY ("log_id")
);

CREATE TABLE sth (
    "serial_id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    "tree_hash" BLOB UNIQUE NOT NULL,
    "log_id" BLOB NOT NULL REFERENCES ctlogs("log_id"),
    "sth_timestamp" BIGINT NOT NULL, -- ms
    "received_time" BIGINT NOT NULL, -- ms
    "tree_size" BIGINT NOT NULL,
    "signature" BLOB NOT NULL,
    PRIMARY KEY ("tree_hash")
);

CREATE UNIQUE INDEX "sth_log_timestamp" ON sth ("log_id", "sth_timestamp");
CREATE UNIQUE INDEX "sth_log_tree_size" ON sth ("log_id", "tree_size");

CREATE TABLE logviolations (
    "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    "log_id" BLOB NOT NULL REFERENCES ctlogs("log_id"),
    "basing_sth_tree_hash" BLOB NOT NULL REFERENCES sth("tree_hash"),
    "next_tree_hash" BLOB NOT NULL,
    "next_tree_size" BIGINT NOT NULL,
    "next_sth_timestamp" BIGINT NOT NULL,
    "received_time" BIGINT NOT NULL,
    "next_sth_signature" BLOB NOT NULL,
    "error_msg" TEXT NOT NULL
);
