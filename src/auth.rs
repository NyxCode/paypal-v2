use crate::Result;
use crate::ENDPOINT;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::Stream;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccessToken {
    #[serde(rename = "access_token")]
    pub token: String,
    pub expires_in: u64,
}

impl AccessToken {
    pub fn refresh_periodically(
        client: &Client,
        id: impl Into<String>,
        secret: impl Into<String>,
    ) -> impl Stream<Item = Result<AccessToken>> {
        let credentials = Arc::new((id.into(), secret.into()));
        let client = client.clone();

        futures::stream::try_unfold(0, move |delay| {
            let credentials = credentials.clone();
            let client = client.clone();
            async move {
                if delay > 0 {
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                }

                let (id, secret) = &*credentials;
                let token = AccessToken::acquire(&client, &id, &secret).await?;
                let expires_in = token.expires_in;
                Ok(Some((token, expires_in)))
            }
        })
    }

    pub async fn acquire(client: &Client, id: &str, secret: &str) -> Result<AccessToken> {
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
