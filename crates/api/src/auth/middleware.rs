use axum::{
    body::Body,
    extract::State,
    http::{header::AUTHORIZATION, HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use axum_common::{AppError, AppResult};

use crate::auth::jwt::{decode_token, Claims};
use crate::state::AppState;

const USER_ID_HEADER: &str = "x-user-id";

fn parse_bearer_token(request: &Request<Body>) -> AppResult<&str> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .ok_or(AppError::Unauthorized)?;
    let auth = auth_header.to_str().map_err(|_| AppError::Unauthorized)?;
    auth.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)
}

fn decode_claims(request: &Request<Body>, secret: &str) -> AppResult<Claims> {
    let token = parse_bearer_token(request)?;
    decode_token(token, secret).map_err(|_| AppError::Unauthorized)
}

pub async fn require_user_auth(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> AppResult<Response> {
    let claims = decode_claims(&request, &state.jwt_secret)?;
    if claims.role != "USER" {
        return Err(AppError::Forbidden);
    }

    let sub_value = HeaderValue::from_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    request.headers_mut().insert(USER_ID_HEADER, sub_value);
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

pub async fn require_admin_auth(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> AppResult<Response> {
    let claims = decode_claims(&request, &state.jwt_secret)?;
    if claims.role != "PLATFORM" && claims.role != "STORE" {
        return Err(AppError::Forbidden);
    }

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}
