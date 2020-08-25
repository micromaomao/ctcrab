table! {
    cert_fetch_errors (id) {
        id -> Int8,
        log_id -> Bytea,
        from_tree_size -> Int8,
        to_tree_size -> Int8,
        error_time -> Timestamptz,
        resolved -> Nullable<Bool>,
    }
}

table! {
    consistency_check_errors (id) {
        id -> Int8,
        log_id -> Bytea,
        from_sth_id -> Int8,
        to_sth_id -> Int8,
        discovery_time -> Timestamptz,
        last_check_time -> Timestamptz,
        last_check_error -> Text,
    }
}

table! {
    ctlogs (log_id) {
        log_id -> Bytea,
        endpoint_url -> Text,
        name -> Text,
        public_key -> Bytea,
        monitoring -> Bool,
        latest_sth -> Nullable<Int8>,
        last_sth_error -> Nullable<Text>,
    }
}

table! {
    retired_log_changed_error (log_id) {
        log_id -> Bytea,
        latest_sth -> Int8,
    }
}

table! {
    sth (id) {
        id -> Int8,
        log_id -> Bytea,
        tree_hash -> Bytea,
        tree_size -> Int8,
        sth_timestamp -> Int8,
        received_time -> Timestamptz,
        signature -> Bytea,
        checked_consistent_with_latest -> Bool,
    }
}

joinable!(cert_fetch_errors -> ctlogs (log_id));
joinable!(consistency_check_errors -> ctlogs (log_id));
joinable!(retired_log_changed_error -> ctlogs (log_id));
joinable!(retired_log_changed_error -> sth (latest_sth));

allow_tables_to_appear_in_same_query!(
    cert_fetch_errors,
    consistency_check_errors,
    ctlogs,
    retired_log_changed_error,
    sth,
);
