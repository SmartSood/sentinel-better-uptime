use poem::{EndpointExt, Route, Server, get, middleware::Cors, post};
pub mod auth_middleware;
pub mod config;
pub mod request_inputs;
pub mod request_outputs;
pub mod routes;
use dotenvy::dotenv;
use redis_queue::connection::RedisConfig;
use redis_queue::members::{reader::run_reader, scheduler::scheduler_tick};
use std::sync::{Arc, Mutex};
use store::store::Store;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    let mut store = Store::default().unwrap();
    let regions = store.get_all_regions().unwrap();
    let region_values: Vec<(String, String)> = regions
        .into_iter()
        .map(|region| (region.id, region.name))
        .collect();

    let s = Arc::new(Mutex::new(store));
    let redis_config = RedisConfig::default(&region_values)
        .await
        .expect("failed to initialize Redis");
    let scheduler_connection = redis_config
        .worker_connection()
        .await
        .expect("failed to connect scheduler to Redis");
    let mut worker_connections = Vec::with_capacity(3);
    for _ in 0..3 {
        worker_connections.push(
            redis_config
                .worker_connection()
                .await
                .expect("failed to connect reader to Redis"),
        );
    }
    let redis_conn = Arc::new(Mutex::new(redis_config));

    // Scheduler and readers have independent multiplexed connections. API
    // calls retain the synchronous connection only for ZADD/HSET writes.
    tokio::spawn(async move {
        let mut connection = scheduler_connection;
        loop {
            if let Err(error) = scheduler_tick(&mut connection).await {
                eprintln!("scheduler error: {error}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    let semaphore = Arc::new(tokio::sync::Semaphore::new(32));
    let http_client = reqwest::Client::new();
    for (index, connection) in worker_connections.into_iter().enumerate() {
        let worker_store = Arc::clone(&s);
        let worker_semaphore = Arc::clone(&semaphore);
        let worker_client = http_client.clone();
        tokio::spawn(async move {
            if let Err(error) = run_reader(
                connection,
                worker_store,
                format!("worker_{}", index + 1),
                worker_semaphore,
                worker_client,
            )
            .await
            {
                eprintln!("reader {} stopped: {error}", index + 1);
            }
        });
    }

    let app = Route::new()
        .at(
            "/website/:id/ticks",
            get(routes::website::get_website_ticks),
        )
        .at("/website/:id", get(routes::website::get_website))
        .at("/website", post(routes::website::create_website))
        .at("/user/sign-in", post(routes::user::sign_in))
        .at("/user/sign-up", post(routes::user::sign_up))
        .at("/all_websites", get(routes::website::get_all_websites))
        .at("/regions", get(routes::regions::get_all_regions))
        .data(s)
        .data(redis_conn)
        .with(
            Cors::new()
                .allow_origin("https://sentinel.smarthsood.com")
                .allow_methods(["GET", "POST", "OPTIONS"])
                .allow_headers(["Authorization", "Content-Type"]),
        );

    Server::new(poem::listener::TcpListener::bind("127.0.0.1:3001"))
        .name("hello-world")
        .run(app)
        .await
}
