use crate::auth::AccessToken;
use crate::{Amount, ApplicationContext, ENDPOINT};
use serde::{Deserialize, Serialize};
use surf::Exception;

pub async fn create_order(
    token: &AccessToken,
    new: &CreateOrder,
) -> Result<CreateOrderResponse, Exception> {
    let mut response = surf::post(format!("{}/v2/checkout/orders", ENDPOINT))
        .set_header("Authorization", format!("Bearer {}", token.as_ref()))
        .body_json(new)?
        .await?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()).into());
    }

    Ok(response.body_json().await?)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateOrder {
    pub intent: OrderIntent,
    pub purchase_units: Vec<PurchaseUnitRequest>,
    pub application_context: ApplicationContext,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateOrderResponse {
    pub id: String,
    pub status: OrderStatus,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
    Created,
    Saved,
    Approved,
    Voided,
    Completed,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, SmartDefault)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderIntent {
    #[default]
    #[serde(rename = "CAPTURE")]
    Capture,
    #[serde(rename = "AUTHORIZE")]
    Authorize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PurchaseUnitRequest {
    pub amount: Amount,
    pub description: String,
}
