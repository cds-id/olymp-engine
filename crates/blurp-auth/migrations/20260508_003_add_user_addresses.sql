-- User address book
CREATE TABLE auth.user_addresses (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  label VARCHAR(50),
  name VARCHAR(255) NOT NULL,
  phone VARCHAR(20) NOT NULL,
  street TEXT NOT NULL,
  city VARCHAR(255) NOT NULL,
  province VARCHAR(255) NOT NULL,
  postal_code VARCHAR(10) NOT NULL,
  country VARCHAR(100) NOT NULL DEFAULT 'Indonesia',
  district_id INT NOT NULL,
  is_default BOOLEAN DEFAULT false,
  created_at TIMESTAMPTZ DEFAULT now(),
  updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_user_addresses_user_id ON auth.user_addresses(user_id);

-- Enforce at most one default address per user via partial unique index
CREATE UNIQUE INDEX idx_user_addresses_one_default
  ON auth.user_addresses(user_id)
  WHERE is_default = true;
