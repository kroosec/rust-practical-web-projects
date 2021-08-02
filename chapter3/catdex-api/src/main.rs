#[macro_use]
extern crate diesel;

use actix_files::Files;
use actix_web::middleware::Logger;
use actix_web::{error, web, App, Error, HttpResponse, HttpServer};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use log::{error, info, warn};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::Deserialize;
use std::env;

use validator::Validate;
use validator_derive::Validate;

mod models;
mod schema;
use self::models::*;
use self::schema::cats::dsl::*;
mod errors;
use self::errors::UserError;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Deserialize, Validate)]
struct CatEndpointPath {
    #[validate(range(min = 1, max = 150))]
    id: i32,
}

async fn cats_endpoint(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let connection = pool.get().map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::UnexpectedError
    })?;

    let cats_data = web::block(move || cats.limit(100).load::<Cat>(&connection))
        .await
        .map_err(|_| {
            error!("Failed to get cats");
            UserError::UnexpectedError
        })?;

    Ok(HttpResponse::Ok().json(cats_data))
}

async fn cat_endpoint(
    pool: web::Data<DbPool>,
    cat_id: web::Path<CatEndpointPath>,
) -> Result<HttpResponse, UserError> {
    cat_id.validate().map_err(|_| {
        warn!("Parameter validation failed");
        UserError::ValidationError
    })?;

    let connection = pool.get().map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::DBPoolGetError
    })?;

    let query_id = cat_id.id.clone();
    let cat_data = web::block(move || cats.filter(id.eq(query_id)).first::<Cat>(&connection))
        .await
        .map_err(|e| match e {
            error::BlockingError::Error(diesel::result::Error::NotFound) => {
                error!("Cat ID: {} not found in DB", &cat_id.id);
                UserError::NotFoundError
            }
            _ => {
                error!("Unexpected error");
                UserError::UnexpectedError
            }
        })?;

    Ok(HttpResponse::Ok().json(cat_data))
}

fn setup_database() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB connections pool")
}

fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .app_data(
                web::PathConfig::default().error_handler(|_, _| UserError::ValidationError.into()),
            )
            .route("/cats", web::get().to(cats_endpoint))
            .route("/cat/{id}", web::get().to(cat_endpoint)),
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key-no-password.pem", SslFiletype::PEM)
        .unwrap();

    let pool = setup_database();
    info!("Listening on port 8082");
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(pool.clone())
            .configure(api_config)
            .service(Files::new("/", "static").show_files_listing())
    })
    .bind_openssl("127.0.0.1:8082", builder)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn test_cats_endpoint_get() {
        let pool = setup_database();
        let mut app = test::init_service(App::new().data(pool.clone()).configure(api_config)).await;

        let req = test::TestRequest::get().uri("/api/cats").to_request();
        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }
}
