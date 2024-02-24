use crate::error::RatError;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use common::utils::shake_your_hash;
use common::{Message, Notification, UserAgent};
use reqwest::header::AUTHORIZATION;
use reqwest::Url;

#[async_trait]
pub trait ConnectorStrategy {
    async fn connect(&mut self) -> Result<()>;
    async fn send(&mut self, message: &Notification) -> Result<Message, RatError>;
    // async fn receive(&mut self) -> Result<Message>;
    async fn disconnect(&mut self) -> Result<()>;
}

pub struct HttpsConnector {
    url: Url,
    client: reqwest::Client,
    jwt: String,
}

impl HttpsConnector {
    pub fn new(url: String) -> Result<Self> {
        Ok(Self {
            url: Url::parse(&url)?,
            client: reqwest::ClientBuilder::new()
                .https_only(true)
                .danger_accept_invalid_certs(true)
                .user_agent(
                    UserAgent {
                        name: env!("CARGO_PKG_NAME").to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        os: std::env::consts::OS.to_string(),
                        hash: shake_your_hash()?,
                    }
                    .to_string(),
                )
                // .proxy()
                .build()?,
            jwt: String::new(),
        })
    }
}

#[async_trait]
impl ConnectorStrategy for HttpsConnector {
    async fn connect(&mut self) -> Result<()> {
        let mac = mac_address::get_mac_address()?
            .ok_or_else(|| anyhow::anyhow!("No MAC address"))?
            .bytes()
            .to_vec();
        Ok(self
            .client
            .put(self.url.as_str())
            .header(AUTHORIZATION, self.jwt.clone())
            .body(mac)
            .send()
            .await
            .map(|response| {
                response.headers().get(AUTHORIZATION).map(|header| {
                    self.jwt = header.to_str().unwrap().to_string();
                });
                ()
            })?)
    }

    async fn send(&mut self, notification: &Notification) -> Result<Message, RatError> {
        let response = self
            .client
            .post(self.url.as_str())
            .header(AUTHORIZATION, self.jwt.clone())
            .json(&notification)
            .send()
            .await
            .map_err(|e| RatError::Error(anyhow!(e)))?;

        response.headers().get(AUTHORIZATION).map(|header| {
            self.jwt = header.to_str().unwrap().to_string();
        });

        if response.status() == reqwest::StatusCode::OK {
            Ok(response
                .json()
                .await
                .map_err(|e| RatError::Error(anyhow!(e)))?)
        } else if response.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(Message::BananaBread)
        } else if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            Err(RatError::Unauthorized)
        } else {
            Err(RatError::Error(anyhow!(
                "Unexpected status code: {}",
                response.status()
            )))
        }
    }

    // async fn receive(&mut self) -> Result<Message> {
    //     let response = self
    //         .client
    //         .get(self.url.join("zadacha")?)
    //         .header(AUTHORIZATION, self.jwt.clone())
    //         .send()
    //         .await?;
    //
    //     if response.status() == reqwest::StatusCode::NO_CONTENT {
    //         Ok(Message::BananaBread)
    //     } else if response.status() == reqwest::StatusCode::OK {
    //         Ok(response.json().await?)
    //     } else {
    //         Err(anyhow::anyhow!(
    //             "Unexpected status code: {}",
    //             response.status()
    //         ))
    //     }
    // }

    async fn disconnect(&mut self) -> Result<()> {
        unimplemented!()
    }
}

// pub struct TcpConnector {
//     url: String,
// }
//
// impl TcpConnector {
//     pub fn new(url: String) -> Self {
//         Self { url }
//     }
// }
//
// impl ConnectorStrategy for TcpConnector {
//     async fn connect(&mut self) {
//         unimplemented!()
//     }
//
//     async fn send(&mut self, _message: Message) -> Notification {
//         unimplemented!()
//     }
//
//     async fn receive(&mut self) -> Notification {
//         unimplemented!()
//     }
//
//     async fn disconnect(&mut self) {
//         unimplemented!()
//     }
// }

pub struct Connector {
    strategy: Box<dyn ConnectorStrategy>,
}

impl Connector {
    pub fn new(strategy: Box<dyn ConnectorStrategy>) -> Self {
        Self { strategy }
    }

    pub async fn connect(&mut self) -> Result<()> {
        self.strategy.connect().await
    }

    pub async fn send(&mut self, notification: &Notification) -> Result<Message, RatError> {
        self.strategy.send(notification).await
    }

    // pub async fn receive(&mut self) -> Result<Message> {
    //     self.strategy.receive().await
    // }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.strategy.disconnect().await
    }
}
