use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::{
    auth::models::{Claims, User},
    errors::AppError,
};

// Authentication
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| AppError::UnAuthorized(err.to_string()))
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|err| AppError::UnAuthorized(err.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// Authorization
pub fn create_access_token(user: &User, secret: &str) -> Result<String, AppError> {
    let claims = Claims {
        sub: user.id,
        school_id: user.school_id,
        role: user.role.clone(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|err| AppError::UnAuthorized(err.to_string()))
}

pub fn verify_access_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::default();
    validation.validate_exp = false;

    /***
     *
     * this is because the token must live forever, and that is why we are diabling required checks
     * There was supposed to be an exp
     *
     */
    validation.required_spec_claims.clear();

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|err| AppError::InternalServerError(err.to_string()))
}
