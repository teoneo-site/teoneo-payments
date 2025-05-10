use crate::crypt;
use crate::database::PaymentDB;
use axum::{extract::State, response::Redirect, Json};
use reqwest::Client;
use serde::{Deserialize, Serialize};
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
        Ok(res) => println!("{:?}", res),
        Err(err) => eprintln!("Ошибка с запросом: {}", err),
    }

    let purchase_url = "https://api.yookassa.ru/v3/payments";
    Ok(Redirect::to(purchase_url))
}
