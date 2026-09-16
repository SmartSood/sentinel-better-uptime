use crate::members::scheduler::{STREAM_NAME, WebsiteJob};
use redis::aio::MultiplexedConnection;
use reqwest::Client;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use store::store::Store;
use tokio::sync::Semaphore;

const GROUP_NAME: &str = "uptime_workers";

pub async fn run_reader(
    mut connection: MultiplexedConnection,
    store: Arc<Mutex<Store>>,
    worker_name: String,
    semaphore: Arc<Semaphore>,
    client: Client,
) -> redis::RedisResult<()> {
    loop {
        let messages: redis::streams::StreamReadReply = match redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(GROUP_NAME)
            .arg(&worker_name)
            .arg("COUNT")
            .arg(10)
            .arg("BLOCK")
            .arg(5000)
            .arg("STREAMS")
            .arg(STREAM_NAME)
            .arg(">")
            .query_async(&mut connection)
            .await
        {
            Ok(messages) => messages,
            Err(error) => {
                eprintln!("[{worker_name}] Redis read failed: {error}; retrying");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        };
        for stream in messages.keys {
            for message in stream.ids {
                let message_id = message.id.clone();
                let job_json = match message
                    .map
                    .get("job")
                    .map(|value| redis::from_redis_value::<String>(value.clone()))
                {
                    Some(Ok(job_json)) => job_json,
                    Some(Err(error)) => {
                        eprintln!("[{worker_name}] invalid job field in {message_id}: {error}");
                        ack(&mut connection, &message_id).await?;
                        continue;
                    }
                    None => {
                        eprintln!("[{worker_name}] message {message_id} has no job field");
                        ack(&mut connection, &message_id).await?;
                        continue;
                    }
                };
                let job: WebsiteJob = match serde_json::from_str(&job_json) {
                    Ok(job) => job,
                    Err(error) => {
                        eprintln!("[{worker_name}] invalid job JSON in {message_id}: {error}");
                        ack(&mut connection, &message_id).await?;
                        continue;
                    }
                };
                let mut task_connection = connection.clone();
                let task_store = Arc::clone(&store);
                let task_semaphore = Arc::clone(&semaphore);
                let task_client = client.clone();
                let task_worker_name = worker_name.clone();
                tokio::spawn(async move {
                    let permit = match task_semaphore.acquire().await {
                        Ok(permit) => permit,
                        Err(_) => return,
                    };
                    let result = check_website(&task_client, &job).await;
                    {
                        let mut store = match task_store.lock() {
                            Ok(store) => store,
                            Err(_) => {
                                eprintln!("[{task_worker_name}] database mutex poisoned");
                                return;
                            }
                        };
                        if let Err(error) = store.create_website_tick(
                            job.website_id.clone(),
                            job.region_id.clone(),
                            result.status,
                            result.status_code,
                            result.response_time_ms,
                            None,
                            None,
                            None,
                            result.ttfb_ms,
                            result.response_size_bytes,
                            None,
                            result.ssl_valid,
                            None,
                            result.error,
                        ) {
                            eprintln!("[{task_worker_name}] DB error: {error}");
                            return;
                        }
                    }
                    drop(permit);
                    if let Err(error) =
                        complete_cycle_and_reschedule(&mut task_connection, &job).await
                    {
                        eprintln!("[{task_worker_name}] cycle tracking error: {error}");
                        return;
                    }
                    if let Err(error) = ack(&mut task_connection, &message_id).await {
                        eprintln!("[{task_worker_name}] XACK failed: {error}");
                    }
                });
            }
        }
    }
}

struct CheckResult {
    status: String,
    status_code: Option<i32>,
    /// Full request time, including downloading the response body.
    response_time_ms: i32,
    /// Time until response headers arrive (the closest reqwest can provide to
    /// time-to-first-byte).
    ttfb_ms: Option<i32>,
    /// Exact body size, including chunked responses without Content-Length.
    response_size_bytes: Option<i64>,
    /// Reqwest's default rustls client rejects invalid HTTPS certificates.
    ssl_valid: Option<bool>,
    error: Option<String>,
}

async fn check_website(client: &Client, job: &WebsiteJob) -> CheckResult {
    let started = Instant::now();
    match client.get(&job.url).send().await {
        Ok(response) => {
            let ttfb_ms = elapsed_ms(started);
            let status_code = response.status().as_u16() as i32;
            let is_success = response.status().is_success();
            let ssl_valid = (response.url().scheme() == "https").then_some(true);

            match response.bytes().await {
                Ok(body) => CheckResult {
                    status: if is_success { "UP" } else { "DOWN" }.to_owned(),
                    status_code: Some(status_code),
                    response_time_ms: elapsed_ms(started),
                    ttfb_ms: Some(ttfb_ms),
                    response_size_bytes: Some(body.len().min(i64::MAX as usize) as i64),
                    ssl_valid,
                    error: None,
                },
                Err(error) => CheckResult {
                    status: "DOWN".to_owned(),
                    status_code: Some(status_code),
                    response_time_ms: elapsed_ms(started),
                    ttfb_ms: Some(ttfb_ms),
                    response_size_bytes: None,
                    ssl_valid,
                    error: Some(error.to_string()),
                },
            }
        }
        Err(error) => CheckResult {
            status: "DOWN".to_owned(),
            status_code: None,
            response_time_ms: elapsed_ms(started),
            ttfb_ms: None,
            response_size_bytes: None,
            ssl_valid: None,
            error: Some(error.to_string()),
        },
    }
}

fn elapsed_ms(started: Instant) -> i32 {
    started.elapsed().as_millis().min(i32::MAX as u128) as i32
}

async fn complete_cycle_and_reschedule(
    connection: &mut MultiplexedConnection,
    job: &WebsiteJob,
) -> redis::RedisResult<()> {
    const SCRIPT: &str = r#"
local cycle_key = KEYS[1]
local schedule_key = KEYS[2]
if redis.call('SADD', cycle_key, ARGV[2]) == 1 then
  redis.call('EXPIRE', cycle_key, tonumber(ARGV[5]))
  if redis.call('SCARD', cycle_key) >= tonumber(ARGV[3]) then
    redis.call('ZADD', schedule_key, ARGV[4], ARGV[1])
    redis.call('DEL', cycle_key)
  end
end
return 0
"#;
    let cycle_key = format!("Cycle:{}:completed_regions", job.cycle_id);
    let next_check_at = chrono::Utc::now()
        .timestamp()
        .saturating_add(job.poll_time.max(0));
    let ttl = job.poll_time.max(0).saturating_add(300).max(300);
    redis::cmd("EVAL")
        .arg(SCRIPT)
        .arg(2)
        .arg(cycle_key)
        .arg("PollingTimeSet")
        .arg(&job.website_id)
        .arg(&job.region_id)
        .arg(job.total_regions)
        .arg(next_check_at)
        .arg(ttl)
        .query_async::<i64>(connection)
        .await?;
    Ok(())
}

async fn ack(connection: &mut MultiplexedConnection, message_id: &str) -> redis::RedisResult<()> {
    redis::cmd("XACK")
        .arg(STREAM_NAME)
        .arg(GROUP_NAME)
        .arg(message_id)
        .query_async::<i64>(connection)
        .await?;
    Ok(())
}
