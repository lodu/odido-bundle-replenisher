use anyhow::{Context, Result};
use clap::{ArgAction, Parser};
use std::time::Duration;

#[derive(Clone, Copy)]
pub enum RunMode {
    Once,
    Loop,
}

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// +31 pre-fixed telefoonnummer.
    #[arg(short, long)]
    msisdn: Option<String>,

    /// Account authorizatie token.
    #[arg(short, long)]
    authorization_token: Option<String>,

    /// Account refresh token.
    #[arg(short, long)]
    refresh_token: Option<String>,

    /// Draai één keer i.p.v. in een loop.
    #[arg(short, long, action = ArgAction::SetTrue)]
    once: bool,
}

pub enum StartupMode {
    Authenticated(AuthenticatedConfig),
    AuthorizationCode(AuthorizationCodeConfig),
    LoginRequired(LoginConfig),
}

#[derive(Clone, Copy)]
pub enum IntervalMode {
    Dynamic {
        interval_below_threshold: Duration,
        interval_above_threshold: Duration,
        threshold: u32, // MB
    },
    Static(Duration),
}

impl IntervalMode {
    pub fn determine_duration(&self, mbs_left: u32) -> Duration {
        match self {
            IntervalMode::Dynamic {
                interval_below_threshold,
                interval_above_threshold,
                threshold,
            } => {
                if mbs_left < *threshold {
                    *interval_below_threshold
                } else {
                    *interval_above_threshold
                }
            }
            IntervalMode::Static(duration) => *duration,
        }
    }
}

pub struct AuthorizationCodeConfig {
    pub odido_api_url: String,
    pub authorization_code: String,
    pub odido_oauth_key: String,
}

pub struct LoginConfig {
    pub odido_url: String,
    pub odido_api_url: String,
    pub odido_fernet_key: String,
    pub odido_oauth_key: String,
}

pub struct AuthenticatedConfig {
    pub authorization_token: String,
    pub msisdn: String,
    pub interval_mode: IntervalMode,
    pub odido_api_url: String,
    pub odido_user_agent: String,
    pub odido_buying_code: String,
    pub mb_threshold: u32,
    pub http_max_retries: u32,
    pub http_retry_delay_step: u32,
    pub run_mode: RunMode,
}

fn parse_env<T: std::str::FromStr>(key: &str) -> Option<T> {
    std::env::var(key).ok()?.parse().ok()
}

fn parse_env_with_default<T: std::str::FromStr>(key: &str, default: T) -> T {
    parse_env(key).unwrap_or(default)
}

fn determine_interval_config() -> IntervalMode {
    if let Some(minutes) = parse_env::<u64>("CHECK_INTERVAL") {
        return IntervalMode::Static(Duration::from_secs(minutes * 60));
    }

    let threshold_mb: u32 = parse_env_with_default("DYNAMIC_INTERVAL_MB_THRESHOLD", 4000);
    let interval_below_threshold_minutes: u64 = parse_env_with_default("DYNAMIC_INTERVAL_LOW", 1);
    let interval_above_threshold_minutes: u64 = parse_env_with_default("DYNAMIC_INTERVAL_HIGH", 10);

    IntervalMode::Dynamic {
        interval_below_threshold: Duration::from_secs(interval_below_threshold_minutes * 60),
        interval_above_threshold: Duration::from_secs(interval_above_threshold_minutes * 60),
        threshold: threshold_mb,
    }
}

impl AuthenticatedConfig {
    pub fn from_env(cli: Cli) -> Result<StartupMode> {
        let odido_url: String = parse_env_with_default("ODIDO_URL", "https://odido.nl".to_owned());
        let odido_fernet_key: String = parse_env_with_default(
            "ODIDO_FERNET_KEY",
            "afIqRZm6iSev4zWysNGAjR6fCrOMf5GQqhKFfmXkgOU".to_owned(),
        );
        let odido_oauth_key: String =
            parse_env_with_default("ODIDO_OAUTH_KEY", "9havvat6hm0b962i".to_owned());

        let odido_api_url: String =
            parse_env_with_default("ODIDO_API_URL", "https://capi.odido.nl".to_owned());

        let authorization_token: String = match cli
            .authorization_token
            .or_else(|| parse_env::<String>("AUTHORIZATION_TOKEN"))
        {
            Some(token) => token,
            None => match cli
                .refresh_token
                .or_else(|| parse_env::<String>("REFRESH_TOKEN"))
            {
                Some(refresh_token) => {
                    return Ok(StartupMode::AuthorizationCode(AuthorizationCodeConfig {
                        odido_api_url,
                        authorization_code: refresh_token,
                        odido_oauth_key,
                    }));
                }
                None => {
                    return Ok(StartupMode::LoginRequired(LoginConfig {
                        odido_url,
                        odido_api_url,
                        odido_fernet_key,
                        odido_oauth_key,
                    }));
                }
            },
        };

        let msisdn = cli
            .msisdn
            .or_else(|| parse_env::<String>("MSISDN"))
            .context("MSISDN niet gevonden in env")?;

        let odido_user_agent: String = parse_env_with_default(
            "ODIDO_USER_AGENT",
            "ODIDO 8.0.0 (Android 12; 12)".to_owned(),
        );

        let odido_buying_code: String =
            parse_env_with_default("ODIDO_BUYING_CODE", "A0DAY01".to_owned());

        let mb_threshold = parse_env_with_default("MB_THRESHOLD", 2000);

        let http_max_retries = parse_env_with_default("HTTP_MAX_RETRIES", 10);

        let http_retry_delay_step = parse_env_with_default("HTTP_RETRY_DELAY_STEP", 10);

        let run_mode = if cli.once || parse_env_with_default("RUN_ONCE", false) {
            RunMode::Once
        } else {
            RunMode::Loop
        };

        Ok(StartupMode::Authenticated(AuthenticatedConfig {
            authorization_token,
            msisdn,
            interval_mode: determine_interval_config(),
            odido_api_url,
            odido_user_agent,
            odido_buying_code,
            mb_threshold,
            http_max_retries,
            http_retry_delay_step,
            run_mode,
        }))
    }
}
