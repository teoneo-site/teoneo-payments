use crate::crypt::{self, ErrorJSON, ErrorMsg};
use crate::database::PaymentDB;
use axum::{extract::State, response::Redirect, Json};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct PurchaseData {
    course_id: i64,
}

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
    metadata: Metadata,
}

#[derive(Deserialize)]
struct ConfirmationAnswer {
    #[serde(rename = "type")]
    _confirmation_type: String,
    confirmation_url: String,
}

#[derive(Serialize, Deserialize)]
struct Metadata {
    user_id: i64,
    course_id: i64,
}

fn get_idempotence_key() -> Uuid {
    Uuid::new_v4()
}

pub async fn redirect_for_payment(
    State(database): State<PaymentDB>,
    claims: crypt::Claims,
    Json(data): Json<PurchaseData>,
) -> Result<Redirect, crypt::ErrorMsg> {
    let client = Client::new();

    let amount = Amount {
        value: database
            .get_course_price(data.course_id)
            .await
            .expect("Нету юзера")
            .to_string(),
        currency: String::from("RUB"),
    };

    let confirmation = Confirmation {
        confirmation_type: String::from("redirect"),
        return_url: format!(
            "{}/courses/{}",
            std::env::var("SERVER_ADDR").expect("Нету сервера"),
            data.course_id
        ),
    };

    let payment_request_data = PaymentRequest {
        amount,
        capture: true,
        confirmation,
        description: String::from("Хз че тут написать, потом дополнить надо"),
        metadata: Metadata {
            user_id: claims.id,
            course_id: data.course_id,
        },
    };

    match client
        .post("https://api.yookassa.ru/v3/payments")
        .basic_auth("No username(((", Some("No password((("))
        .header("Idempotence-Key", get_idempotence_key().to_string())
        .json(&payment_request_data)
        .send()
        .await
    {

        Ok(res) => {
            let data = res.json::<Value>().await.map_err(|_| ErrorMsg {
                json_data: ErrorJSON {
                    error_type: String::from("BadRequest"),
                    error_message: String::from("Ответ от платежки пришел без JSON"),
                },
                status_code: StatusCode::BAD_REQUEST,
            })?;

            let confirmation =
                serde_json::from_value::<ConfirmationAnswer>(data["confirmation"].clone()).unwrap();

            Ok(Redirect::to(&confirmation.confirmation_url))
        }

        Err(err) => Err(ErrorMsg {
            json_data: ErrorJSON {
                error_type: String::from("InternalServerError"),
                error_message: String::from(format!("Проблема на стороне платежки: {err}")),
            },
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }),
    }
}
