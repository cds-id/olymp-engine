-- Allow multiple stages per tier (e.g. multiple district locations running in parallel)
-- Drop unique constraint on (event_id, tier) so multiple district stages can exist
ALTER TABLE stages DROP CONSTRAINT IF EXISTS stages_event_id_tier_key;

-- Add location/capacity fields for stage instances
ALTER TABLE stages ADD COLUMN name TEXT;
ALTER TABLE stages ADD COLUMN location TEXT;
ALTER TABLE stages ADD COLUMN district_id UUID;
ALTER TABLE stages ADD COLUMN province_id UUID;
ALTER TABLE stages ADD COLUMN capacity INT;

-- Prevent exact duplicates
CREATE UNIQUE INDEX idx_stages_event_tier_name ON stages(event_id, tier, COALESCE(name, ''));
