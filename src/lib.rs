#![allow(dead_code)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate smart_default;

use serde::{Deserialize, Serialize};

mod auth;
mod order;

pub use auth::*;
pub use order::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkDescription {
    href: String,
    rel: String,
    method: String
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
    pub shipping_preference: ShippingPreference
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, SmartDefault)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShippingPreference {
    #[default]
    GetFromFile,
    NoShipping,
    SetProvidedAddress
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, SmartDefault)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserAction {
    #[default]
    Continue,
    PayNow,
}

pub(crate) const ENDPOINT: &str = "https://api.sandbox.paypal.com";

impl Amount {
    pub fn euro(eur: u32, cent: u32) -> Amount {
        Amount {
            value: format!("{}.{}", eur, cent),
            currency_code: "EUR".to_string(),
        }
    }
}