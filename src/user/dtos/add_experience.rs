use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddExperiencePayload {
    pub gained_exp: i32,
}
