use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ErrorJSON {
    error_type: String,
    error_message: String,
}

pub struct ErrorMsg {
    json_data: ErrorJSON,
    status_code: StatusCode,
}

impl IntoResponse for ErrorMsg {
    fn into_response(self) -> Response {
        let body = Json(self.json_data);

        let mut headers = HeaderMap::new();
        headers.append("Content-Type", HeaderValue::from_static("application/json"));

        (self.status_code, headers, body).into_response()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: i64,
    exp: i64,
}

impl<S: Send + Sync> FromRequestParts<S> for Claims {
    type Rejection = ErrorMsg;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Извлечение заголовка Authorization
        let auth_header = parts.headers.get("Authorization").ok_or(ErrorMsg {
            json_data: ErrorJSON {
                error_type: "BadRequest".to_string(),
                error_message: "Нету Authorization".to_string(),
            },
            status_code: StatusCode::BAD_REQUEST,
        })?;

        // Преобразование заголовка в строку
        let token = auth_header.to_str().map_err(|_| ErrorMsg {
            json_data: ErrorJSON {
                error_type: "BadRequest".to_string(),
                error_message: "Не получается преобразовать заголовок".to_string(),
            },
            status_code: StatusCode::BAD_REQUEST,
        })?;

        // Декодирование токена
        let key = DecodingKey::from_secret(std::env::var("SECRET_WORD_JWT").unwrap().as_ref());
        let data = decode::<Claims>(token, &key, &Validation::default()).map_err(|_| ErrorMsg {
            json_data: ErrorJSON {
                error_type: "Unauthorized".to_string(),
                error_message: "JWT токен не прошел проверку".to_string(),
            },
            status_code: StatusCode::UNAUTHORIZED,
        })?;

        Ok(data.claims)
    }
}
