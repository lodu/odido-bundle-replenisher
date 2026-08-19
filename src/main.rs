mod config;

mod http;
mod models;
mod odido;
use anyhow::Context;
use base64::{Engine as _, engine::general_purpose};
use config::{Config, OAuthConfig};
use odido::Odido;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::config::StartupConfig::{Authenticated, LoginRequired, OAuthRequired};

fn create_authorization_token(config: OAuthConfig) -> anyhow::Result<String> {
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

    let body = serde_json::json!({ "AuthorizationCode": config.refresh_token }).to_string();
    let response = Client::new()
        .post(format!("{}/createtoken", config.odido_api_url))
        .headers(headers)
        .body(body)
        .send()?
        .error_for_status()?;

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

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let (mut odido, interval, mb_threshold) = match Config::from_env()? {
        LoginRequired(config) => {
            println!(
                "Open deze URL om in te loggen:\n{}\n\nPlak daarna de URL hier OF herstart de app met de token (REFRESH_TOKEN).",
                config.login_url()?
            );
            io::stdout().flush().unwrap();

            let mut answer = String::new();
            io::stdin().read_line(&mut answer).unwrap();

            let answer = answer.trim();

            let url = reqwest::Url::parse(answer).context("Ingevoerde URL is geen URL.")?;
            let token = url
                .query_pairs()
                .find_map(|(name, value)| (name == "token").then_some(value))
                .context("URL bevat geen token.")?;

            let refresh_token = config.refresh_token(&token)?;
            let authorization_token = create_authorization_token(OAuthConfig {
                odido_api_url: config.odido_api_url,
                refresh_token,
                odido_oauth_key: config.odido_oauth_key,
            })?;
            println!("\n\n\n\nAUTHORIZATION_TOKEN={authorization_token}");

            return Ok(());
        }
        OAuthRequired(config) => {
            let authorization_token = create_authorization_token(config)?;
            println!("\n\n\n\nAUTHORIZATION_TOKEN={authorization_token}");
            return Ok(());
        }

        Authenticated(config) => {
            let interval = config.update_interval;
            let mb_threshold = config.mb_threshold;
            (Odido::new(config), interval, mb_threshold)
        }
    };

    loop {
        let started = Instant::now();

        match odido.mbs_left() {
            Ok(mb_left) => {
                println!("{mb_left} MB's beschikbaar");
                if mb_left < mb_threshold {
                    if let Err(e) = odido.request_bundle() {
                        eprintln!("Fout bij aanvragen van extra databundel: {e}");
                    }
                }
            }
            Err(e) => eprintln!("Fout bij opvragen hoeveel MB's beschikbaar: {e}"),
        }

        let sleep_for: Duration = interval.saturating_sub(started.elapsed());
        std::thread::sleep(sleep_for);
    }
}
