-- Add registration window fields to stages (separate from exam phase)
ALTER TABLE stages ADD COLUMN registration_opens_at TIMESTAMPTZ;
ALTER TABLE stages ADD COLUMN registration_closes_at TIMESTAMPTZ;
