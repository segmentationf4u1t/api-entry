use actix_web::web;

mod health;
mod rate_test;
mod user;
pub(crate) mod statistics;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(statistics::get_statistics)
            .service(health::health_check)
            .service(rate_test::rate_test)
            .service(user::register)
            .service(user::get_user)
            .configure(statistics::config)
    );
}