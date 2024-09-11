use actix_web::{get, web, HttpResponse, Responder};
use crate::statistics::Statistics;
use crate::error::AppError;

#[get("/statistics")]
async fn get_statistics(stats: web::Data<Statistics>) -> Result<impl Responder, AppError> {
    let statistics = stats.get_statistics().await;
    Ok(HttpResponse::Ok().json(statistics))
}

#[get("/system_health")]
async fn get_system_health() -> Result<impl Responder, AppError> {
    let cpu_usage = sys_info::loadavg().map_err(|_| AppError::InternalServerError)?;
    let mem_info = sys_info::mem_info().map_err(|_| AppError::InternalServerError)?;
    let disk_info = sys_info::disk_info().map_err(|_| AppError::InternalServerError)?;

    let system_health = serde_json::json!({
        "cpu_usage": {
            "1min": cpu_usage.one,
            "5min": cpu_usage.five,
            "15min": cpu_usage.fifteen,
        },
        "memory_usage": (mem_info.total - mem_info.avail) as f32 / mem_info.total as f32 * 100.0,
        "disk_usage": (disk_info.total - disk_info.free) as f32 / disk_info.total as f32 * 100.0,
    });

    Ok(HttpResponse::Ok().json(system_health))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_statistics)
       .service(get_system_health);
}