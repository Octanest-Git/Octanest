//! ORG-01 Wave 0 stubs: organization email invites (create/list/revoke/accept).
//!
//! RED until invite RPC + hash-at-rest land. Do not implement handlers here.
//! D-ORG-03: email invite works when allow_signup is false (creates/links account).

/// `invites.create` issues an email invite for a role (ORG-01 / D-ORG-03).
#[tokio::test]
async fn org_invites_create() {
    assert!(
        false,
        "Wave 0: invites.create must enqueue invite for email + role (ORG-01 / D-ORG-03)"
    );
}

/// `invites.list` returns pending invites without plaintext tokens (ORG-01 / T-10-SC).
#[tokio::test]
async fn org_invites_list_omits_plaintext_token() {
    assert!(
        false,
        "Wave 0: invites.list must omit plaintext token; hash-at-rest only (ORG-01)"
    );
}

/// `invites.revoke` removes a pending invite (ORG-01).
#[tokio::test]
async fn org_invites_revoke() {
    assert!(
        false,
        "Wave 0: invites.revoke must drop pending invite from list (ORG-01)"
    );
}

/// `invites.accept` creates or links account when allow_signup is false (ORG-01 / D-ORG-03).
#[tokio::test]
async fn org_invites_accept_closed_signup_creates_or_links_account() {
    assert!(
        false,
        "Wave 0: invites.accept with allow_signup=false must create/link account (ORG-01 / D-ORG-03)"
    );
}

/// Invite tokens stored as hash-at-rest only (ORG-01).
#[tokio::test]
async fn org_invites_token_hash_at_rest() {
    assert!(
        false,
        "Wave 0: organization_invites.token_hash only — no plaintext token column (ORG-01)"
    );
}
