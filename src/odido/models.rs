use serde::Deserialize;

#[derive(Deserialize)]
pub struct Resource {
    #[serde(rename = "Url")]
    pub url: String,
}

#[derive(Deserialize)]
pub struct SubscriptionsResource {
    #[serde(rename = "Resources")]
    pub resources: Vec<Resource>,
}

#[derive(Deserialize)]
pub struct Subscription {
    #[serde(rename = "MSISDN")]
    pub msisdn: String,
    #[serde(rename = "SubscriptionURL")]
    pub subscription_url: String,
}

#[derive(Deserialize)]
pub struct SubscriptionsResponse {
    pub subscriptions: Vec<Subscription>,
}

#[derive(Deserialize)]
pub struct Bundle {
    #[serde(rename = "ZoneColor")]
    pub zone_color: String,
    #[serde(rename = "Remaining")]
    pub remaining: Remaining,
}

#[derive(Deserialize)]
pub struct Remaining {
    #[serde(rename = "Value")]
    pub value: f64,
}

#[derive(Deserialize)]
pub struct BundlesResponse {
    #[serde(rename = "Bundles")]
    pub bundles: Vec<Bundle>,
}
