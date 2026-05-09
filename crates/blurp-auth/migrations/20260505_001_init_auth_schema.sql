-- Create auth schema
CREATE SCHEMA IF NOT EXISTS auth;

-- Users table
CREATE TABLE auth.users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email VARCHAR(255) UNIQUE NOT NULL,
  username VARCHAR(100) UNIQUE,
  password_hash VARCHAR(255),
  name VARCHAR(255),
  phone VARCHAR(20),
  is_guest BOOLEAN DEFAULT false,
  auth_method VARCHAR(50) DEFAULT 'magic_link', -- 'magic_link', 'password', 'google', 'google_password'
  created_at TIMESTAMPTZ DEFAULT now(),
  updated_at TIMESTAMPTZ DEFAULT now()
);

-- Sessions table
CREATE TABLE auth.sessions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  token_hash VARCHAR(255) UNIQUE NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ DEFAULT now()
);

-- Magic links table
CREATE TABLE auth.magic_links (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email VARCHAR(255) NOT NULL UNIQUE,
  token_hash VARCHAR(255) UNIQUE NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  used_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT now()
);

-- OAuth providers table
CREATE TABLE auth.oauth_providers (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  provider VARCHAR(50) NOT NULL,
  provider_user_id VARCHAR(255) NOT NULL,
  provider_email VARCHAR(255),
  access_token TEXT,
  refresh_token TEXT,
  expires_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT now(),
  updated_at TIMESTAMPTZ DEFAULT now(),
  UNIQUE(provider, provider_user_id)
);

-- Password resets table
CREATE TABLE auth.password_resets (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  token_hash VARCHAR(255) UNIQUE NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  used_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes
CREATE INDEX idx_auth_users_email ON auth.users(email);
CREATE INDEX idx_auth_users_username ON auth.users(username);
CREATE INDEX idx_auth_sessions_user_id ON auth.sessions(user_id);
CREATE INDEX idx_auth_sessions_expires_at ON auth.sessions(expires_at);
CREATE INDEX idx_auth_magic_links_email ON auth.magic_links(email);
CREATE INDEX idx_auth_magic_links_expires_at ON auth.magic_links(expires_at);
CREATE INDEX idx_auth_oauth_providers_user_id ON auth.oauth_providers(user_id);
CREATE INDEX idx_auth_oauth_providers_provider ON auth.oauth_providers(provider);
CREATE INDEX idx_auth_password_resets_user_id ON auth.password_resets(user_id);
CREATE INDEX idx_auth_password_resets_expires_at ON auth.password_resets(expires_at);
