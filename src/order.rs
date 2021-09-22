use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::AccessToken, check_success, Amount, ApplicationContext, Error, LinkDescription, Result,
    ENDPOINT,
};

pub async fn create_order(
    client: &Client,
    token: &AccessToken,
    new: &CreateOrder,
) -> Result<OrderDetails> {
    let response = client
        .post(&format!("{}/v2/checkout/orders", ENDPOINT))
        .bearer_auth(&token.token)
        .header(ACCEPT, "application/json")
        .json(&new)
        .send()
        .await?;

    check_success(&response)?;
    Ok(response.json().await?)
}

pub async fn get_order(client: &Client, token: &AccessToken, id: &str) -> Result<OrderDetails> {
    let response = client
        .get(&format!("{}/v2/checkout/orders/{}", ENDPOINT, id))
        .header(ACCEPT, "application/json")
        .bearer_auth(&token.token)
        .send()
        .await?;

    check_success(&response)?;
    Ok(response.json().await?)
}

pub async fn capture_order(client: &Client, token: &AccessToken, id: &str) -> Result<OrderDetails> {
    let response = client
        .post(&format!("{}/v2/checkout/orders/{}/capture", ENDPOINT, id))
        .bearer_auth(&token.token)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .send()
        .await?;

    check_success(&response)?;

    let response = response.json::<OrderDetails>().await?;

    if response.status != OrderStatus::Completed {
        Err(Error::Api(format!(
            "Unexpected state of order: {:?}",
            response.status
        )))
    } else {
        Ok(response)
    }
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
    pub links: Vec<LinkDescription>,
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
