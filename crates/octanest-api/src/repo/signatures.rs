//! Build git `allowedSignersFile` for SSH commit verification on commit pages.

use std::path::PathBuf;

use octanest_db::Database;

use super::author_resolve;

/// Collect allowed_signers for the given author emails + web-flow key.
///
/// Returns a temp dir (kept alive) and the path to the `allowed_signers` file inside it.
pub async fn allowed_signers_for_emails(
    db: &Database,
    emails: &[String],
) -> Option<(tempfile::TempDir, PathBuf)> {
    let resolved = author_resolve::resolve_authors_for_emails(db, emails).await;
    let named = author_resolve::build_allowed_signers_file(db, emails, &resolved).await?;

    let dir = tempfile::TempDir::new().ok()?;
    let path = dir.path().join("allowed_signers");
    tokio::fs::copy(named.path(), &path).await.ok()?;
    Some((dir, path))
}
