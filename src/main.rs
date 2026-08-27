mod config;
mod http;
mod odido;

use anyhow::Context;
use clap::Parser;
use config::{AuthenticatedConfig, AuthorizationCodeConfig, Cli, RunMode};
use odido::{OdidoClient, auth::create_authorization_token};
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::config::StartupMode::{Authenticated, AuthorizationCode, LoginRequired};
use crate::odido::IsReplenished;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli: Cli = Cli::parse();
    let startup_mode = AuthenticatedConfig::from_env(cli)?;

    let (mut odido, interval_mode, run_mode) = match startup_mode {
        LoginRequired(config) => {
            println!(
                "Open deze URL om in te loggen:\n{}\n\nPlak daarna de URL hier OF herstart de app met de token (REFRESH_TOKEN env variable).",
                config.login_url()?
            );
            io::stdout().flush()?;

            let mut answer = String::new();
            let bytes_read = io::stdin().read_line(&mut answer)?;
            if bytes_read == 0 {
                anyhow::bail!("Stel REFRESH_TOKEN in (of gebruik de -it flag bij docker).");
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
            let interval = config.interval_mode;
            let run_mode = config.run_mode;
            (OdidoClient::new(config), interval, run_mode)
        }
    };

    loop {
        let started = Instant::now();

        let result = odido.replenish_if_needed();
        let mb_left: Option<u32> = match result {
            Ok(is_replenished) => {
                let mb_left = match is_replenished {
                    IsReplenished::Replenished { mb_left } => {
                        println!(
                            "Nieuwe bundel succesvol aangevraagd, vanaf nu {mb_left} MB in je bundel."
                        );
                        mb_left
                    }
                    IsReplenished::NotReplenished {
                        mb_left,
                        mb_left_to_replenish,
                    } => {
                        println!(
                            "Nog {mb_left} MB in de bundel. Replenishment mogelijk over {mb_left_to_replenish} MB."
                        );
                        mb_left
                    }
                };
                Some(mb_left)
            }
            Err(e) => {
                eprintln!("Fout bij bekijken (en eventueel aanvragen) bundels: {e:#?}");
                None
            }
        };

        match run_mode {
            RunMode::Once => match mb_left {
                Some(_) => {
                    break Ok(());
                }
                None => std::process::exit(1),
            },
            RunMode::Loop => {
                let interval = interval_mode.determine_duration(mb_left.unwrap_or(0)); // Default naar laagste bij Dynamic
                let sleep_for: Duration = interval.saturating_sub(started.elapsed());
                std::thread::sleep(sleep_for);
            }
        }
    }
}
