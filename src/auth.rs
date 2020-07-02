use crate::ENDPOINT;
use crate::{Error, Result};
use parking_lot::Mutex;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccessToken {
    #[serde(rename = "access_token")]
    pub token: String,
    pub expires_in: u64,
}

#[derive(Clone)]
pub struct RefreshingAccessToken(Arc<Mutex<AccessToken>>);

fn refresh_in(secs: u64) -> Duration {
    Duration::from_secs(secs.saturating_sub(10))
}

impl RefreshingAccessToken {
    async fn refresh_periodically(self, client: Client, id: String, secret: String) -> Error {
        let mut timeout = refresh_in(self.0.lock().expires_in);
        loop {
            info!("refreshing access token in {:?}", timeout);
            tokio::time::delay_for(timeout).await;

            *self.0.lock() = match AccessToken::get(&client, &id, &secret).await {
                Ok(token) => {
                    timeout = refresh_in(token.expires_in);
                    token
                },
                Err(err) => return err,
            };
        }
    }

    pub async fn refreshing(
        client: &Client,
        id: impl Into<String>,
        secret: impl Into<String>,
    ) -> Result<(Self, JoinHandle<Error>)> {
        let id = id.into();
        let secret = secret.into();

        let initial = AccessToken::get(client, &id, &secret).await?;
        let token = RefreshingAccessToken(Arc::new(Mutex::new(initial)));

        let refresh_future = token.clone().refresh_periodically(client.clone(), id, secret);
        let handle = tokio::task::spawn(refresh_future);

        Ok((token, handle))
    }

    pub fn current(&self) -> AccessToken {
        self.0.deref().lock().clone()
    }
}

impl AccessToken {
    pub async fn get(client: &Client, id: &str, secret: &str) -> Result<AccessToken> {
        info!("acquiring access token..");
        let url = format!("{}/v1/oauth2/token?grant_type=client_credentials", ENDPOINT);

        let response = client
            .post(&url)
            .basic_auth(id, Some(secret))
            .header(ACCEPT_LANGUAGE, "en_US")
            .header(ACCEPT, "application/json")
            .send()
            .await?
            .json()
            .await?;

        info!("successfully acquired access token!");

        Ok(response)
    }
}
