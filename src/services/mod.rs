use serde::{Deserialize, Serialize};
use crate::errors::AppError;

#[derive(Serialize)]
struct InitializePaymentBody {
    email: String,
    amount: u64, // in kobo (smallest currency unit). e.g. NGN 5000 = 500000 kobo
    reference: String,
}

#[derive(Deserialize)]
struct PaystackInitResponse {
    status: bool,
    data: PaystackInitData,
}

#[derive(Deserialize)]
pub struct PaystackInitData {
    pub authorization_url: String,
    pub reference: String,
}

pub async fn initialize_paystack_transaction(
    secret_key: &str,
    email: &str,
    amount_kobo: u64,
    reference: &str,
) -> Result<PaystackInitData, AppError> {
    let client = reqwest::Client::new();

    let body = InitializePaymentBody {
        email: email.to_string(),
        amount: amount_kobo,
        reference: reference.to_string(),
    };

    let response = client
        .post("https://api.paystack.co/transaction/initialize")
        .header("Authorization", format!("Bearer {}", secret_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let parsed = response
        .json::<PaystackInitResponse>()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    if !parsed.status {
        return Err(AppError::InternalServerError(
            "Paystack rejected the transaction".to_string(),
        ));
    }

    Ok(parsed.data)
}