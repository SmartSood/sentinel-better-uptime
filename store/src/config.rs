
use std::env::{self};


pub struct Config{
    pub database_url: String,
    pub jwt_secret: String,
    pub redis_url: String
}

impl Default for Config{
    fn default() -> Self {
        Self { database_url: env::var("DATABASE_URL").unwrap_or_else(|_| panic!("DATABASE_URL must be set")) 
    , jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| panic!("JWT_SECRET must be set")),
    redis_url: env::var("REDIS_URL").unwrap_or_else(|_| panic!("REDIS_URL must be set"))}
    }
}