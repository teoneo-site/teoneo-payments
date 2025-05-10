use super::metadata::Metadata;
use crate::database::PaymentDB;
use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use reqwest::StatusCode;
use serde_json::Value;

pub async fn handle_webhook(
    State(database): State<PaymentDB>,
    Json(data): Json<Value>,
) -> impl IntoResponse {
    let metadata = match serde_json::from_value::<Metadata>(data["object"]["metadata"].clone()) {
        Ok(val) => val,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "IDK"),
    };

    match database
        .register_payment(metadata.user_id, metadata.course_id, Utc::now().naive_utc())
        .await
    {
        Ok(_) => (StatusCode::OK, "Okay"),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IDK"),
    }
}
