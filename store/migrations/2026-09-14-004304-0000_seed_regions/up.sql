-- Your SQL goes here
INSERT INTO region (id, name) VALUES
    ('us-east', 'US East (N. Virginia)'),
    ('us-west', 'US West (Oregon)'),
    ('eu-central', 'Europe (Frankfurt)'),
    ('ap-south', 'Asia Pacific (Mumbai)'),
    ('ap-northeast', 'Asia Pacific (Tokyo)'),
    ('sa-east', 'South America (São Paulo)')
ON CONFLICT (id) DO NOTHING; 