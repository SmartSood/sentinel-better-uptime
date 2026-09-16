use crate::auth_middleware::UserId;
use crate::request_inputs::{CreateWebsiteInput, CreateWebsiteTickInput};
use crate::request_outputs::{
    CreateWebsiteOutput, GetAllWebsiteOutput, WebsiteOutput, WebsiteTickHistoryOutput,
    WebsiteTickOutput,
};
use poem::http::StatusCode;
use poem::{
    Error, handler,
    web::{Data, Json, Path, Query},
};
use redis_queue::connection::RedisConfig;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use store::{models::website, store::Store};

#[handler]
pub async fn get_website(
    Path(id): Path<String>,
    Data(s): Data<&Arc<Mutex<Store>>>,
    user_id: UserId,
) -> Result<Json<WebsiteOutput>, Error> {
    let mut locked_s = s
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
    let websites = locked_s
        .get_website_by_id(id, user_id.0)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
    let response = WebsiteOutput {
        id: websites.id,
        url: websites.url,
        user_id: websites.user_id,
        time_added: websites.time_added.to_string(),
        region_ids: websites.region_ids,
        poll_time: websites.poll_time,
    };
    Ok(Json(response))
}

#[handler]
pub async fn get_all_websites(
    Data(s): Data<&Arc<Mutex<Store>>>,
    user_id: UserId,
) -> Result<Json<GetAllWebsiteOutput>, Error> {
    let mut locked_s = s
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
    let websites = locked_s
        .get_websites(user_id.0)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
    let response = GetAllWebsiteOutput {
        websites: websites
            .into_iter()
            .map(|website| WebsiteOutput {
                id: website.id,
                url: website.url,
                user_id: website.user_id,
                time_added: website.time_added.to_string(),
                region_ids: website.region_ids,
                poll_time: website.poll_time,
            })
            .collect(),
    };
    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct TickHistoryQuery {
    pub limit: Option<i64>,
}

#[handler]
pub async fn get_website_ticks(
    Path(id): Path<String>,
    Query(query): Query<TickHistoryQuery>,
    Data(s): Data<&Arc<Mutex<Store>>>,
    user_id: UserId,
) -> Result<Json<WebsiteTickHistoryOutput>, Error> {
    let limit = query.limit.unwrap_or(200).clamp(1, 1_000);
    let mut locked_store = s
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
    let ticks = locked_store
        .get_website_ticks(id, user_id.0, limit)
        .map_err(|_| Error::from_status(StatusCode::NOT_FOUND))?;

    Ok(Json(WebsiteTickHistoryOutput {
        ticks: ticks
            .into_iter()
            .map(|tick| WebsiteTickOutput {
                id: tick.id,
                website_id: tick.website_id,
                region_id: tick.region_id,
                status: tick.status,
                status_code: tick.status_code,
                response_time_ms: tick.response_time_ms,
                dns_time_ms: tick.dns_time_ms,
                tcp_time_ms: tick.tcp_time_ms,
                tls_time_ms: tick.tls_time_ms,
                ttfb_ms: tick.ttfb_ms,
                response_size_bytes: tick.response_size_bytes,
                content_valid: tick.content_valid,
                ssl_valid: tick.ssl_valid,
                ssl_days_remaining: tick.ssl_days_remaining,
                error: tick.error,
                created_at: tick.created_at.to_string(),
            })
            .collect(),
    }))
}

// #[handler]
// pub async fn create_website(
//     Json(data): Json<CreateWebsiteInput>,
//     Data(s): Data<&Arc<Mutex<Store>>>,
//     Data(redis): Data<&Arc<Mutex<RedisConfig>>>,
//     user_id: UserId,
// ) -> Result<Json<CreateWebsiteOutput>, Error> {
//     let mut locked_s = s
//         .lock()
//         .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

//     let mut locked_redis = redis
//         .lock()
//         .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

//     let website = locked_s
//         .create_website(user_id.0, data.url, data.region_ids, data.poll_time)
//         .map_err(|_| Error::from_status(StatusCode::CONFLICT))?;
//     let next_check_at = website
//     .time_added
//     .and_utc()
//     .timestamp()
//     + data.poll_time;
//     let _adding_to_redis = locked_redis.push_to_set(
//         website.id.clone(),
//         next_check_at,
//         website.url.clone(),
//         website.region_ids.clone(),
//         website.poll_time,

//     )
//     .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

//     Ok(Json(CreateWebsiteOutput {
//         website_id: website.id,
//     }))
// }

#[handler]
pub async fn create_website(
    Json(data): Json<CreateWebsiteInput>,

    Data(s): Data<&Arc<Mutex<Store>>>,

    Data(redis): Data<&Arc<Mutex<RedisConfig>>>,

    user_id: UserId,
) -> Result<Json<CreateWebsiteOutput>, Error> {
    let mut locked_s = s
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    let mut locked_redis = redis
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    // ==========================================
    // PostgreSQL
    // ==========================================

    let website = locked_s
        .create_website(user_id.0, data.url, data.region_ids, data.poll_time)
        .map_err(|_| Error::from_status(StatusCode::CONFLICT))?;

    // ==========================================
    // First execution
    // ==========================================

    let next_check_at = website.time_added.and_utc().timestamp() + website.poll_time;

    // ==========================================
    // Redis
    // ==========================================

    locked_redis
        .push_to_set(
            website.id.clone(),
            next_check_at,
            website.url.clone(),
            website.region_ids.clone(),
            website.poll_time,
        )
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(CreateWebsiteOutput {
        website_id: website.id,
    }))
}

#[handler]
pub async fn create_website_tick(
    Json(data): Json<CreateWebsiteTickInput>,
    Data(s): Data<&Arc<Mutex<Store>>>,
) -> Result<StatusCode, Error> {
    let mut locked_s = s
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    locked_s
        .create_website_tick(
            data.website_id,
            data.region_id,
            data.status,
            data.status_code,
            data.response_time_ms,
            data.dns_time_ms,
            data.tcp_time_ms,
            data.tls_time_ms,
            data.ttfb_ms,
            data.response_size_bytes,
            data.content_valid,
            data.ssl_valid,
            data.ssl_days_remaining,
            data.error,
        )
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(StatusCode::CREATED)
}
