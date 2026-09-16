use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateWebsiteInput {
    pub url: String,
    pub region_ids: Vec<Option<String>>,
    pub poll_time: i64,
}
#[derive(Deserialize, Serialize)]
pub struct CreateUserInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebsiteTickInput {
    pub website_id: String,
    pub region_id: String,

    pub status: String,
    pub status_code: Option<i32>,

    pub response_time_ms: i32,

    pub dns_time_ms: Option<i32>,
    pub tcp_time_ms: Option<i32>,
    pub tls_time_ms: Option<i32>,
    pub ttfb_ms: Option<i32>,

    pub response_size_bytes: Option<i64>,

    pub content_valid: Option<bool>,

    pub ssl_valid: Option<bool>,
    pub ssl_days_remaining: Option<i32>,

    pub error: Option<String>,
}
