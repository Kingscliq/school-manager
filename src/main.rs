mod config;
mod errors;
mod handlers;
mod logger;
mod models;
mod routes;
mod services;

use std::net::{Ipv6Addr, SocketAddr};

use models::AppStore;
use tokio::net::TcpListener;

use config::get_env_vars;
use routes::create_router;

use logger::AppLogger;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    AppLogger::init();
    let fallback_port = 8080;
    let port: u16 = get_env_vars::<u16>("PORT".to_string()).unwrap_or(fallback_port);
    tracing::info!("Starting server on port: {}", port);
    let listening_address: SocketAddr = SocketAddr::from((Ipv6Addr::LOCALHOST, port));
    let store = AppStore::new();
    let app = create_router(store);

    let binder: TcpListener = TcpListener::bind(listening_address)
        .await
        .expect("Failed to bind address");

    // println!("Server is listening on {}", binder.local_addr().unwrap());

    AppLogger::info(&format!("Listening at {}", listening_address));

    AppLogger::info(&format!(
        "Server listening  at {}",
        binder.local_addr().unwrap()
    ));
    axum::serve(binder, app).await.unwrap();
}

// HTTP VERBS

/****
 * POST
 * GET
 * DELETE
 * PATCH
 * PUT
 *
 *
 */
