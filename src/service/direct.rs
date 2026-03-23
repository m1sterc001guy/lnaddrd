use std::{str::FromStr, sync::Arc, time::Instant};

use super::{
    ChallengeResponse, ILnaddrService, LnaddrService, ReclaimResponse, RegisterResponse,
    ReverseLookupResponse,
};
use crate::repository::{DestinationPaymentAddress, PaymentAddressRepository};
use anyhow::{Result, bail};
use async_trait::async_trait;
use dashmap::DashMap;
use lnurl::{LnUrlResponse, pay::PayResponse};
use rand::distributions::DistString;
use secp256k1::{Message, PublicKey, Secp256k1, ecdsa::Signature};
use sha2::{Digest, Sha256};

const CHALLENGE_TTL_SECS: u64 = 60;

pub struct DirectLnaddrService {
    repo: PaymentAddressRepository,
    domains: Vec<String>,
    client: lnurl::AsyncClient,
    /// Maps challenge hex -> (recipient_pk hex, created_at)
    challenges: Arc<DashMap<String, (String, Instant)>>,
}

impl DirectLnaddrService {
    pub fn new(repo: PaymentAddressRepository, domains: Vec<String>) -> Self {
        Self {
            repo,
            domains,
            client: lnurl::AsyncClient::from_client(reqwest::Client::new()),
            challenges: Arc::new(DashMap::new()),
        }
    }

    pub fn into_dyn(self) -> LnaddrService {
        Arc::new(self)
    }

    fn cleanup_expired_challenges(&self) {
        let now = Instant::now();
        self.challenges
            .retain(|_, (_, created)| now.duration_since(*created).as_secs() < CHALLENGE_TTL_SECS);
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

        let response = match self
            .client
            .make_request(&lnaddr_entry.destination.url())
            .await?
        {
            LnUrlResponse::LnUrlPayResponse(response) => response,
            LnUrlResponse::LnUrlWithdrawResponse(_) => bail!("Invalid LNURL type: LNURLwithdraw"),
            LnUrlResponse::LnUrlChannelResponse(_) => bail!("Invalid LNURL type: LNURLchannel"),
        };

        Ok(Some(response))
    }

    async fn get_destination(
        &self,
        domain: &str,
        username: &str,
    ) -> Result<Option<DestinationPaymentAddress>> {
        let Some(lnaddr_entry) = self.repo.get_payment_address(domain, username).await? else {
            return Ok(None);
        };

        Ok(Some(lnaddr_entry.destination))
    }

    async fn register_lnaddr(
        &self,
        domain: &str,
        username: &str,
        destination: &str,
        recipient_pk: Option<&str>,
    ) -> Result<RegisterResponse> {
        if !self.domains.contains(&domain.to_string()) {
            bail!("Unsupported domain: {}", domain);
        }

        // Test if the lnurl is valid
        let destination = DestinationPaymentAddress::from_str(destination)?;

        let authentication_token =
            rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 20);
        self.repo
            .add_payment_address(
                domain,
                username,
                destination,
                &authentication_token,
                recipient_pk,
            )
            .await?;

        Ok(RegisterResponse {
            lnaddr: format!("{}@{}", username, domain),
            authentication_token,
        })
    }

    async fn remove_lnaddr(
        &self,
        domain: &str,
        username: &str,
        authentication_token: &str,
    ) -> Result<()> {
        self.repo
            .remove_payment_address(domain, username, authentication_token)
            .await
    }

    async fn reverse_lookup_by_recipient_pk(
        &self,
        recipient_pk: &str,
    ) -> Result<Option<ReverseLookupResponse>> {
        let Some(entry) = self
            .repo
            .get_payment_address_by_recipient_pk(recipient_pk)
            .await?
        else {
            return Ok(None);
        };

        Ok(Some(ReverseLookupResponse {
            username: entry.username,
            domain: entry.domain,
            lnurl: entry.destination.to_string(),
        }))
    }

    async fn create_challenge(&self, recipient_pk: &str) -> Result<ChallengeResponse> {
        // Verify the recipient_pk is a valid public key
        let _pk = PublicKey::from_str(recipient_pk)?;

        // Verify an address exists for this recipient_pk
        let exists = self
            .repo
            .get_payment_address_by_recipient_pk(recipient_pk)
            .await?;
        if exists.is_none() {
            bail!("No payment address found for recipient_pk");
        }

        self.cleanup_expired_challenges();

        let challenge =
            rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
        let challenge_hex = hex::encode(challenge.as_bytes());

        self.challenges.insert(
            challenge_hex.clone(),
            (recipient_pk.to_string(), Instant::now()),
        );

        let expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + CHALLENGE_TTL_SECS;

        Ok(ChallengeResponse {
            challenge: challenge_hex,
            expires_at,
        })
    }

    async fn reclaim_lnaddr(
        &self,
        recipient_pk: &str,
        challenge: &str,
        signature: &str,
    ) -> Result<ReclaimResponse> {
        // Look up and validate challenge
        let (stored_pk, created) = self
            .challenges
            .remove(challenge)
            .map(|(_, v)| v)
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired challenge"))?;

        if Instant::now().duration_since(created).as_secs() >= CHALLENGE_TTL_SECS {
            bail!("Challenge expired");
        }

        if stored_pk != recipient_pk {
            bail!("Challenge was not issued for this recipient_pk");
        }

        // Verify signature: ECDSA signature of SHA256(challenge) by recipient_pk
        let secp = Secp256k1::verification_only();
        let pk = PublicKey::from_str(recipient_pk)?;
        let sig = Signature::from_compact(&hex::decode(signature)?)?;
        let challenge_hash = Sha256::digest(hex::decode(challenge)?);
        let msg = Message::from_digest(challenge_hash.into());
        secp.verify_ecdsa(&msg, &sig, &pk)?;

        // Look up the address
        let entry = self
            .repo
            .get_payment_address_by_recipient_pk(recipient_pk)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No payment address found for recipient_pk"))?;

        // Generate new auth token and update
        let new_token =
            rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 20);
        self.repo
            .update_authentication_token(&entry.domain, &entry.username, &new_token)
            .await?;

        Ok(ReclaimResponse {
            username: entry.username,
            domain: entry.domain,
            authentication_token: new_token,
        })
    }

    async fn update_recipient_pk(
        &self,
        domain: &str,
        username: &str,
        authentication_token: &str,
        recipient_pk: &str,
    ) -> Result<()> {
        // Validate it's a valid public key
        let _pk = PublicKey::from_str(recipient_pk)?;

        self.repo
            .update_recipient_pk(domain, username, authentication_token, recipient_pk)
            .await
    }
}
