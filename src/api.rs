use anyhow::Result;
use axum::{
    Json,
    extract::{Host, Path, State},
};
use serde::{Deserialize, Serialize};

use crate::service::{LnaddrService, RegisterResponse};

pub async fn list_domains_handler(
    State(service): State<LnaddrService>,
) -> Result<Json<Vec<String>>, axum::http::StatusCode> {
    service
        .list_domains()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}

pub async fn get_lnaddr_manifest_handler(
    State(service): State<LnaddrService>,
    Host(domain): Host,
    Path(username): Path<String>,
) -> Result<Json<lnurl::pay::PayResponse>, axum::http::StatusCode> {
    service
        .get_lnaddr_manifest(&domain, &username)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)
        .map(Json)
}

pub async fn get_lnaddr_handler(
    State(service): State<LnaddrService>,
    Path((domain, username)): Path<(String, String)>,
) -> Result<Json<lnurl::lnurl::LnUrl>, axum::http::StatusCode> {
    service
        .get_lnaddr(&domain, &username)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)
        .map(Json)
}

pub async fn register_lnaddr_handler(
    State(service): State<LnaddrService>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, axum::http::StatusCode> {
    service
        .register_lnaddr(&payload.domain, &payload.username, &payload.lnurl)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)
        .map(Json)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub domain: String,
    pub username: String,
    pub lnurl: String,
}
