-- logical: 0009_ssh_keys — registered SSH public keys (fingerprint UNIQUE; D-SSH-05)
-- MySQL: TEXT cannot carry DEFAULT — use VARCHAR/CHAR where a default is required.
CREATE TABLE IF NOT EXISTS ssh_public_keys (
  id             CHAR(36)     PRIMARY KEY,
  user_id        CHAR(36)     NOT NULL,
  title          VARCHAR(200) NOT NULL,
  public_key     TEXT         NOT NULL,
  fingerprint    VARCHAR(128) NOT NULL,
  key_type       VARCHAR(64)  NOT NULL,
  last_used_at   TIMESTAMP    NULL,
  last_used_ip   VARCHAR(64)  NULL,
  created_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_ssh_keys_fingerprint (fingerprint),
  CONSTRAINT fk_ssh_keys_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_ssh_keys_user_id ON ssh_public_keys(user_id);
