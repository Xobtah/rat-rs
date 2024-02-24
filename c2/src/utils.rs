use crate::dao;
use crate::entities::Agent;
use crate::error::C2Error;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub type C2HttpResponse = Result<actix_web::HttpResponse, C2Error>;

#[derive(Serialize, Deserialize)]
pub struct JwtClaim {
    agent_id: i32,
    exp: i64,
}

impl JwtClaim {
    pub fn new(agent_id: i32, timeout_in_minutes: i64) -> Self {
        Self {
            agent_id,
            exp: (chrono::Utc::now() + chrono::Duration::minutes(timeout_in_minutes)).timestamp(),
        }
    }

    pub fn encode(&self, jwt_signing_key: &str) -> Result<String> {
        Ok(jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            self,
            &jsonwebtoken::EncodingKey::from_secret(jwt_signing_key.as_bytes()),
        )?)
    }

    // pub fn is_expired(&self) -> bool {
    //     let now = chrono::Utc::now().timestamp();
    //     let exp = self.exp as i64;
    //     now > exp
    // }
}

// pub async fn begin_sqlx_transaction(
//     db_pool: &sqlx::SqlitePool,
// ) -> Result<sqlx::Transaction<'_, sqlx::Sqlite>> {
//     db_pool
//         .begin()
//         .await
//         .map_err(|e| anyhow!("Failed to begin transaction : {}", e))
// }
//
// pub async fn commit_sqlx_transaction(tx: sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<()> {
//     tx.commit()
//         .await
//         .map_err(|e| anyhow!("Failed to commit transaction : {}", e))
// }

fn bearer_to_claim(header: &str, jwt_signing_key: &str) -> Result<JwtClaim> {
    let words: Vec<&str> = header.split_whitespace().collect();
    if words.len() == 2 {
        jsonwebtoken::decode::<JwtClaim>(
            words[1],
            &jsonwebtoken::DecodingKey::from_secret(jwt_signing_key.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| anyhow!("JWT decoding error: {}", e))
    } else {
        Err(anyhow!(
            "Could not extract claim from JWT - Header: {:?}",
            words
        ))
    }
}

fn get_jwt(req: &actix_web::HttpRequest, jwt_signing_key: &str) -> Result<JwtClaim> {
    req.headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .ok_or(anyhow!("Cannot find JWT header"))
        .and_then(|header_value| {
            header_value
                .to_str()
                .map_err(|_| anyhow!("Can not decode header to string"))
                .and_then(|s| {
                    bearer_to_claim(s, jwt_signing_key)
                        .map_err(|e| anyhow!("Failed to decode JWT: {}", e))
                })
        })
}

pub async fn authenticate_jwt(
    req: &actix_web::HttpRequest,
    db_pool: &sqlx::SqlitePool,
    jwt_signing_key: &str,
    // validate_user: impl Fn(&api_model::User) -> bool,
) -> Result<Agent> {
    {
        // Decode JWT
        let claim = get_jwt(req, jwt_signing_key)?;
        // Is JWT expired
        // if claim.is_expired() {
        //     warn!("JWT expired");
        //     return Err(anyhow!("JWT expired"));
        // }
        // Check user rights
        if let Some(agent) = dao::agent::get_by_id(db_pool, claim.agent_id).await? {
            dao::agent::update_last_seen_at(db_pool, &agent).await?;
            Ok(agent)
        } else {
            Err(anyhow!("Agent not found"))
        }
    }
    .map_err(|_: anyhow::Error| anyhow!(C2Error::Unauthorized))
}

pub fn get_user_agent(req: &actix_web::HttpRequest) -> Result<String> {
    req.headers()
        .get(actix_web::http::header::USER_AGENT)
        .ok_or(anyhow!("Cannot find User-Agent header"))
        .and_then(|header_value| {
            header_value
                .to_str()
                .map_err(|_| anyhow!("Can not decode header to string"))
                .map(String::from)
        })
}

pub fn db_model_to_entity_o<Entity, DbModel: TryInto<Entity>>(
    db_model_o: Option<DbModel>,
) -> Result<Option<Entity>> {
    db_model_o
        .map(|db_model| db_model.try_into())
        .transpose()
        .map_err(|_| anyhow!("Failed to convert DB model to entity"))
}
