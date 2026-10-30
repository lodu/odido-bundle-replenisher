use nyquest::{BlockingClient, Body, Request, blocking::Response};
use serde::de::DeserializeOwned;
use std::{convert::Infallible, time::Duration};
use thiserror::Error;

const RETRY_DELAY_STEP_MS: u32 = 100;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("request mislukt: {0}")]
    Request(#[from] nyquest::Error),
    #[error("na {0} niet-gelukt pogingen gestopt")]
    RetriesExhausted(u32),
    #[error("onverwachte response: {0}")]
    UnexpectedResponse(String),
    #[error("HTTP {0}: {1}")]
    ServerError(nyquest::StatusCode, String),
}

pub fn get_json<T: DeserializeOwned>(
    client: &BlockingClient,
    url: &str,
    max_retries: u32,
    retry_delay_stepsize: u32,
) -> Result<T, HttpError> {
    request_with_retry(
        || client.request(Request::get(url.to_string())),
        max_retries,
        retry_delay_stepsize,
    )
}

pub fn post(
    client: &BlockingClient,
    url: &str,
    body: Body<Infallible>,
) -> Result<Response, HttpError> {
    let request = Request::post(url.to_string()).with_body(body);

    let response = client.request(request)?;

    check_http_status(response)
}

fn check_http_status(response: Response) -> Result<Response, HttpError> {
    let status = response.status();
    if status.is_successful() {
        return Ok(response);
    }

    let body = response.text().unwrap_or_default();
    Err(HttpError::ServerError(status, body))
}

fn request_with_retry<T: DeserializeOwned>(
    send: impl Fn() -> Result<Response, nyquest::Error>,
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
