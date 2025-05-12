use std::sync::Arc;

use super::{ILnaddrService, LnaddrService, RegisterResponse};
use crate::repository::PaymentAddressRepository;
use anyhow::{Result, bail};
use async_trait::async_trait;
use lnurl::{LnUrlResponse, lnurl::LnUrl, pay::PayResponse};
use rand::distributions::DistString;

pub struct DirectLnaddrService {
    repo: PaymentAddressRepository,
    domains: Vec<String>,
    client: lnurl::AsyncClient,
}

impl DirectLnaddrService {
    pub fn new(repo: PaymentAddressRepository, domains: Vec<String>) -> Self {
        Self {
            repo,
            domains,
            client: lnurl::AsyncClient::from_client(reqwest::Client::new()),
        }
    }

    pub fn into_dyn(self) -> LnaddrService {
        Arc::new(self)
    }
}

#[async_trait]
impl ILnaddrService for DirectLnaddrService {
    async fn list_domains(&self) -> Result<Vec<String>> {
        Ok(self.domains.clone())
    }

    async fn get_lnaddr_manifest(
        &self,
        domain: &str,
        username: &str,
    ) -> Result<Option<PayResponse>> {
        let Some(lnaddr_entry) = self.repo.get_payment_address(domain, username).await? else {
            return Ok(None);
        };

        let response = match self.client.make_request(&lnaddr_entry.lnurl.url).await? {
            LnUrlResponse::LnUrlPayResponse(response) => response,
            LnUrlResponse::LnUrlWithdrawResponse(_) => bail!("Invalid LNURL type: LNURLwithdraw"),
            LnUrlResponse::LnUrlChannelResponse(_) => bail!("Invalid LNURL type: LNURLchannel"),
        };

        Ok(Some(response))
    }

    async fn get_lnaddr(&self, domain: &str, username: &str) -> Result<Option<LnUrl>> {
        let Some(lnaddr_entry) = self.repo.get_payment_address(domain, username).await? else {
            return Ok(None);
        };

        Ok(Some(lnaddr_entry.lnurl))
    }

    async fn register_lnaddr(
        &self,
        domain: &str,
        username: &str,
        lnurl: &str,
    ) -> Result<RegisterResponse> {
        if !self.domains.contains(&domain.to_string()) {
            bail!("Unsupported domain: {}", domain);
        }

        // Test if the lnurl is valid
        LnUrl::decode(lnurl.to_owned())?;

        let authentication_token =
            rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 20);
        self.repo
            .add_payment_address(domain, username, lnurl, &authentication_token)
            .await?;

        Ok(RegisterResponse {
            lnaddr: format!("{}@{}", username, domain),
            authentication_token,
        })
    }
}
