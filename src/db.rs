use entity::User;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Schema};
use std::env;

pub async fn establish_connection() -> DatabaseConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Wait for database to be ready
    let mut attempts = 0;
    let max_attempts = 5;

    println!("Trying to connect to db");
    while attempts < max_attempts {
        match Database::connect(&database_url).await {
            Ok(db) => {
                println!("Database connected successfully!");

                // Create the schema builder
                let builder = db.get_database_backend();
                let schema = Schema::new(builder);

                // Create table statements
                let stmt_user = builder.build(&schema.create_table_from_entity(User));

                // Execute the create table statements in the correct order
                let results = vec![("User", db.execute(stmt_user).await)];

                for (table_name, result) in results {
                    match result {
                        Ok(_) => println!("Table '{table_name}' created successfully!"),
                        Err(e) => println!("Error creating table '{table_name}': {e}"),
                    }
                }

                return db;
            }
            Err(e) => {
                println!(
                    "Failed to connect to database: {}. Attempt {} of {}",
                    e,
                    attempts + 1,
                    max_attempts
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                attempts += 1;
            }
        }
    }

    panic!("Failed to connect to database after {max_attempts} attempts");
}
