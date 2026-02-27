mod auth;
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



/*****
 * 
 * 
 * 
 * I need to have an endpoint that the frontend can call to initiate payment. /pay which accepts amount, and email and then maybe a transaction_reference - 
 * which uses the reqwest crate to make a call to paystacks backend. 
 * once I make that call the response I get from paystack will contain the payment URL that I will send back to the frontend,
 * Once the payment is made I will need to setup a callback URL(not sure where this settings is made) that the user will be redirected to, maybe a frontend app, where they can verify the payment - 
 * the callback URL will contain, the trasation_id and transaction_reference (which one can use to verify the trasaction for sucess and failure).
 * of course I will have an endpoint that will recieve that call to verify the trasaction (is this called idempotency?). 
 * Also I need to setup an endpoint, that is my web hook not sure how that is done, but the goal is the provide an endpoint that paystack will call once a trasaction is completed, 
 * which on that endpoint, I just need to manipulate the students records based on the id of the user that initiated the transaction 
 * like change their status from pending to paid and also saving the payment the payment table
 * 
 * 
 */
