use crate::database::PaymentDB;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Json,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PurchaseData {
    course_id: i64,
    user_id: i64,
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
}

fn get_idempotence_key() -> usize {
    std::env::var("IDEMPOTENCE_KEY")
        .expect("Нету ключа")
        .parse()
        .expect("Это точно число, Err не будет")
}

pub async fn redirect_for_payment(
    State(database): State<PaymentDB>,
    Json(data): Json<PurchaseData>,
) -> impl IntoResponse {
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
    };

    match client
        .post("https://api.yookassa.ru/v3/payments")
        .basic_auth(
            "No username(((",
            Some("No password((("),
        )
        .header("Idempotence-Key", get_idempotence_key())
        .json(&payment_request_data)
        .send()
        .await
    {
        Ok(res) => println!("{:?}", res),
        Err(err) => eprintln!("Ошибка с запросом: {}", err),
    }

    let purchase_url = "https://api.yookassa.ru/v3/payments";
    Redirect::to(purchase_url)
}
