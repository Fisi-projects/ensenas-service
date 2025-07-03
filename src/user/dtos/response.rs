use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub streak: i32,
    pub level: i32,
}
