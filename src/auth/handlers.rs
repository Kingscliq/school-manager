use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    auth::{
        middleware::AuthContext,
        models::{
            AuthResponse, CreateSchoolRequest, LoginRequest, RegisterRequest, User, UserProfile,
            UserRole,
        },
        service::{create_access_token, hash_password, verify_password},
    },
    config::get_env_vars,
    models::AppStore,
};

fn trim_required(value: &str, field: &str) -> Result<String, (StatusCode, Json<String>)> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(format!("{field} cannot be empty")),
        ));
    }
    Ok(trimmed.to_string())
}

pub async fn create_school_handler(
    State(app_store): State<AppStore>,
    Json(req): Json<CreateSchoolRequest>,
) -> impl IntoResponse {
    let name = match trim_required(&req.name, "name") {
        Ok(value) => value,
        Err(err) => return err.into_response(),
    };

    let payload = CreateSchoolRequest { name };
    match app_store.create_school(payload).await {
        Ok(school) => (StatusCode::CREATED, Json(school)).into_response(),
        Err(err) => (StatusCode::UNPROCESSABLE_ENTITY, Json(err.to_string())).into_response(),
    }
}

pub async fn register_handler(
    State(app_store): State<AppStore>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    let jwt_secret: String = match get_env_vars("JWT_SECRET".to_string()) {
        Ok(secret) => secret,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
        }
    };

    let email = match trim_required(&req.email, "email") {
        Ok(value) => value,
        Err(err) => return err.into_response(),
    };
    let first_name = match trim_required(&req.first_name, "first_name") {
        Ok(value) => value,
        Err(err) => return err.into_response(),
    };
    let last_name = match trim_required(&req.last_name, "last_name") {
        Ok(value) => value,
        Err(err) => return err.into_response(),
    };
    let password = req.password.trim().to_string();
    if password.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json("password cannot be empty".to_string()),
        )
            .into_response();
    }

    let password_hash = match hash_password(&password) {
        Ok(hash) => hash,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
        }
    };

    if let Some(school_id) = req.school_id {
        if !app_store.school_exists(school_id).await {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json("Invalid school_id".to_string()),
            )
                .into_response();
        }
    }

    let user = User {
        id: Uuid::new_v4(),
        school_id: req.school_id,
        email,
        password_hash,
        first_name,
        last_name,
        role: req.role.unwrap_or(UserRole::Student),
        is_active: true,
        created_at: Utc::now().timestamp(),
    };

    let created_user = match app_store.create_user(user).await {
        Ok(user) => user,
        Err(err) => {
            return (StatusCode::UNPROCESSABLE_ENTITY, Json(err.to_string())).into_response();
        }
    };

    let access_token = match create_access_token(&created_user, &jwt_secret) {
        Ok(token) => token,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
        }
    };

    (
        StatusCode::CREATED,
        Json(AuthResponse {
            access_token,
            user: UserProfile::from(&created_user),
        }),
    )
        .into_response()
}

pub async fn login_handler(
    State(app_store): State<AppStore>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let jwt_secret: String = match get_env_vars("JWT_SECRET".to_string()) {
        Ok(secret) => secret,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
        }
    };

    let email = match trim_required(&req.email, "email") {
        Ok(value) => value,
        Err(err) => return err.into_response(),
    };
    let password = req.password.trim().to_string();
    if password.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json("password cannot be empty".to_string()),
        )
            .into_response();
    }

    let user = match app_store.find_user_by_email(&email).await {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json("Invalid email or password".to_string()),
            )
                .into_response();
        }
    };

    let password_ok = match verify_password(&password, &user.password_hash) {
        Ok(result) => result,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
        }
    };

    if !password_ok {
        return (
            StatusCode::UNAUTHORIZED,
            Json("Invalid email or password".to_string()),
        )
            .into_response();
    }

    let access_token = match create_access_token(&user, &jwt_secret) {
        Ok(token) => token,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
        }
    };

    (
        StatusCode::OK,
        Json(AuthResponse {
            access_token,
            user: UserProfile::from(&user),
        }),
    )
        .into_response()
}

pub async fn me_handler(
    State(app_store): State<AppStore>,
    Extension(ctx): Extension<AuthContext>,
) -> impl IntoResponse {
    let user = match app_store.find_user_by_id(ctx.user_id).await {
        Some(user) => user,
        None => {
            return (StatusCode::NOT_FOUND, Json("User not found".to_string())).into_response();
        }
    };

    (StatusCode::OK, Json(UserProfile::from(&user))).into_response()
}

pub async fn get_all_users_handler(
    State(app_store): State<AppStore>,
    Extension(ctx): Extension<AuthContext>,
) -> impl IntoResponse {
    if !matches!(ctx.role, UserRole::SchoolAdmin | UserRole::SuperAdmin) {
        return (StatusCode::FORBIDDEN, Json("Forbidden".to_string())).into_response();
    }

    let users = app_store.get_all_users().await;
    let profiles: Vec<UserProfile> = users
        .iter()
        .filter(|u| {
            matches!(ctx.role, UserRole::SuperAdmin)
                || (ctx.school_id.is_some() && u.school_id == ctx.school_id)
        })
        .map(UserProfile::from)
        .collect();

    (StatusCode::OK, Json(profiles)).into_response()
}

pub async fn get_all_schools_handler(
    State(app_store): State<AppStore>,
    Extension(ctx): Extension<AuthContext>,
) -> impl IntoResponse {
    if !matches!(ctx.role, UserRole::SuperAdmin) {
        return (StatusCode::FORBIDDEN, Json("Forbidden".to_string())).into_response();
    }

    let schools = app_store.get_all_schools().await;
    (StatusCode::OK, Json(schools)).into_response()
}
