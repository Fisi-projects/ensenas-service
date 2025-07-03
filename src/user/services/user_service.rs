use crate::user::dtos::UserResponse;
use anyhow::{Context, Result};
use entity::user::{self, Entity};
use firebase_auth::FirebaseUser;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr, EntityTrait, Set, Value};

pub struct UserService;

impl UserService {
    pub async fn get_user(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Option<UserResponse>, DbErr> {
        let user = Entity::find_by_id(user_id).one(db).await?;

        match user {
            Some(user) => Ok(Some(UserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                level: user.level,
                streak: user.streak,
            })),
            None => Ok(None),
        }
    }

    pub async fn get_all_users(db: &DatabaseConnection) -> Result<Vec<UserResponse>, DbErr> {
        let results = Entity::find().all(db).await?;
        let users = results
            .into_iter()
            .map(|user| UserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                level: user.level,
                streak: user.streak,
            })
            .collect();

        Ok(users)
    }

    pub async fn create_user_from_token(
        db: &DatabaseConnection,
        user: &FirebaseUser,
    ) -> Result<user::Model> {
        let new_user = user::ActiveModel {
            id: Set(user.user_id.clone()),
            name: Set(user.name.clone().unwrap_or_default()),
            email: Set(user.email.clone().unwrap_or_default()),
            level: Set(1),
            streak: Set(0),
            experience: Set(0),
        };

        let inserted = new_user
            .insert(db)
            .await
            .context("Failed to insert user into database")?;

        Ok(inserted)
    }

    pub async fn delete_user(db: &DatabaseConnection, user_id: &str) -> Result<(), DbErr> {
        Entity::delete_by_id(user_id).exec(db).await?;
        Ok(())
    }

    pub async fn add_experience(
        db: &DatabaseConnection,
        user_id: &str,
        gained_exp: i32,
    ) -> Result<user::Model, DbErr> {
        let user = user::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom(format!("User not found: {user_id}")))?;

        let mut active_user: user::ActiveModel = user.into();

        let current_level = Self::unwrap_active_value(&active_user.level, 1);
        let current_exp = Self::unwrap_active_value(&active_user.experience, 0);

        let (new_level, new_exp) = Self::apply_experience(current_level, current_exp, gained_exp);

        active_user.level = Set(new_level);
        active_user.experience = Set(new_exp);

        active_user.update(db).await
    }

    fn unwrap_active_value<T>(value: &ActiveValue<T>, default: T) -> T
    where
        T: Clone + Into<Value>,
    {
        match value {
            ActiveValue::Set(v) | ActiveValue::Unchanged(v) => v.clone(),
            ActiveValue::NotSet => default,
        }
    }

    fn experience_required_for_level(level: i32) -> i32 {
        100 + (level - 1) * 10
    }

    fn apply_experience(current_level: i32, current_exp: i32, gained_exp: i32) -> (i32, i32) {
        let mut level = current_level;
        let mut exp = current_exp + gained_exp;

        while exp >= Self::experience_required_for_level(level) {
            exp -= Self::experience_required_for_level(level);
            level += 1;
        }

        (level, exp)
    }
}
