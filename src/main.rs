mod crypt;
mod database;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let router: Router = Router::new()
        .route("/buy", post(handlers::purchase::redirect_for_payment))
        .route("/webhook", get(handlers::webhook::handle_webhook))
        .with_state(database::PaymentDB::get_pool().await.unwrap());

    let socket = TcpListener::bind("127.0.0.1:8800").await.unwrap();
    axum::serve(socket, router).await.unwrap();
}
