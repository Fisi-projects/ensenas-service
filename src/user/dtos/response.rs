use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub streak: i32,
    pub level: i32,
    pub experience: i32,
    pub last_experience_at: Option<DateTime<Utc>>,
    pub timezone: String,
}
