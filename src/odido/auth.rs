use crate::config::{AuthorizationCodeConfig, LoginConfig};
use crate::http::post;
use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use fernet::Fernet;
use nyquest::ClientBuilder;
use nyquest::blocking::Body;
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

    pub fn authorization_code(&self, encrypted_login_response: &str) -> Result<String> {
        let fernet = Fernet::new(&self.odido_fernet_key).context("Ongeldige odido_fernet_key")?;
        let login_response: LoginResponse = serde_json::from_slice(
            &fernet
                .decrypt(encrypted_login_response)
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
    let auth_http_client = ClientBuilder::default()
        .with_header(
            "Accept",
            "application/json,application/vnd.capi.tmobile.nl.createtoken.v1+json",
        )
        .with_header(
            "Authorization",
            format!(
                "Basic {}",
                general_purpose::STANDARD.encode(format!("{}:", config.odido_oauth_key).as_bytes())
            ),
        )
        .with_header("grant_type", "authorization_code")
        .build_blocking()?;

    let body = Body::text(
        serde_json::json!({ "AuthorizationCode": config.authorization_code }).to_string(),
        "application/vnd.capi.tmobile.nl.createtoken.v1+json",
    );

    let response = post(
        &auth_http_client,
        &format!("{}/createtoken", config.odido_api_url),
        body,
    )?;

    let error_header = response.get_header("ErrorText")?;

    if !error_header.is_empty() {
        anyhow::bail!(
            "Odido kon geen authorization token maken. Let op dat de REFRESH_TOKEN tijdgevoelig is: {}",
            error_header.join(", ")
        );
    }

    response
        .get_header("Accesstoken")
        .context("Odido gaf geen AUTHORIZATION_TOKEN terug")?
        .first()
        .cloned()
        .context("Odido gaf een ongeldige AUTHORIZATION_TOKEN terug")
}
