use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserDto {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub username: String,
    pub password: String,
}
