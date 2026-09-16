-- Your SQL goes here
-- Change the array type so elements INSIDE the array cannot be NULL
ALTER TABLE website ALTER COLUMN region_ids TYPE TEXT[] USING region_ids::TEXT[];
