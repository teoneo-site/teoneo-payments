use axum::response::{IntoResponse, Redirect};
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct Amount {
    value: String,
    currency: String,
}

#[derive(Serialize)]
struct Confirmation {
    #[serde(rename = "type")]
    confirmation_type: String,
    return_url: String,
}

#[derive(Serialize)]
struct PaymentRequest {
    amount: Amount,
    capture: bool,
    confirmation: Confirmation,
    description: String,
}

async fn redirect_for_payment() -> impl IntoResponse {
    let purchase_url = todo!();
    Redirect::to(purchase_url)
}