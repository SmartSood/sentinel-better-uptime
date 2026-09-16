-- This file should undo anything in `up.sql`
DELETE FROM region WHERE id IN (
    'us-east', 
    'us-west', 
    'eu-central', 
    'ap-south', 
    'ap-northeast', 
    'sa-east'
);
