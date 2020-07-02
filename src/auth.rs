use crate::ENDPOINT;
use crate::{Error, Result};
use parking_lot::Mutex;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccessToken {
    #[serde(rename = "access_token")]
    pub token: String,
    pub expires_in: u64,
}

pub struct RefreshingAccessToken {
    token: Arc<Mutex<AccessToken>>,
    pub join_handle: JoinHandle<Result<()>>,
    pub shutdown: oneshot::Sender<()>,
}

fn refresh_in(secs: u64) -> Duration {
    Duration::from_secs(secs.saturating_sub(10))
}

async fn refresh_periodically(
    token: Arc<Mutex<AccessToken>>,
    client: Client,
    id: String,
    secret: String,
) -> Error {
    let mut timeout = refresh_in(token.lock().expires_in);
    loop {
        info!("refreshing access token in {:?}", timeout);
        tokio::time::delay_for(timeout).await;

        *token.lock() = match AccessToken::get(&client, &id, &secret).await {
            Ok(token) => {
                timeout = refresh_in(token.expires_in);
                token
            }
            Err(err) => return err,
        };
    }
}

impl RefreshingAccessToken {
    pub async fn refreshing(
        client: &Client,
        id: impl Into<String>,
        secret: impl Into<String>,
    ) -> Result<Self> {
        let id = id.into();
        let secret = secret.into();

        let initial = Arc::new(Mutex::new(AccessToken::get(client, &id, &secret).await?));
        let refresh_future = refresh_periodically(initial.clone(), client.clone(), id, secret);

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let join_handle = tokio::task::spawn(async move {
            tokio::select! {
                _ = shutdown_rx => Ok(()),
                other = refresh_future => Err(other)
            }
        });

        Ok(RefreshingAccessToken {
            token: initial,
            join_handle,
            shutdown: shutdown_tx,
        })
    }

    pub fn current(&self) -> AccessToken {
        self.token.deref().lock().clone()
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
