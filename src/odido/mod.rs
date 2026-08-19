pub mod auth;
mod models;

use crate::config::AuthenticatedConfig;
use crate::http::{HttpError, get_json, post_empty};
use models::{Bundle, BundlesResponse, SubscriptionsResource, SubscriptionsResponse};
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};

pub struct OdidoClient {
    client: Client,
    config: AuthenticatedConfig,
    subscription_url: Option<String>,
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

    fn resolve_subscription_url(&mut self) -> Result<String, HttpError> {
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
                HttpError::UnexpectedResponse(
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
                HttpError::UnexpectedResponse(
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
            .filter(|bundle| bundle.zone_color == "NL")
            .map(|bundle| bundle.remaining.value / 1024.0)
            .sum::<f64>()
            .floor() as u32
    }

    pub fn mbs_left(&mut self) -> Result<u32, HttpError> {
        let url = self.resolve_subscription_url()?;
        let response: BundlesResponse = get_json(
            &self.client,
            &url,
            self.auth_headers()?,
            self.config.http_max_retries,
            self.config.http_retry_delay_step,
        )?;

        Ok(Self::calculate_mb_left(&response.bundles))
    }

    pub fn request_bundle(&mut self) -> Result<(), HttpError> {
        let url = self.resolve_subscription_url()?;
        let mut headers = self.auth_headers()?;
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let body =
            serde_json::json!({ "Bundles": [{ "BuyingCode": &self.config.odido_buying_code }] })
                .to_string();
        post_empty(&self.client, &url, headers, body)?;
        println!("Success: Nieuwe bundel is aangevraagd.");
        Ok(())
    }
}
