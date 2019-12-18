use crate::auth::AccessToken;
use crate::{Amount, ApplicationContext, ENDPOINT, LinkDescription};
use serde::{Deserialize, Serialize};
use surf::Exception;

pub async fn create_order(
    token: &AccessToken,
    new: &CreateOrder,
) -> Result<OrderDetails, Exception> {
    let mut response = surf::post(format!("{}/v2/checkout/orders", ENDPOINT))
        .set_header("Authorization", format!("Bearer {}", &token.token))
        .body_json(new)?
        .await?;


    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()).into());
    }

    Ok(response.body_json().await?)
}

pub async fn get_order(token: &AccessToken, id: &str) -> Result<OrderDetails, Exception> {
    let mut response = surf::get(format!("{}/v2/checkout/orders/{}", ENDPOINT, id))
        .set_header("Authorization", format!("Bearer {}", &token.token))
        .await?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()).into());
    }

    Ok(response.body_json().await?)
}

pub async fn capture_order(token: &AccessToken, id: &str) -> Result<OrderDetails, Exception> {
    let mut response = surf::post(format!("{}/v2/checkout/orders/{}/capture", ENDPOINT, id))
        .set_header("Authorization", format!("Bearer {}", &token.token))
        .body_string("{}".to_owned())
        .set_mime(surf::mime::APPLICATION_JSON)
        .await?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()).into());
    }

    let response: OrderDetails = response.body_json().await?;

    if response.status != OrderStatus::Completed {
        return Err(format!("Unexpected state of order: {:?}", response.status).into());
    }

    Ok(response)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateOrder {
    pub intent: OrderIntent,
    pub purchase_units: Vec<PurchaseUnitRequest>,
    pub application_context: ApplicationContext,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderDetails {
    pub id: String,
    pub status: OrderStatus,
    pub links: Vec<LinkDescription>
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
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
    Capture,
    Authorize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PurchaseUnitRequest {
    pub amount: Amount,
    pub description: String,
}
