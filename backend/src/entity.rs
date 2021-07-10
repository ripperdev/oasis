use serde::Serialize;
use sqlx::FromRow;
use sqlx::{Pool, Sqlite};
use std::sync::Mutex;

pub struct AppState {
    pub first_run: Mutex<bool>,
    pub pool: Mutex<Pool<Sqlite>>,
}

pub struct Auth {
    pub option: i8,
}

#[derive(Serialize, FromRow, Debug)]
pub struct Site {
    pub version: f64,
    pub first_run: u8,
    pub created_at: String,
    pub storage: String,
}
