//! Wave 0 / 09-02: `0009_ssh_keys` + ssh_public_keys (UNIQUE fingerprint).
//!
//! RED until tri-dialect migration lands. Mirror dialect_pats pattern.
//! Columns: id, user_id, title, public_key, fingerprint UNIQUE, key_type, last_used_at, created_at.

/// Expect sqlite `0009_ssh_keys.sql` with `ssh_public_keys` + UNIQUE fingerprint.
#[tokio::test]
async fn dialect_ssh_keys_migrate_0009_schema_presence() {
    let migration_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/sqlite/0009_ssh_keys.sql"
    );
    let sql = std::fs::read_to_string(migration_path).unwrap_or_default();
    assert!(
        !sql.is_empty(),
        "Wave 0: 0009_ssh_keys.sql must exist (ssh_public_keys + UNIQUE fingerprint)"
    );
    assert!(
        sql.contains("ssh_public_keys"),
        "Wave 0: 0009 must define ssh_public_keys"
    );
    for col in [
        "id",
        "user_id",
        "title",
        "public_key",
        "fingerprint",
        "key_type",
        "last_used_at",
        "created_at",
    ] {
        assert!(
            sql.contains(col),
            "Wave 0: 0009 ssh_public_keys must mention column {col}"
        );
    }
    assert!(
        sql.to_ascii_uppercase().contains("UNIQUE") || sql.contains("unique"),
        "Wave 0: 0009 fingerprint must be UNIQUE (D-SSH-05)"
    );

    assert!(
        false,
        "Wave 0: Database SSH key APIs + migrate 0009_ssh_keys not wired yet"
    );
}

/// Tri-dialect parity: postgres and mysql siblings must exist alongside sqlite.
#[test]
fn dialect_ssh_keys_tri_dialect_files() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations");
    for dialect in ["sqlite", "postgres", "mysql"] {
        let path = format!("{root}/{dialect}/0009_ssh_keys.sql");
        let sql = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            !sql.is_empty(),
            "Wave 0: missing {path} — tri-dialect 0009_ssh_keys required"
        );
        assert!(
            sql.contains("ssh_public_keys"),
            "Wave 0: {dialect} 0009_ssh_keys must mention ssh_public_keys"
        );
        assert!(
            sql.contains("fingerprint"),
            "Wave 0: {dialect} 0009_ssh_keys must mention fingerprint"
        );
    }
}
