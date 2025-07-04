use actix_web::{web, HttpResponse, Responder};
use firebase_auth::FirebaseUser;
use log::{error, info};
use sea_orm::DatabaseConnection;

use crate::user::{dtos::AddExperiencePayload, services::UserService};

pub async fn get_user(db: web::Data<DatabaseConnection>, id: web::Path<String>) -> impl Responder {
    let user_id = id.into_inner();
    info!("Fetching user with id: {user_id}");
    let db = db.get_ref();

    match UserService::get_user(db, &user_id).await {
        Ok(user) => {
            info!("Successfully fetched user: {user:?}");
            HttpResponse::Ok().json(user)
        }
        Err(e) => {
            error!("Failed to fetch user with id: {user_id:?}: {e:?}");
            HttpResponse::NotFound().body("User not found")
        }
    }
}

pub async fn get_all_users(db: web::Data<DatabaseConnection>) -> impl Responder {
    info!("Fetching all users");
    let db = db.get_ref();

    match UserService::get_all_users(db).await {
        Ok(users) => {
            info!("Successfully fetched {} users", users.len());
            HttpResponse::Ok().json(users)
        }
        Err(e) => {
            error!("Failed to fetch users: {e:?}");
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

pub async fn create_user(db: web::Data<DatabaseConnection>, user: FirebaseUser) -> impl Responder {
    info!("Creating user");
    let db = db.get_ref();

    match UserService::create_user_from_token(db, &user).await {
        Ok(user) => {
            info!("Successfully created user: {user:?}");
            HttpResponse::Created().json(user)
        }
        Err(err) => {
            error!("Failed to create user: {err:?}");
            if err.to_string().contains("Authorization") {
                HttpResponse::Unauthorized().body(err.to_string())
            } else if err.to_string().contains("verify Firebase") {
                HttpResponse::Unauthorized().body("Invalid Firebase token")
            } else {
                HttpResponse::InternalServerError().body("Something went wrong")
            }
        }
    }
}

pub async fn delete_user(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>,
) -> impl Responder {
    let user_id = id.into_inner();
    info!("Deleting user with id: {user_id:?}");
    let db = db.get_ref();

    match UserService::delete_user(db, &user_id).await {
        Ok(_) => {
            info!("Successfully deleted user with id: {user_id}");
            HttpResponse::Ok().body("User deleted")
        }
        Err(e) => {
            error!("Failed to delete user with id: {user_id:?}: {e:?}");
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

pub async fn add_experience(
    db: web::Data<DatabaseConnection>,
    user_id: web::Path<String>,
    payload: web::Json<AddExperiencePayload>,
) -> impl Responder {
    let db = db.get_ref();

    match UserService::add_experience(db, &user_id, payload.gained_exp).await {
        Ok(updated_user) => HttpResponse::Ok().json(updated_user),
        Err(e) => {
            eprintln!("Failed to add experience: {e}");
            HttpResponse::InternalServerError().body("Failed to add experience")
        }
    }
}
