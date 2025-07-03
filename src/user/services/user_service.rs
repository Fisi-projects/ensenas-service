use crate::user::dtos::UserResponse;
use anyhow::{Context, Result};
use entity::user::{self, Entity};
use firebase_auth::FirebaseUser;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};

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
}
