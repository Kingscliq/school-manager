use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use sha2::Sha512;
use uuid::Uuid;

use crate::{
    config::get_env_vars,
    models::{AppStore, CreateStudentRequest},
    services::initialize_paystack_transaction,
};

pub async fn create_student_handler(
    State(app_store): State<AppStore>,
    Json(req): Json<CreateStudentRequest>,
) -> impl IntoResponse {
    match app_store.create_student(req).await {
        Ok(()) => (StatusCode::CREATED, Json("Student created successfully".to_string())).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

pub async fn get_all_students_handler(State(app_store): State<AppStore>) -> impl IntoResponse {
    match app_store.get_all_students().await {
        Ok(students) => (StatusCode::OK, Json(students)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

pub async fn get_student_handler(
    State(app_store): State<AppStore>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match app_store.get_student(id).await {
        Ok(student) => (StatusCode::OK, Json(student)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

pub async fn delete_student_handler(
    State(app_store): State<AppStore>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match app_store.delete_student(id).await {
        Ok(_) => (StatusCode::OK, Json("Student deleted!")).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

// Initiates a Paystack payment for a student and returns the checkout URL
pub async fn initiate_payment_handler(
    State(app_store): State<AppStore>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let secret_key: String = match get_env_vars("PAYSTACK_SECRET_KEY".to_string()) {
        Ok(k) => k,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    };

    let student = match app_store.get_student(id).await {
        Ok(s) => s,
        Err(e) => return (StatusCode::NOT_FOUND, Json(e.to_string())).into_response(),
    };

    // Generate a unique reference for this transaction
    let reference = format!("sch-{}", Uuid::new_v4());
    // Amount in kobo — adjust this to your school fees amount
    let amount_kobo: u64 = 500_000; // NGN 5,000

    match initialize_paystack_transaction(&secret_key, &student.email, amount_kobo, &reference).await {
        Ok(data) => {
            // Store the reference so the webhook can find this student later
            let _ = app_store.set_payment_reference(id, data.reference.clone()).await;
            (StatusCode::OK, Json(serde_json::json!({
                "authorization_url": data.authorization_url,
                "reference": data.reference,
            }))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}

// Paystack calls this when a payment succeeds
pub async fn paystack_webhook_handler(
    State(app_store): State<AppStore>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let secret_key: String = match get_env_vars("PAYSTACK_SECRET_KEY".to_string()) {
        Ok(k) => k,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    // Verify the webhook signature using HMAC-SHA512
    let signature = match headers.get("x-paystack-signature").and_then(|v| v.to_str().ok()) {
        Some(sig) => sig.to_string(),
        None => return StatusCode::UNAUTHORIZED,
    };

    let mut mac = Hmac::<Sha512>::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(&body);
    let expected = hex::encode(mac.finalize().into_bytes());

    if expected != signature {
        return StatusCode::UNAUTHORIZED;
    }

    // Parse the event body
    let event: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    // Only handle successful charge events
    if event["event"] == "charge.success" {
        if let Some(reference) = event["data"]["reference"].as_str() {
            let _ = app_store.mark_student_paid_by_reference(reference).await;
        }
    }

    // Always return 200 so Paystack stops retrying
    StatusCode::OK
}

