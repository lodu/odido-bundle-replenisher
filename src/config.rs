use anyhow::{Context, Result};
use fernet::Fernet;
use serde::Deserialize;
use std::time::Duration;

pub enum StartupConfig {
    Authenticated(Config),
    OAuthRequired(OAuthConfig),
    LoginRequired(LoginConfig),
}

pub struct OAuthConfig {
    pub odido_api_url: String,
    pub refresh_token: String,
    pub odido_oauth_key: String,
}

pub struct LoginConfig {
    pub odido_url: String,
    pub odido_api_url: String,
    pub odido_fernet_key: String,
    pub odido_oauth_key: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LoginResponse {
    access_token: String,
}

impl LoginConfig {
    pub fn login_url(&self) -> Result<String> {
        let fernet = Fernet::new(&self.odido_fernet_key).unwrap();
        let encrypted_oauth_key = fernet.encrypt(self.odido_oauth_key.as_bytes());

        Ok(format!(
            "{url}/login?returnSystem=app&nav=off&token={token}",
            url = self.odido_url,
            token = encrypted_oauth_key
        ))
    }

    pub fn refresh_token(&self, encrypted_login_response: &str) -> Result<String> {
        let fernet = Fernet::new(&self.odido_fernet_key).unwrap();
        let login_response: LoginResponse =
            serde_json::from_slice(&fernet.decrypt(encrypted_login_response).unwrap()).unwrap();

        String::from_utf8(fernet.decrypt(&login_response.access_token).unwrap())
            .context("Odido authorization code is geen geldige UTF-8")
    }
}

pub struct Config {
    pub authorization_token: String,
    pub msisdn: String,
    pub check_interval: Duration,
    pub odido_api_url: String,
    pub odido_user_agent: String,
    pub odido_buying_code: String,
    pub mb_threshold: u32,
    pub http_max_retries: u32,
    pub http_retry_delay_step: u32,
}

impl Config {
    pub fn from_env() -> Result<StartupConfig> {
        let odido_url: String =
            std::env::var("ODIDO_URL").unwrap_or_else(|_| "https://odido.nl".to_owned());

        let odido_fernet_key: String = std::env::var("ODIDO_FERNET_KEY")
            .unwrap_or_else(|_| "afIqRZm6iSev4zWysNGAjR6fCrOMf5GQqhKFfmXkgOU".to_owned());
        let odido_oauth_key: String =
            std::env::var("ODIDO_OAUTH_KEY").unwrap_or_else(|_| "9havvat6hm0b962i".to_owned());

        let odido_api_url: String =
            std::env::var("ODIDO_API_URL").unwrap_or_else(|_| "https://capi.odido.nl".to_owned());

        let authorization_token = match std::env::var("AUTHORIZATION_TOKEN") {
            Ok(token) => token,
            Err(std::env::VarError::NotPresent) => match std::env::var("REFRESH_TOKEN") {
                Ok(refresh_token) => {
                    return Ok(StartupConfig::OAuthRequired(OAuthConfig {
                        odido_api_url,
                        refresh_token,
                        odido_oauth_key,
                    }));
                }
                Err(std::env::VarError::NotPresent) => {
                    return Ok(StartupConfig::LoginRequired(LoginConfig {
                        odido_url,
                        odido_api_url,
                        odido_fernet_key,
                        odido_oauth_key,
                    }));
                }
                Err(error) => {
                    return Err(error).context("REFRESH_TOKEN kon niet worden gelezen");
                }
            },
            Err(error) => return Err(error).context("AUTHORIZATION_TOKEN kon niet worden gelezen"),
        };

        let msisdn = std::env::var("MSISDN").context("MSISDN niet gevonden in env")?;

        let minutes: u64 = std::env::var("CHECK_INTERVAL")
            .ok()
            .and_then(|s: String| s.parse().ok())
            .unwrap_or(5);

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

        Ok(StartupConfig::Authenticated(Config {
            authorization_token,
            msisdn,
            check_interval: Duration::from_secs(minutes * 60),
            odido_api_url,
            odido_user_agent,
            odido_buying_code,
            mb_threshold,
            http_max_retries,
            http_retry_delay_step,
        }))
    }
}
