use crate::connection::RedisConfig;

use redis::Commands;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashSetWebDetails {
    pub url: String,

    pub region_id: Vec<Option<String>>,

    pub poll_time: i64,
}

impl RedisConfig {
    pub fn push_to_set(
        &mut self,

        input_id: String,

        next_check_at: i64,

        input_url: String,

        input_region_ids: Vec<Option<String>>,

        poll_time: i64,
    ) -> redis::RedisResult<()> {
        // ==========================================
        // ZSET
        //
        // website_id -> next timestamp
        // ==========================================

        let _: usize = self
            .redis_connection
            .zadd("PollingTimeSet", &input_id, next_check_at)?;

        // ==========================================
        // Website details
        // ==========================================

        let website_details = HashSetWebDetails {
            url: input_url,

            region_id: input_region_ids,

            poll_time,
        };

        let serialized = serde_json::to_string(&website_details).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::Client,
                "Failed to serialize website details",
                e.to_string(),
            ))
        })?;

        // ==========================================
        // HSET
        //
        // website_id -> JSON
        // ==========================================

        let _: usize = self
            .redis_connection
            .hset("WebsiteDetails", &input_id, serialized)?;

        Ok(())
    }
}
