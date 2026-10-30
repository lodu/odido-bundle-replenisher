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
    #[error("HTTP {0}: {1}")]
    ServerError(reqwest::StatusCode, String),
    #[error("niet geautoriseerd (401): {0}")]
    Unauthorized(String),
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

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(HttpError::Unauthorized(response.text().unwrap_or_default()));
    }

    let body = response.text().unwrap_or_default();
    Err(HttpError::ServerError(status, body))
}

fn request_with_retry<T: DeserializeOwned>(
    send: impl Fn() -> reqwest::Result<Response>,
    max_retries: u32,
    retry_delay_stepsize: u32,
) -> Result<T, HttpError> {
    let mut attempt = 0;
    loop {
        let result = send()
            .map_err(HttpError::from)
            .and_then(check_http_status)
            .and_then(|r| r.json::<T>().map_err(HttpError::from));

        let e = match result {
            Ok(value) => return Ok(value),
            Err(e) => e,
        };

        if matches!(e, HttpError::Unauthorized(_)) {
            return Err(e);
        }

        if attempt == max_retries {
            return Err(e);
        }

        attempt += 1;
        std::thread::sleep(Duration::from_millis(
            (retry_delay_stepsize * RETRY_DELAY_STEP_MS * attempt) as u64,
        ));
    }
}

#[cfg(test)]
mod request_with_retry {
    use super::*;
    use std::cell::Cell;

    const MAX_RETRIES: u32 = 3;
    const NO_DELAY: u32 = 0;

    fn response_with(status: reqwest::StatusCode) -> reqwest::Result<Response> {
        Ok(::http::Response::builder()
            .status(status)
            .body("")
            .unwrap()
            .into())
    }

    #[test]
    fn unauthorized_not_retried() {
        let calls = Cell::new(0);
        let send = || {
            calls.set(calls.get() + 1);
            response_with(reqwest::StatusCode::UNAUTHORIZED)
        };

        let result = request_with_retry::<serde_json::Value>(send, MAX_RETRIES, NO_DELAY);

        assert!(matches!(result, Err(HttpError::Unauthorized(_))));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn server_error_retried() {
        let calls = Cell::new(0);
        let send = || {
            calls.set(calls.get() + 1);
            response_with(reqwest::StatusCode::INTERNAL_SERVER_ERROR)
        };

        let result = request_with_retry::<serde_json::Value>(send, MAX_RETRIES, NO_DELAY);

        assert!(matches!(result, Err(HttpError::ServerError(..))));
        assert_eq!(calls.get(), MAX_RETRIES + 1);
    }
}
