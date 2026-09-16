-- Your SQL goes here
ALTER TABLE website ADD COLUMN region_ids TEXT[] NOT NULL DEFAULT '{}';
