use crate::config::{AuthorizationCodeConfig, LoginConfig};
use crate::http::post;
use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use fernet::Fernet;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LoginResponse {
    access_token: String,
}

impl LoginConfig {
    pub fn login_url(&self) -> Result<String> {
        let fernet = Fernet::new(&self.odido_fernet_key).context("Ongeldige odido_fernet_key")?;
        let encrypted_oauth_key = fernet.encrypt(self.odido_oauth_key.as_bytes());

        Ok(format!(
            "{url}/login?returnSystem=app&nav=off&token={token}",
            url = self.odido_url,
            token = encrypted_oauth_key
        ))
    }

    pub fn extract_authorization_code(&self, url: &str) -> Result<String> {
        let url = reqwest::Url::parse(url).context("Ingevoerde URL is geen URL.")?;
        let token = url
            .query_pairs()
            .find_map(|(name, value)| (name == "token").then_some(value))
            .context("Ingevoerde URL bevat geen token.")?;

        let fernet = Fernet::new(&self.odido_fernet_key).context("Ongeldige odido_fernet_key")?;
        let login_response: LoginResponse = serde_json::from_slice(
            &fernet
                .decrypt(&token)
                .context("Odido-loginresponse kon niet worden decrypt.")?,
        )
        .context("Decrypte Odido loginresponse bevat geen valide JSON.")?;

        String::from_utf8(
            fernet
                .decrypt(&login_response.access_token)
                .context("Odido authorization code kon niet worden decrypt.")?,
        )
        .context("Odido authorization code is geen valide UTF-8.")
    }
}

pub fn create_authorization_token(config: AuthorizationCodeConfig) -> Result<String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static(
            "application/json,application/vnd.capi.tmobile.nl.createtoken.v1+json",
        ),
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.capi.tmobile.nl.createtoken.v1+json"),
    );
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!(
            "Basic {}",
            general_purpose::STANDARD.encode(format!("{}:", config.odido_oauth_key).as_bytes())
        ))?,
    );
    headers.insert(
        HeaderName::from_static("grant_type"),
        HeaderValue::from_static("authorization_code"),
    );

    let response = post(
        &Client::new(),
        &format!("{}/createtoken", config.odido_api_url),
        headers,
        serde_json::json!({ "AuthorizationCode": config.authorization_code }).to_string(),
    )?;

    if let Some(error) = response.headers().get("ErrorText") {
        anyhow::bail!(
            "Odido kon geen authorization token maken. Let op dat de REFRESH_TOKEN tijdgevoelig is: {}",
            error.to_str()?
        );
    }

    response
        .headers()
        .get("Accesstoken")
        .context("Odido gaf geen AUTHORIZATION_TOKEN terug")?
        .to_str()
        .context("Odido gaf een ongeldige AUTHORIZATION_TOKEN terug")
        .map(str::to_owned)
}
