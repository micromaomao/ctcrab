CREATE TABLE ctlogs (
    "log_id" BLOB UNIQUE NOT NULL,
    "endpoint_url" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "public_key" BLOB NOT NULL,
    PRIMARY KEY ("log_id")
);

-- Use bigint instead of integer to make diesel generate correct schema.

CREATE TABLE sth (
    "tree_hash" BLOB UNIQUE NOT NULL,
    "log_id" BLOB NOT NULL REFERENCES ctlogs("log_id"),
    "sth_timestamp" BIGINT NOT NULL,
    "tree_size" BIGINT NOT NULL,
    "signature" BLOB NOT NULL,
    PRIMARY KEY ("tree_hash")
);

CREATE UNIQUE INDEX "sth_log_timestamp" ON sth ("log_id", "sth_timestamp");
CREATE UNIQUE INDEX "sth_log_tree_size" ON sth ("log_id", "tree_size");

