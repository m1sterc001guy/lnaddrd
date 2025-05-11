pub mod pg;

use std::{sync::Arc, time::SystemTime};

use anyhow::Result;
use async_trait::async_trait;

pub type PaymentAddressRepository = Arc<dyn IPaymentAddressRepository + Send + Sync>;

#[async_trait]
pub trait IPaymentAddressRepository {
    async fn get_payment_address(
        &self,
        domain: &str,
        username: &str,
    ) -> Result<Option<PaymentAddress>>;

    async fn add_payment_address(
        &self,
        domain: &str,
        username: &str,
        lnurl: &str,
        authentication_token: &str,
    ) -> Result<()>;
}

pub struct PaymentAddress {
    pub username: String,
    pub domain: String,
    pub lnurl: lnurl::lnurl::LnUrl,
    pub authentication_token: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}
