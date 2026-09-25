-- logical: 0006_bootstrap_flags — allow_signup + must_change_credentials
ALTER TABLE instance_auth_settings ADD COLUMN allow_signup BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN must_change_credentials BOOLEAN NOT NULL DEFAULT FALSE;
