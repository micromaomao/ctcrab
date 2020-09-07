table! {
    cert_fetch_errors (id) {
        id -> Int8,
        log_id -> Bytea,
        from_tree_size -> Int8,
        to_tree_size -> Int8,
        error_time -> Timestamptz,
        error_msg -> Text,
    }
}

table! {
    certificate_appears_in_leaf (__pk) {
        __pk -> Int8,
        leaf_hash -> Bytea,
        cert_fp -> Bytea,
        log_id -> Bytea,
        leaf_index -> Int8,
    }
}

table! {
    certificate_chain (__pk) {
        __pk -> Int8,
        certificate_fingerprint -> Bytea,
        chain -> Array<Bytea>,
    }
}

table! {
    certificate_dns_names (__pk) {
        __pk -> Int8,
        cert_fp -> Bytea,
        dns_name -> Text,
    }
}

table! {
    certificates (fingerprint) {
        fingerprint -> Bytea,
        x509 -> Bytea,
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
joinable!(certificate_appears_in_leaf -> certificates (cert_fp));
joinable!(certificate_appears_in_leaf -> ctlogs (log_id));
joinable!(certificate_chain -> certificates (certificate_fingerprint));
joinable!(certificate_dns_names -> certificates (cert_fp));
joinable!(consistency_check_errors -> ctlogs (log_id));
joinable!(retired_log_changed_error -> ctlogs (log_id));
joinable!(retired_log_changed_error -> sth (latest_sth));

allow_tables_to_appear_in_same_query!(
    cert_fetch_errors,
    certificate_appears_in_leaf,
    certificate_chain,
    certificate_dns_names,
    certificates,
    consistency_check_errors,
    ctlogs,
    retired_log_changed_error,
    sth,
);
