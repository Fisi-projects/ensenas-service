use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FirebaseClaims {
    pub uid: String,
    pub email: String,
    pub name: String,
}
