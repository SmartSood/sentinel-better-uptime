use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateWebsiteOutput {
    pub website_id: String,
}
#[derive(Deserialize, Serialize)]
pub struct CreateUserOutput {
    pub user_id: String,
}
#[derive(Deserialize, Serialize)]
pub struct SignInOutput {
    pub jwt: String,
}

#[derive(Deserialize, Serialize)]
pub struct GetAllWebsiteOutput {
    pub websites: Vec<WebsiteOutput>,
}

#[derive(Deserialize, Serialize)]
pub struct WebsiteOutput {
    pub id: String,
    pub url: String,
    pub user_id: String,
    pub time_added: String,
    pub region_ids: Vec<Option<String>>,
    pub poll_time: i64,
}

#[derive(Deserialize, Serialize)]
pub struct WebsiteTickOutput {
    pub id: String,
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
    pub created_at: String,
}

#[derive(Deserialize, Serialize)]
pub struct WebsiteTickHistoryOutput {
    pub ticks: Vec<WebsiteTickOutput>,
}
#[derive(Deserialize, Serialize)]
pub struct Region {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct RegionsOutput {
    pub regions: Vec<Region>,
}
