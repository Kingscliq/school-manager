use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{
    auth::models::UserRole,
    auth::service::verify_access_token,
    config::get_env_vars,
    models::AppStore,
};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub school_id: Option<Uuid>,
    pub role: UserRole,
}

pub async fn auth_middleware(
    State(_app_store): State<AppStore>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let jwt_secret: String = get_env_vars("JWT_SECRET".to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let claims = verify_access_token(token, &jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
    req.extensions_mut().insert(AuthContext {
        user_id: claims.sub,
        school_id: claims.school_id,
        role: claims.role,
    });

    Ok(next.run(req).await)
}
