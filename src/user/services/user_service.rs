use crate::{
    user::dtos::{FirebaseClaims, UserResponse},
    utils::verify_token,
};
use actix_web::HttpRequest;
use anyhow::{Context, Result};
use entity::user::{self, Entity};
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
        req: &HttpRequest,
    ) -> Result<user::Model> {
        // Extract token from Authorization header
        let token = req
            .headers()
            .get("Authorization")
            .context("Missing Authorization header")?
            .to_str()
            .context("Invalid Authorization header format")?
            .strip_prefix("Bearer ")
            .context("Authorization header must start with 'Bearer '")?;

        // Verify token and extract claims
        let claims: FirebaseClaims = verify_token(token)
            .await
            .context("Failed to verify Firebase token")?;

        // Create new user model
        let new_user = user::ActiveModel {
            id: Set(claims.uid),
            name: Set(claims.name),
            email: Set(claims.email),
            level: Set(1),
            streak: Set(0),
        };

        // Insert into DB
        let inserted_user = new_user
            .insert(db)
            .await
            .context("Failed to insert user into database")?;

        Ok(inserted_user)
    }

    pub async fn delete_user(db: &DatabaseConnection, user_id: &str) -> Result<(), DbErr> {
        Entity::delete_by_id(user_id).exec(db).await?;
        Ok(())
    }
}
