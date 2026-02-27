use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UserRole {
    Student,
    Lecturer,
    SchoolAdmin,
    SuperAdmin,
}

#[derive(Debug, Clone, Serialize)]
pub struct School {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub is_active: bool,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateSchoolRequest {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub school_id: Option<Uuid>,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub school_id: Option<Uuid>,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: i64,
}

impl From<&User> for UserProfile {
    fn from(value: &User) -> Self {
        Self {
            id: value.id,
            school_id: value.school_id,
            email: value.email.clone(),
            first_name: value.first_name.clone(),
            last_name: value.last_name.clone(),
            role: value.role.clone(),
            is_active: value.is_active,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub school_id: Option<Uuid>,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role: Option<UserRole>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub user: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub school_id: Option<Uuid>,
    pub role: UserRole,
}
