-- Add admin flag to users
ALTER TABLE auth.users ADD COLUMN IF NOT EXISTS is_admin BOOLEAN DEFAULT FALSE;

-- Make first user admin for testing
UPDATE auth.users SET is_admin = TRUE WHERE username = 'stdtest2';
