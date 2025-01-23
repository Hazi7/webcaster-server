use sqlx::MySql;

use crate::models::user::CreateUserDto;

pub struct UserRepository {
    pool: MySql,
}

impl UserRepository {
    pub fn new(pool: MySql) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, user: CreateUserDto) {}
}
