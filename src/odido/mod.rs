pub mod auth;
mod models;

use crate::config::AuthenticatedConfig;
use crate::http::{HttpError, get_json, post};
use models::{Bundle, BundlesResponse, SubscriptionsResource, SubscriptionsResponse};
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use thiserror::Error;

const ZONE_COLOR_NL: &str = "NL";
const KB_PER_MB: f64 = 1024.0;

pub struct OdidoClient {
    client: Client,
    config: AuthenticatedConfig,
    subscription_url: Option<String>,
}

pub enum IsReplenished {
    NotReplenished {
        mb_left: u32,
        mb_left_to_replenish: u32,
    },
    Replenished {
        mb_left: u32,
    },
}
#[derive(Debug, Error)]
pub enum OdidoError {
    #[error("AUTHORIZATION_TOKEN ongeldig of verlopen, vraag een nieuwe aan.")]
    Unauthorized,
    #[error("onverwachte response: {0}")]
    UnexpectedResponse(String),
    #[error(transparent)]
    Http(HttpError),
}

impl From<HttpError> for OdidoError {
    fn from(e: HttpError) -> Self {
        match e {
            HttpError::Unauthorized(_) => OdidoError::Unauthorized,
            other => OdidoError::Http(other),
        }
    }
}

impl OdidoClient {
    pub fn new(config: AuthenticatedConfig) -> Self {
        Self {
            client: Client::new(),
            config,
            subscription_url: None,
        }
    }

    fn auth_headers(&self) -> Result<HeaderMap, HttpError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&self.config.odido_user_agent)?,
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.authorization_token))?,
        );
        Ok(headers)
    }

    fn resolve_subscription_url(&mut self) -> Result<String, OdidoError> {
        if let Some(url) = &self.subscription_url {
            return Ok(url.clone());
        }

        let resource: SubscriptionsResource = get_json(
            &self.client,
            &format!(
                "{}/account/current?resourcelabel=LinkedSubscriptions",
                self.config.odido_api_url
            ),
            self.auth_headers()?,
            self.config.http_max_retries,
            self.config.http_retry_delay_step,
        )?;
        let subscriptions_url = &resource
            .resources
            .first()
            .ok_or_else(|| {
                OdidoError::UnexpectedResponse(
                    "Geen resources (LinkedSubscriptions) gevonden via Odido API.".into(),
                )
            })?
            .url;

        let response: SubscriptionsResponse = get_json(
            &self.client,
            subscriptions_url,
            self.auth_headers()?,
            self.config.http_max_retries,
            self.config.http_retry_delay_step,
        )?;

        let personal_api_uri = response
            .subscriptions
            .iter()
            .find(|subscription| subscription.msisdn == self.config.msisdn)
            .map(|subscription| subscription.subscription_url.clone())
            .ok_or_else(|| {
                OdidoError::UnexpectedResponse(
                    "Geen abonnement die opgegeven MSISDN matched.".into(),
                )
            })?;

        let url = format!("{personal_api_uri}/roamingbundles");
        self.subscription_url = Some(url.clone());
        Ok(url)
    }

    fn calculate_mb_left(bundles: &[Bundle]) -> u32 {
        bundles
            .iter()
            .filter(|bundle| bundle.zone_color == ZONE_COLOR_NL)
            .map(|bundle| bundle.remaining.value / KB_PER_MB)
            .sum::<f64>()
            .floor() as u32
    }

    fn find_alt_buying_codes(&self, bundles: &[Bundle]) {
        bundles
            .iter()
            .filter(|bundle| bundle.zone_color == ZONE_COLOR_NL)
            .filter_map(|bundle| {
                bundle
                    .buying_code
                    .as_deref()
                    .filter(|code| *code != self.config.odido_buying_code)
            })
            .for_each(|code| println!("Alternatieve BuyingCode gevonden: {code}"));
    }

    fn bundles(&mut self) -> Result<Vec<Bundle>, OdidoError> {
        let url = self.resolve_subscription_url()?;
        let response: BundlesResponse = get_json(
            &self.client,
            &url,
            self.auth_headers()?,
            self.config.http_max_retries,
            self.config.http_retry_delay_step,
        )?;

        Ok(response.bundles)
    }

    fn request_bundle(&mut self) -> Result<(), OdidoError> {
        let url = self.resolve_subscription_url()?;
        let mut headers = self.auth_headers()?;
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let body =
            serde_json::json!({ "Bundles": [{ "BuyingCode": &self.config.odido_buying_code }] })
                .to_string();
        post(&self.client, &url, headers, body)?;
        Ok(())
    }

    pub fn replenish_if_needed(&mut self) -> Result<IsReplenished, OdidoError> {
        let bundles = self.bundles()?;
        if self.config.discover_buying_code {
            self.find_alt_buying_codes(&bundles);
        }

        let mb_left = Self::calculate_mb_left(&bundles);
        if mb_left >= self.config.mb_threshold {
            return Ok(IsReplenished::NotReplenished {
                mb_left,
                mb_left_to_replenish: mb_left - self.config.mb_threshold,
            });
        }
        self.request_bundle()?;
        let mb_left_after = Self::calculate_mb_left(&self.bundles()?);
        Ok(IsReplenished::Replenished {
            mb_left: mb_left_after,
        })
    }
}

#[cfg(test)]
mod calculate_mb_left {
    use super::*;
    use crate::odido::models::Remaining;

    #[test]
    fn multiple_different_code_bundles() {
        let bundles: &[Bundle] = &[
            Bundle {
                zone_color: "NL".into(),
                remaining: Remaining { value: 2048.0 },
                buying_code: None,
            },
            Bundle {
                zone_color: "DE".into(),
                remaining: Remaining { value: 4096.0 },
                buying_code: None,
            },
        ];

        assert_eq!(OdidoClient::calculate_mb_left(bundles), 2);
    }

    #[test]
    fn multiple_same_code_bundles() {
        let bundles: &[Bundle] = &[
            Bundle {
                zone_color: "NL".into(),
                remaining: Remaining { value: 2048.0 },
                buying_code: None,
            },
            Bundle {
                zone_color: "NL".into(),
                remaining: Remaining { value: 4096.0 },
                buying_code: None,
            },
        ];

        assert_eq!(OdidoClient::calculate_mb_left(bundles), 6);
    }

    #[test]
    fn mb_rounds_down() {
        let bundles: &[Bundle] = &[Bundle {
            zone_color: "NL".into(),
            remaining: Remaining { value: 1023.0 },
            buying_code: None,
        }];

        assert_eq!(OdidoClient::calculate_mb_left(bundles), 0);
    }

    #[cfg(test)]
    mod odido_error {
        use super::*;

        #[test]
        fn unauthorized_maps_to_domain() {
            let error = OdidoError::from(HttpError::Unauthorized(String::new()));
            assert!(matches!(error, OdidoError::Unauthorized));
        }
    }
}
