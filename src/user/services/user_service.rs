use crate::user::dtos::UserResponse;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
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
            last_experience_at: Set(None),
            timezone: Set("UTC".to_string()),
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

        let user_streak = user.streak;
        let user_timezone = user.timezone.clone();
        let user_last_experience_at = user.last_experience_at;

        let mut active_user: user::ActiveModel = user.into();

        let current_level = Self::unwrap_active_value(&active_user.level, 1);
        let current_exp = Self::unwrap_active_value(&active_user.experience, 0);

        let (new_level, new_exp) = Self::apply_experience(current_level, current_exp, gained_exp);

        active_user.level = Set(new_level);
        active_user.experience = Set(new_exp);

        let tz: Tz = user_timezone.parse().unwrap_or(chrono_tz::America::Lima);
        let now = Utc::now();

        if Self::is_new_day(user_last_experience_at, now, tz) {
            active_user.streak = Set(user_streak + 1);
        }

        active_user.last_experience_at = Set(Some(now));

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

    fn is_new_day(last_time: Option<DateTime<Utc>>, now: DateTime<Utc>, tz: Tz) -> bool {
        match last_time {
            Some(last) => {
                let last_local = last.with_timezone(&tz).date_naive();
                let now_local = now.with_timezone(&tz).date_naive();
                now_local > last_local
            }
            None => true,
        }
    }
}
