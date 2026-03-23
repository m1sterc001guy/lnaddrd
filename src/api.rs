use anyhow::Result;
use axum::extract::{Host, Path};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::AppState;
use crate::service::{ChallengeResponse, ReclaimResponse, RegisterResponse, ReverseLookupResponse};

pub async fn list_domains_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, axum::http::StatusCode> {
    state
        .service
        .list_domains()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}

pub async fn get_lnaddr_manifest_handler(
    State(state): State<AppState>,
    Host(domain): Host,
    Path(username): Path<String>,
) -> Result<Json<lnurl::pay::PayResponse>, axum::http::StatusCode> {
    state
        .service
        .get_lnaddr_manifest(&domain, &username)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)
        .map(Json)
}

pub async fn get_lnaddr_handler(
    State(state): State<AppState>,
    Path((domain, username)): Path<(String, String)>,
) -> Result<Json<Value>, axum::http::StatusCode> {
    state
        .service
        .get_destination(&domain, &username)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)
        .map(|d| Json(json!({ "url": d.url() })))
}

pub async fn register_lnaddr_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, axum::http::StatusCode> {
    state
        .service
        .register_lnaddr(
            &payload.domain,
            &payload.username,
            &payload.lnurl,
            payload.recipient_pk.as_deref(),
        )
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)
        .map(Json)
}

pub async fn remove_lnaddr_handler(
    State(state): State<AppState>,
    Json(payload): Json<RemoveRequest>,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    state
        .service
        .remove_lnaddr(
            &payload.domain,
            &payload.username,
            &payload.authentication_token,
        )
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn reverse_lookup_handler(
    State(state): State<AppState>,
    Query(params): Query<ReverseLookupQuery>,
) -> Result<Json<ReverseLookupResponse>, axum::http::StatusCode> {
    state
        .service
        .reverse_lookup_by_recipient_pk(&params.recipient_pk)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)
        .map(Json)
}

pub async fn challenge_handler(
    State(state): State<AppState>,
    Query(params): Query<ChallengeQuery>,
) -> Result<Json<ChallengeResponse>, axum::http::StatusCode> {
    state
        .service
        .create_challenge(&params.recipient_pk)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)
        .map(Json)
}

pub async fn reclaim_lnaddr_handler(
    State(state): State<AppState>,
    Json(payload): Json<ReclaimRequest>,
) -> Result<Json<ReclaimResponse>, axum::http::StatusCode> {
    state
        .service
        .reclaim_lnaddr(
            &payload.recipient_pk,
            &payload.challenge,
            &payload.signature,
        )
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)
        .map(Json)
}

pub async fn update_pk_handler(
    State(state): State<AppState>,
    Json(payload): Json<UpdatePkRequest>,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    state
        .service
        .update_recipient_pk(
            &payload.domain,
            &payload.username,
            &payload.authentication_token,
            &payload.recipient_pk,
        )
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub domain: String,
    pub username: String,
    pub lnurl: String,
    pub recipient_pk: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RemoveRequest {
    pub domain: String,
    pub username: String,
    pub authentication_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReverseLookupQuery {
    pub recipient_pk: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChallengeQuery {
    pub recipient_pk: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReclaimRequest {
    pub recipient_pk: String,
    pub challenge: String,
    pub signature: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePkRequest {
    pub domain: String,
    pub username: String,
    pub authentication_token: String,
    pub recipient_pk: String,
}
