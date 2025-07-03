use crate::user::controllers::handlers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .route(web::post().to(handlers::create_user))
            .route(web::get().to(handlers::get_all_users)),
    )
    .service(
        web::resource("/{id}")
            .route(web::get().to(handlers::get_user))
            .route(web::delete().to(handlers::delete_user)),
    );
}
