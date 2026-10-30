use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, InvalidHeaderValue};
use serde::de::DeserializeOwned;
use std::time::Duration;
use thiserror::Error;

const RETRY_DELAY_STEP_MS: u32 = 100;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("request mislukt: {0}")]
    Request(#[from] reqwest::Error),
    #[error("foutieve HTTP header: {0}")]
    Header(#[from] InvalidHeaderValue),
    #[error("na {0} niet-gelukt pogingen gestopt")]
    RetriesExhausted(u32),
    #[error("onverwachte response: {0}")]
    UnexpectedResponse(String),
    #[error("HTTP {0}: {1}")]
    ServerError(reqwest::StatusCode, String),
}

pub fn init() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

pub fn get_json<T: DeserializeOwned>(
    client: &Client,
    url: &str,
    headers: HeaderMap,
    max_retries: u32,
    retry_delay_stepsize: u32,
) -> Result<T, HttpError> {
    request_with_retry(
        || client.get(url).headers(headers.clone()).send(),
        max_retries,
        retry_delay_stepsize,
    )
}

pub fn post(
    client: &Client,
    url: &str,
    headers: HeaderMap,
    body: String,
) -> Result<Response, HttpError> {
    let response = client.post(url).headers(headers).body(body).send()?;
    check_http_status(response)
}

fn check_http_status(response: Response) -> Result<Response, HttpError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let body = response.text().unwrap_or_default();
    Err(HttpError::ServerError(status, body))
}

fn request_with_retry<T: DeserializeOwned>(
    send: impl Fn() -> reqwest::Result<Response>,
    max_retries: u32,
    retry_delay_stepsize: u32,
) -> Result<T, HttpError> {
    let mut last_err: Option<HttpError> = None;

    for attempt in 0..=max_retries {
        let result = send()
            .map_err(HttpError::from)
            .and_then(check_http_status)
            .and_then(|r| r.json::<T>().map_err(HttpError::from));

        match result {
            Ok(value) => return Ok(value),
            Err(e) => {
                last_err = Some(e);
                std::thread::sleep(Duration::from_millis(
                    (retry_delay_stepsize * RETRY_DELAY_STEP_MS * (attempt + 1)) as u64,
                ));
            }
        }
    }
    Err(last_err.unwrap_or(HttpError::RetriesExhausted(max_retries)))
}
