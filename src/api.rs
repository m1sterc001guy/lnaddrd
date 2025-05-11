use anyhow::Result;
use axum::{extract::{Host, Path, State}, response::IntoResponse, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{debug, info};

use crate::{config::Config, repository::pg::PgPaymentAddressRepository, service::{direct::DirectLnaddrService, LnaddrService, RegisterResponse}};

async fn list_domains_handler(
    State(service): State<LnaddrService>,
) -> Result<Json<Vec<String>>, axum::http::StatusCode> {
    service.list_domains().await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}

async fn get_lnaddr_manifest_handler(
    State(service): State<LnaddrService>,
    Host(domain): Host,
    Path(username): Path<String>,
) -> Result<Json<lnurl::pay::PayResponse>, axum::http::StatusCode> {
    service.get_lnaddr_manifest(&domain, &username).await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)
        .map(Json)
}

async fn get_lnaddr_handler(
    State(service): State<LnaddrService>,
    Path((domain, username)): Path<(String, String)>,
) -> Result<Json<lnurl::lnurl::LnUrl>, axum::http::StatusCode> {
    service.get_lnaddr(&domain, &username).await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)
        .map(Json)
}

async fn register_lnaddr_handler(
    State(service): State<LnaddrService>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, axum::http::StatusCode> {
    service.register_lnaddr(&payload.domain, &payload.username, &payload.lnurl).await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)
        .map(Json)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub domain: String,
    pub username: String,
    pub lnurl: String,
}



pub async fn serve(config: &Config) -> Result<()> {
    debug!(db=%config.database, "Opening database connection");
    let lnaddr_repo = PgPaymentAddressRepository::new(&config.database)?.into_dyn();

    debug!(domains=?config.domains, "Starting LN address service");
    let lnaddr_service = DirectLnaddrService::new(lnaddr_repo, config.domains.clone()).into_dyn();

    let app = Router::new()
        .route("/domains", get(list_domains_handler))
        .route("/lnaddress/:domain/:username", get(get_lnaddr_handler))
        .route("/lnaddress/register", post(register_lnaddr_handler))
        .route("/.well-known/lnurlp/:username", get(get_lnaddr_manifest_handler))
        .with_state(lnaddr_service.clone())
        .fallback(|_req: axum::http::Request<axum::body::Body>| async move {
            axum::http::StatusCode::NOT_FOUND
        })
        .layer(axum::middleware::map_response(|res: axum::response::Response| async {
            if res.status().is_client_error() || res.status().is_server_error() {
                let status = res.status();
                let body = format!(
                    r#"<!DOCTYPE html>
                    <html>
                        <head><title>{} {}</title></head>
                        <body>
                            <h1>{} {}</h1>
                        </body>
                    </html>"#,
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("Unknown"),
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("Unknown")
                );
                return (status, [(axum::http::header::CONTENT_TYPE, "text/html")], body).into_response();
            }
            res
        }));

    info!(bind=%config.bind, "Starting HTTP server");
    let listener = TcpListener::bind(&config.bind).await?;
    axum::serve(listener, app).await?;

    Ok(())
}