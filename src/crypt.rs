use axum_extra::{headers::{authorization::Bearer, Authorization}, TypedHeader};
use jsonwebtoken::{Validation, decode, DecodingKey};
use reqwest::StatusCode;
use serde::{Serialize, Deserialize};
use axum::{extract::{rejection::FormRejection, FromRequestParts}, http::request::Parts, response::{IntoResponse, Response}, Json};

#[derive(Serialize, Deserialize)]
pub struct ErrorJSON {
    error_type: String,
    error_message: String,
}

impl IntoResponse for ErrorJSON {
    fn into_response(self) -> Response {
        let body = Json(self);
        
        (StatusCode::UNAUTHORIZED, body).into_response()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    id: u32,
    exp: i64,
}

impl<S: Send + Sync> FromRequestParts<S> for Claims {
    type Rejection = ErrorJSON;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Извлечение заголовка Authorization
        let auth_header = parts.headers.get("Authorization")
            .ok_or(ErrorJSON {
                error_type: "BadRequest".to_string(),
                error_message: "Нету Authorization".to_string(),
            })?;

        // Преобразование заголовка в строку
        let token = auth_header.to_str()
            .map_err(|_| ErrorJSON {
                error_type: "BadRequest".to_string(),
                error_message: "Не получается преобразовать заголовок".to_string(),
            })?;

        // Декодирование токена
        let key = DecodingKey::from_secret(std::env::var("SECRET_WORD_REFRESH").unwrap().as_ref());
        let data = decode::<Claims>(token, &key, &Validation::default())
            .map_err(|_| ErrorJSON {
                error_type: "Unauthorized".to_string(),
                error_message: "JWT токен не прошел проверку".to_string(),
            })?;

        Ok(data.claims)
    }
}

pub fn verify_jwt_token(token: &str) -> Result<u32, jsonwebtoken::errors::Error> {
    let validation = Validation::default();

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(std::env::var("SECRET_WORD_JWT").unwrap().as_ref()),
        &validation,
    )?;
    Ok(claims.claims.id)
}