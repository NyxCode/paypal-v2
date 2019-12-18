use surf::Exception;

use crate::ENDPOINT;
use async_std::sync::{Arc, RwLock};
use async_std::task;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccessToken {
    #[serde(rename = "access_token")]
    pub token: String,
    pub expires_in: u64,
}

impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        &self.token
    }
}

#[derive(Clone)]
pub struct RefreshingAccessToken(Arc<RwLock<AccessToken>>);

impl RefreshingAccessToken {
    pub fn refreshing(id: &str, secret: &str) -> Result<Self, Exception> {
        let initial = task::block_on(AccessToken::get(id, secret))?;
        let token = RefreshingAccessToken(Arc::new(RwLock::new(initial)));

        let token_ref = token.clone();
        let id = id.to_owned();
        let secret = secret.to_owned();
        task::spawn(async move {
            loop {
                let interval = token_ref.0.deref().read().await.expires_in / 2;
                let interval = Duration::from_secs(interval);
                info!("refreshing access token in {:?}", interval);
                task::sleep(interval).await;
                let new_token = AccessToken::get(&id, &secret)
                    .await
                    .expect("could not refresh access token");
                *token_ref.0.deref().write().await = new_token;
            }
        });

        Ok(token)
    }
}

impl AccessToken {
    pub async fn get(id: &str, secret: &str) -> Result<AccessToken, Exception> {
        info!("refreshing access token..");
        let url = format!("{}/v1/oauth2/token?grant_type=client_credentials", ENDPOINT);
        let basic_auth = format!("Basic {}", base64::encode(&format!("{}:{}", id, secret)));

        let response: AccessToken = surf::post(url)
            .set_header("Accept", "application/json")
            .set_header("Accept-Language", "en_US")
            .set_header("Authorization", basic_auth)
            .recv_json()
            .await?;

        info!("successfully refreshed access token!");

        Ok(response)
    }
}
