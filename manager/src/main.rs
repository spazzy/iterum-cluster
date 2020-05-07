#[macro_use]
extern crate log;
use actix_web::{web, HttpResponse};
use actix_web::{App, HttpServer};
use dotenv::dotenv;

use listenfd::ListenFd;
use std::env;

mod config;
mod error;
mod pipelines;

use crate::pipelines::pipeline_manager::PipelineManager;

use actix::prelude::*;
use std::collections::HashMap;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let mut listenfd = ListenFd::from_env();

    let manager = PipelineManager {
        addresses: HashMap::new(),
    };
    let addr = manager.start();

    let config = web::Data::new(config::Config { manager: addr });

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                let message = format!("Error when handling JSON: {:?}", err);
                error!("{}", message);
                actix_web::error::InternalError::from_response(
                    err,
                    HttpResponse::Conflict().body(message),
                )
                .into()
            }))
            .configure(pipelines::init_routes)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST").expect("Host not set");
            let port = env::var("PORT").expect("Port not set");
            server.bind(format!("{}:{}", host, port))?
        }
    };

    // let response = addr.send(Ping { amount: 1 });

    info!("Starting Iterum Cluster Manager");
    server.run().await
}
