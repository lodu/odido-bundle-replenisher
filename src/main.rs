mod config;
mod http;
mod odido;

use anyhow::Context;
use config::{AuthenticatedConfig, AuthorizationCodeConfig};
use odido::{OdidoClient, auth::create_authorization_token};
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::config::StartupMode::{Authenticated, AuthorizationCode, LoginRequired};

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let (mut odido, interval, mb_threshold) = match AuthenticatedConfig::from_env()? {
        LoginRequired(config) => {
            println!(
                "Open deze URL om in te loggen:\n{}\n\nPlak daarna de URL hier OF herstart de app met de token (REFRESH_TOKEN env variable).",
                config.login_url()?
            );
            io::stdout().flush()?;

            let mut answer = String::new();
            let bytes_read = io::stdin().read_line(&mut answer)?;
            if bytes_read == 0 {
                anyhow::bail!("Stel REFRESH_TOKEN in (of -it flag bij docker commando).");
            }

            let answer = answer.trim();

            let url = reqwest::Url::parse(answer).context("Ingevoerde URL is geen URL.")?;
            let token = url
                .query_pairs()
                .find_map(|(name, value)| (name == "token").then_some(value))
                .context("Ingevoerde URL bevat geen token.")?;

            let authorization_code = config.authorization_code(&token)?;
            let authorization_token = create_authorization_token(AuthorizationCodeConfig {
                odido_api_url: config.odido_api_url,
                authorization_code,
                odido_oauth_key: config.odido_oauth_key,
            })?;
            println!("\n\n\n\nAUTHORIZATION_TOKEN={authorization_token}");

            return Ok(());
        }
        AuthorizationCode(config) => {
            let authorization_token = create_authorization_token(config)?;
            println!("\n\n\n\nAUTHORIZATION_TOKEN={authorization_token}");
            return Ok(());
        }

        Authenticated(config) => {
            let interval = config.check_interval;
            let mb_threshold = config.mb_threshold;
            (OdidoClient::new(config), interval, mb_threshold)
        }
    };

    loop {
        let started = Instant::now();

        match odido.mbs_left() {
            Ok(mb_left) => {
                println!("Nog {mb_left} MB's beschikbaar.");
                if mb_left < mb_threshold {
                    if let Err(e) = odido.request_bundle() {
                        eprintln!(
                            "Fout bij aanvragen van extra databundel. Dit gebeurt ook als je nog te veel MB's hebt: {e}"
                        );
                    }
                }
            }
            Err(e) => eprintln!("Fout bij opvragen hoeveel MB's beschikbaar: {e}"),
        }

        let sleep_for: Duration = interval.saturating_sub(started.elapsed());
        std::thread::sleep(sleep_for);
    }
}
