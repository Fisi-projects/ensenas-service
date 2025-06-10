use actix_cors::Cors;
use actix_web::{web::Data, App, HttpServer};
use dotenvy::dotenv;
use firebase_auth::FirebaseAuth;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    // Enviroment variables
    let firebase_project_id = std::env::var("FIREBASE_PROJECT_ID")
        .expect("FIREBASE_PROJECT_ID environment variable is required");

    // Firebase auth
    let firebase_auth = FirebaseAuth::new(&firebase_project_id).await;
    let app_data = Data::new(firebase_auth);

    let _jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        println!("Warning: JWT_SECRET not set, using default secret");
        "your_secret_key".to_string()
    });

    println!("Starting application...");

    // Establish database connection
    // let db = establish_connection().await;
    // let db_pool = web::Data::new(db.clone());

    // let auth_service = web::Data::new(auth::services::auth_service::AuthService::new(
    //     db.clone(),
    //     jwt_secret.clone(),
    // ));

    println!("Starting HTTP server...");

    let server = HttpServer::new(move || {
        App::new()
            // .app_data(db_pool.clone())
            // .app_data(auth_service.clone())
            .wrap(actix_web::middleware::Logger::default())
            .app_data(app_data.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::ACCEPT,
                        actix_web::http::header::CONTENT_TYPE,
                    ])
                    .max_age(3600),
            )
        // .configure(configure)
    })
    .bind(("0.0.0.0", 8080));

    let server = match server {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind to address: {}", e);
            return Err(e);
        }
    };

    println!("Server bound to 0.0.0.0:8080");

    match server.run().await {
        Ok(_) => {
            println!("Server shutdown successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("Server error: {}", e);
            Err(e)
        }
    }
}
