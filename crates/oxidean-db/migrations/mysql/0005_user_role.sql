-- Instance role replaces is_admin (user | admin | sys-admin).
ALTER TABLE users ADD COLUMN role VARCHAR(32) NOT NULL DEFAULT 'user';
UPDATE users SET role = 'sys-admin' WHERE is_admin = TRUE;
ALTER TABLE users DROP COLUMN is_admin;
-- MySQL 8.0.16+ CHECK; ignored on older engines without error for ADD CONSTRAINT name.
ALTER TABLE users ADD CONSTRAINT users_role_check CHECK (role IN ('user', 'admin', 'sys-admin'));
