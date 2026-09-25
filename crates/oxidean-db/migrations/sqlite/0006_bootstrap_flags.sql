-- logical: 0006_bootstrap_flags — allow_signup + must_change_credentials
ALTER TABLE instance_auth_settings ADD COLUMN allow_signup INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN must_change_credentials INTEGER NOT NULL DEFAULT 0;
