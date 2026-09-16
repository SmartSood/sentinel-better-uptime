use redis::AsyncConnectionConfig;
use redis::aio::MultiplexedConnection;
use redis::{Client, Connection};

pub struct RedisConfig {
    client: Client,
    pub redis_connection: Connection,
    pub async_redis_connection: MultiplexedConnection,
}

impl RedisConfig {
    /// Each background worker gets a distinct TCP connection. Cloning a
    /// multiplexed connection shares one socket, which is unsuitable for a
    /// blocking XREADGROUP command.
    pub async fn worker_connection(&self) -> redis::RedisResult<MultiplexedConnection> {
        let config = AsyncConnectionConfig::new().set_response_timeout(None);
        self.client
            .get_multiplexed_async_connection_with_config(&config)
            .await
    }
}

impl RedisConfig {
    pub async fn default(regions: &[(String, String)]) -> redis::RedisResult<Self> {
        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

        let client = Client::open(redis_url)?;

        // Synchronous connection
        let redis_connection = client.get_connection()?;

        // Async connection
        let async_config = AsyncConnectionConfig::new().set_response_timeout(None);
        let async_redis_connection = client
            .get_multiplexed_async_connection_with_config(&async_config)
            .await?;

        let mut config = Self {
            client,
            redis_connection,
            async_redis_connection,
        };

        // Cache region information
        config.cache_regions(regions)?;

        // Create stream consumer group
        config.create_consumer_group().await?;

        Ok(config)
    }

    pub fn cache_regions(&mut self, regions: &[(String, String)]) -> redis::RedisResult<()> {
        for (region_id, region_name) in regions {
            redis::cmd("HSET")
                .arg("Regions")
                .arg(region_id)
                .arg(region_name)
                .query::<()>(&mut self.redis_connection)?;
        }

        Ok(())
    }

    pub async fn create_consumer_group(&mut self) -> redis::RedisResult<()> {
        let result: redis::RedisResult<()> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg("uptime_stream")
            .arg("uptime_workers")
            .arg("0")
            .arg("MKSTREAM")
            .query_async(&mut self.async_redis_connection)
            .await;

        match result {
            Ok(_) => Ok(()),

            Err(e) => {
                if e.to_string().contains("BUSYGROUP") {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }
}
