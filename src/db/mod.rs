use derive_more::derive::Display;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use std::error::Error;

pub mod mission_log;
pub mod models;
pub mod schema;

pub mod mission;

#[derive(Display, Debug)]
pub enum DbError {
    UnexpectedError(String),
}

impl Error for DbError {}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
