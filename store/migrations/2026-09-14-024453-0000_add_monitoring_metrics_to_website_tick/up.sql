-- Your SQL goes here
ALTER TABLE website_tick
ADD COLUMN status_code INTEGER,
ADD COLUMN dns_time_ms INTEGER,
ADD COLUMN tcp_time_ms INTEGER,
ADD COLUMN tls_time_ms INTEGER,
ADD COLUMN ttfb_ms INTEGER,
ADD COLUMN response_size_bytes INTEGER,
ADD COLUMN content_valid BOOLEAN,
ADD COLUMN ssl_valid BOOLEAN,
ADD COLUMN ssl_days_remaining INTEGER,
ADD COLUMN error TEXT;