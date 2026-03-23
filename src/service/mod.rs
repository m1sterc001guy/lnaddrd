pub mod direct;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use lnurl::pay::PayResponse;
use serde::{Deserialize, Serialize};

use crate::repository::DestinationPaymentAddress;

pub type LnaddrService = Arc<dyn ILnaddrService + Send + Sync>;

#[async_trait]
pub trait ILnaddrService {
    async fn list_domains(&self) -> Result<Vec<String>>;

    async fn get_lnaddr_manifest(
        &self,
        domain: &str,
        username: &str,
    ) -> Result<Option<PayResponse>>;

    async fn get_destination(
        &self,
        domain: &str,
        username: &str,
    ) -> Result<Option<DestinationPaymentAddress>>;

    async fn register_lnaddr(
        &self,
        domain: &str,
        username: &str,
        destination: &str,
        recipient_pk: Option<&str>,
    ) -> Result<RegisterResponse>;

    async fn remove_lnaddr(
        &self,
        domain: &str,
        username: &str,
        authentication_token: &str,
    ) -> Result<()>;

    async fn reverse_lookup_by_recipient_pk(
        &self,
        recipient_pk: &str,
    ) -> Result<Option<ReverseLookupResponse>>;

    async fn create_challenge(&self, recipient_pk: &str) -> Result<ChallengeResponse>;

    async fn reclaim_lnaddr(
        &self,
        recipient_pk: &str,
        challenge: &str,
        signature: &str,
    ) -> Result<ReclaimResponse>;

    async fn update_recipient_pk(
        &self,
        domain: &str,
        username: &str,
        authentication_token: &str,
        recipient_pk: &str,
    ) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub lnaddr: String,
    pub authentication_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseLookupResponse {
    pub username: String,
    pub domain: String,
    pub lnurl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReclaimResponse {
    pub username: String,
    pub domain: String,
    pub authentication_token: String,
}
