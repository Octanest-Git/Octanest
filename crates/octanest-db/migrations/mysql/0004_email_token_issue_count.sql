-- logical: 0004_email_token_issue_count — soft rate-limit issue counter (D-19)
ALTER TABLE auth_email_tokens ADD COLUMN issue_count INT NOT NULL DEFAULT 1;
