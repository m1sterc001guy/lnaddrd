pub mod direct;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use lnurl::{lnurl::LnUrl, pay::PayResponse};
use serde::{Deserialize, Serialize};

pub type LnaddrService = Arc<dyn ILnaddrService + Send + Sync>;

#[async_trait]
pub trait ILnaddrService {
    async fn list_domains(&self) -> Result<Vec<String>>;

    async fn get_lnaddr_manifest(
        &self,
        domain: &str,
        username: &str,
    ) -> Result<Option<PayResponse>>;

    async fn get_lnaddr(&self, domain: &str, username: &str) -> Result<Option<LnUrl>>;

    async fn register_lnaddr(
        &self,
        domain: &str,
        username: &str,
        lnurl: &str,
    ) -> Result<RegisterResponse>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub lnaddr: String,
    pub authentication_token: String,
}
