use chrono::Utc;
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const BATCH_SIZE: usize = 100;
pub const STREAM_NAME: &str = "uptime_stream";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteJob {
    pub website_id: String,
    pub url: String,
    pub poll_time: i64,
    pub region_id: String,
    pub cycle_id: String,
    pub total_regions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebsiteDetails {
    url: String,
    region_id: Vec<Option<String>>,
    poll_time: i64,
}

/// Dispatch one bounded batch. The caller owns the loop, so this function
/// never monopolizes a mutex or executor thread.
pub async fn scheduler_tick(connection: &mut MultiplexedConnection) -> redis::RedisResult<usize> {
    let website_ids: Vec<String> = redis::cmd("ZRANGEBYSCORE")
        .arg("PollingTimeSet")
        .arg("-inf")
        .arg(Utc::now().timestamp())
        .arg("LIMIT")
        .arg(0)
        .arg(BATCH_SIZE)
        .query_async(connection)
        .await?;
    if website_ids.is_empty() {
        return Ok(0);
    }

    let details: Vec<Option<String>> = redis::cmd("HMGET")
        .arg("WebsiteDetails")
        .arg(&website_ids)
        .query_async(connection)
        .await?;

    let mut jobs = Vec::new();
    let mut dispatched_websites = Vec::new();
    for (website_id, details_json) in website_ids.iter().zip(details.iter()) {
        let Some(details_json) = details_json else {
            eprintln!("Missing WebsiteDetails for {website_id}; leaving it scheduled");
            continue;
        };
        let website_details: WebsiteDetails = match serde_json::from_str(details_json) {
            Ok(details) => details,
            Err(error) => {
                eprintln!("Invalid WebsiteDetails for {website_id}: {error}; leaving it scheduled");
                continue;
            }
        };
        let mut regions: Vec<String> = website_details.region_id.into_iter().flatten().collect();
        regions.sort();
        regions.dedup();
        if regions.is_empty() {
            eprintln!("Website {website_id} has no regions; leaving it scheduled");
            continue;
        }
        let cycle_id = Uuid::new_v4().to_string();
        let total_regions = regions.len();
        for region_id in regions {
            jobs.push(WebsiteJob {
                website_id: website_id.clone(),
                url: website_details.url.clone(),
                poll_time: website_details.poll_time,
                region_id,
                cycle_id: cycle_id.clone(),
                total_regions,
            });
        }
        dispatched_websites.push(website_id);
    }
    if jobs.is_empty() {
        return Ok(0);
    }

    // One round trip: XADD all regional jobs, then remove their source IDs.
    let mut pipeline = redis::pipe();
    for job in &jobs {
        let serialized = serde_json::to_string(job).map_err(|error| {
            redis::RedisError::from((
                redis::ErrorKind::Client,
                "failed to serialize job",
                error.to_string(),
            ))
        })?;
        pipeline
            .cmd("XADD")
            .arg(STREAM_NAME)
            .arg("*")
            .arg("job")
            .arg(serialized)
            .ignore();
    }
    for website_id in dispatched_websites {
        pipeline
            .cmd("ZREM")
            .arg("PollingTimeSet")
            .arg(website_id)
            .ignore();
    }
    pipeline.query_async::<()>(connection).await?;
    Ok(jobs.len())
}
