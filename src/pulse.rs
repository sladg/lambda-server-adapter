use reqwest::blocking::Client;
use std::time::Duration;
use tracing::{debug, info, warn};

pub enum HealthCheckError {
    Timeout,
    NetworkError,
}

// Inspired by
// https://stackoverflow.com/questions/36181719/what-is-the-correct-way-in-rust-to-create-a-timeout-for-a-thread-or-a-function

// We wait for given time or until we get a response.

pub async fn pulse(domain: String, timeout: Duration) -> Result<(), HealthCheckError> {
    info!("[Pulse] Pinging server ...");

    let client = Client::builder().timeout(timeout).build().unwrap();
    let result = client.get(&domain).send();

    match result {
        Ok(resp) => match resp.status() {
            reqwest::StatusCode::OK => {
                info!("[Pulse] Server is up");
                Ok(())
            }
            _ => {
                warn!("[Pulse] Server wrong answer, status code: {:?}", resp.status());
                Err(HealthCheckError::NetworkError)
            }
        },
        Err(err) => {
            if err.is_timeout() {
                debug!("[Pulse] Timeout");
                Err(HealthCheckError::Timeout)
            } else {
                debug!("[Pulse] Network error {:?}", err);
                Err(HealthCheckError::NetworkError)
            }
        }
    }
}
