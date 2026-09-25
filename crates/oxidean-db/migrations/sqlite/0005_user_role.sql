-- Instance role replaces is_admin (user | admin | sys-admin).
ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'user';
UPDATE users SET role = 'sys-admin' WHERE is_admin != 0;
ALTER TABLE users DROP COLUMN is_admin;
