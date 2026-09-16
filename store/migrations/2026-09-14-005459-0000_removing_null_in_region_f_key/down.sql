-- This file should undo anything in `up.sql`
-- Reverts the column to standard behavior if you roll back
ALTER TABLE website ALTER COLUMN region_ids TYPE TEXT[] USING region_ids::TEXT[];
