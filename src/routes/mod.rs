use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    handlers::{
        create_student_handler, delete_student_handler, get_all_students_handler,
        get_student_handler, initiate_payment_handler, paystack_webhook_handler,
    },
    models::AppStore,
};

pub fn create_router(store: AppStore) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello from Axum! 🦀" }))
        .route(
            "/student",
            post(create_student_handler).get(get_all_students_handler),
        )
        .route(
            "/student/{id}",
            get(get_student_handler)
            .delete(delete_student_handler),
        )
        // New routes
        .route("/student/{id}/pay", post(initiate_payment_handler))
        .route("/webhook/paystack", post(paystack_webhook_handler))
        .with_state(store)
}