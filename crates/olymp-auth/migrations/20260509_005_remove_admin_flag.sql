-- Remove is_admin flag, replaced by RBAC system (olymp-rbac)
ALTER TABLE auth.users DROP COLUMN IF EXISTS is_admin;
