#![allow(dead_code)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate smart_default;

use serde::{Deserialize, Serialize};
use thiserror::Error;
mod auth;
mod order;

pub use auth::*;
pub use order::*;
use reqwest::Response;

#[derive(Debug, Error)]
pub enum Error {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("unexpected http status: {0}")]
    Status(u16),
    #[error("unexpected api response: {0}")]
    Api(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkDescription {
    pub href: String,
    pub rel: String,
    pub method: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Amount {
    pub value: String,
    pub currency_code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, SmartDefault)]
pub struct ApplicationContext {
    pub brand_name: Option<String>,
    pub locale: Option<String>,
    pub user_action: UserAction,
    pub return_url: Option<String>,
    pub cancel_url: Option<String>,
    pub shipping_preference: ShippingPreference,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, SmartDefault)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShippingPreference {
    #[default]
    GetFromFile,
    NoShipping,
    SetProvidedAddress,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, SmartDefault)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserAction {
    #[default]
    Continue,
    PayNow,
}

#[cfg(not(feature = "production"))]
pub(crate) const ENDPOINT: &str = "https://api.sandbox.paypal.com";
#[cfg(feature = "production")]
pub(crate) const ENDPOINT: &str = "https://api.paypal.com";

impl Amount {
    pub fn euro(eur: u32, cent: u32) -> Amount {
        Amount {
            value: format!("{}.{}", eur, cent),
            currency_code: "EUR".to_string(),
        }
    }
}

fn check_success(response: &Response) -> Result<()> {
    if response.status().is_success() {
        Ok(())
    } else {
        Err(Error::Status(
            response.status().as_u16(),
        ))
    }
}
