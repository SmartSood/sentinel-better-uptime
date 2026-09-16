-- This file should undo anything in `up.sql`
ALTER TABLE website_tick
DROP COLUMN status_code,
DROP COLUMN dns_time_ms,
DROP COLUMN tcp_time_ms,
DROP COLUMN tls_time_ms,
DROP COLUMN ttfb_ms,
DROP COLUMN response_size_bytes,
DROP COLUMN content_valid,
DROP COLUMN ssl_valid,
DROP COLUMN ssl_days_remaining,
DROP COLUMN error;