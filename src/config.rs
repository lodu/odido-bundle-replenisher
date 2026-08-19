use anyhow::{Context, Result};
use std::time::Duration;

pub struct Config {
    pub authorization_token: String,
    pub msisdn: String,
    pub update_interval: Duration,
    pub odido_api_url: String,
    pub odido_user_agent: String,
    pub odido_buying_code: String,
    pub mb_threshold: u32,
    pub http_max_retries: u32,
    pub http_retry_delay_step: u32,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let authorization_token = std::env::var("AUTHORIZATIONTOKEN")
            .context("AUTHORIZATIONTOKEN niet gevonden in env")?;
        let msisdn = std::env::var("MSISDN").context("MSISDN niet gevonden in env")?;

        let minutes: u64 = std::env::var("UPDATE_INTERVAL")
            .ok()
            .and_then(|s: String| s.parse().ok())
            .unwrap_or(5);

        let odido_api_url: String =
            std::env::var("ODIDO_API_URL").unwrap_or_else(|_| "https://capi.odido.nl".to_owned());

        let odido_user_agent: String = std::env::var("ODIDO_USER_AGENT")
            .unwrap_or_else(|_| "ODIDO 8.0.0 (Android 12; 12)".to_owned());
        let odido_buying_code: String =
            std::env::var("ODIDO_BUYING_CODE").unwrap_or_else(|_| "A0DAY01".to_owned());

        let mb_threshold = std::env::var("MB_THRESHOLD")
            .ok()
            .and_then(|s: String| s.parse().ok())
            .unwrap_or(2000);

        let http_max_retries = std::env::var("HTTP_MAX_RETRIES")
            .ok()
            .and_then(|s: String| s.parse().ok())
            .unwrap_or(10);

        let http_retry_delay_step = std::env::var("HTTP_RETRY_DELAY_STEP")
            .ok()
            .and_then(|s: String| s.parse().ok())
            .unwrap_or(10);

        Ok(Config {
            authorization_token,
            msisdn,
            update_interval: Duration::from_secs(minutes * 60),
            odido_api_url,
            odido_user_agent,
            odido_buying_code,
            mb_threshold,
            http_max_retries,
            http_retry_delay_step,
        })
    }
}
