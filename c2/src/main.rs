mod dao;
mod entities;
mod error;
mod handlers;
mod utils;

use actix_web::{web, App, HttpServer};

pub struct CcState {
    jwt_signing_key: String,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let db = dao::init().await?;

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::new(CcState {
                jwt_signing_key: "arVmmx4E".to_string(),
            }))
            .app_data(web::Data::new(db.clone()))
            .configure(handlers::config)
    })
    .bind_openssl(("0.0.0.0", 443), {
        let mut builder =
            openssl::ssl::SslAcceptor::mozilla_intermediate(openssl::ssl::SslMethod::tls())
                .unwrap();
        builder
            .set_private_key_file("certs/key.pem", openssl::ssl::SslFiletype::PEM)
            .unwrap();
        builder
            .set_certificate_chain_file("certs/cert.pem")
            .unwrap();
        builder
    })?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
