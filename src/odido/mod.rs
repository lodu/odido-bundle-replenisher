pub mod auth;
mod models;

use crate::config::AuthenticatedConfig;
use crate::http::{HttpError, get_json, post};
use models::{Bundle, BundlesResponse, SubscriptionsResource, SubscriptionsResponse};
use nyquest::blocking::Body;
use nyquest::{BlockingClient, ClientBuilder};

const ZONE_COLOR_NL: &str = "NL";
const KB_PER_MB: f64 = 1024.0;

pub struct OdidoClient {
    http_client: BlockingClient,
    config: AuthenticatedConfig,
    subscription_url: Option<String>,
}

enum DiscoverAltCodes {
    Yes,
    No,
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

impl OdidoClient {
    pub fn new(config: AuthenticatedConfig) -> Self {
        let http_client = ClientBuilder::default()
            .user_agent(&config.odido_user_agent)
            .with_header(
                "Authorization",
                format!("Bearer {}", config.authorization_token),
            )
            .with_header("Accept", "application/json")
            .build_blocking()
            .expect("Kan HTTP-client niet bouwen.");

        Self {
            http_client,
            config,
            subscription_url: None,
        }
    }

    fn resolve_subscription_url(&mut self) -> Result<String, HttpError> {
        if let Some(url) = &self.subscription_url {
            return Ok(url.clone());
        }

        let resource: SubscriptionsResource = get_json(
            &self.http_client,
            &format!(
                "{}/account/current?resourcelabel=LinkedSubscriptions",
                self.config.odido_api_url
            ),
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
            &self.http_client,
            subscriptions_url,
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

    fn mbs_left(&mut self, discover_buying_code: DiscoverAltCodes) -> Result<u32, HttpError> {
        let url = self.resolve_subscription_url()?;
        let response: BundlesResponse = get_json(
            &self.http_client,
            &url,
            self.config.http_max_retries,
            self.config.http_retry_delay_step,
        )?;

        if self.config.discover_buying_code && matches!(discover_buying_code, DiscoverAltCodes::Yes)
        {
            self.find_alt_buying_codes(&response.bundles);
        }

        Ok(Self::calculate_mb_left(&response.bundles))
    }

    fn request_bundle(&mut self) -> Result<(), HttpError> {
        let url = self.resolve_subscription_url()?;

        let body = Body::text(
            serde_json::json!({ "Bundles": [{ "BuyingCode": &self.config.odido_buying_code }] })
                .to_string(),
            "application/json",
        );
        post(&self.http_client, &url, body)?;
        Ok(())
    }

    pub fn replenish_if_needed(&mut self) -> Result<IsReplenished, HttpError> {
        let mb_left = self.mbs_left(DiscoverAltCodes::Yes)?;
        if mb_left >= self.config.mb_threshold {
            return Ok(IsReplenished::NotReplenished {
                mb_left,
                mb_left_to_replenish: mb_left - self.config.mb_threshold,
            });
        }
        self.request_bundle()?;
        let mb_left_after: u32 = self.mbs_left(DiscoverAltCodes::No)?;
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
}
