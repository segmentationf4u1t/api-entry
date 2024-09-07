use actix_web::web;

mod health;
mod rate_test;
mod user;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health::health_check)
        .service(rate_test::rate_test)
        .service(user::register);
}