use reqwest::blocking::{Client, Response};
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]

pub enum HttpError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("giving up after {0} retries")]
    RetriesExhausted(u32),
    #[error("unexpected response shape: {0}")]
    UnexpectedResponse(String),
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

pub fn post_empty(
    client: &Client,
    url: &str,
    headers: HeaderMap,
    body: String,
) -> Result<(), HttpError> {
    client
        .post(url)
        .headers(headers)
        .body(body)
        .send()?
        .error_for_status()?;
    Ok(())
}

fn request_with_retry<T: DeserializeOwned>(
    send: impl Fn() -> reqwest::Result<Response>,
    max_retries: u32,
    retry_delay_stepsize: u32,
) -> Result<T, HttpError> {
    let mut last_err: Option<reqwest::Error> = None;
    for attempt in 0..=max_retries {
        match send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.json::<T>())
        {
            Ok(value) => return Ok(value),
            Err(e) => {
                eprintln!("request failed, retrying: {e}");
                last_err = Some(e);
                std::thread::sleep(Duration::from_millis(
                    (retry_delay_stepsize * 100 * (attempt + 1)) as u64,
                ));
            }
        }
    }
    Err(last_err
        .map(HttpError::from)
        .unwrap_or(HttpError::RetriesExhausted(max_retries)))
}
