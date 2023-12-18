use axum::http::StatusCode;
use axum::response::IntoResponse;
use lazy_static::lazy_static;
use prometheus::{Encoder, TextEncoder, IntCounter, IntGauge};
use crate::error::ServerError;


lazy_static! {
    pub static ref SCRAPE_COUNTER: IntCounter = IntCounter::new(
        "http_metrics_counter_scrape",
        "How many times was scrapped "
    ).unwrap();

    pub static ref MESSAGE_COUNTER: IntCounter = IntCounter::new(
        "http_metrics_counter_message",
        "How many messages was sent through the running server.",
    ).unwrap();

    pub static ref SUCCESSFUL_CONNECTION_COUNTER: IntCounter = IntCounter::new(
        "http_metrics_counter_successful_connection",
        "How many successful connections from clients was established."
    ).unwrap();

    pub static ref NOT_AUTHORIZED_CONNECTION_COUNTER: IntCounter = IntCounter::new(
        "http_metrics_counter_not_authorized_connection",
        "How many authorizations from clients failed."
    ).unwrap();

    pub static ref CURRENT_CLIENT_COUNT_GAUGE: IntGauge = IntGauge::new(
        "http_metrics_gauge_current_client_count",
        "How many clients are currently connected."
    ).unwrap();
}


pub fn register_prometheus() -> Result<(), ServerError> {
    let counters = vec![
        Box::new(SCRAPE_COUNTER.clone()),
        Box::new(MESSAGE_COUNTER.clone()),
        Box::new(SUCCESSFUL_CONNECTION_COUNTER.clone()),
        Box::new(NOT_AUTHORIZED_CONNECTION_COUNTER.clone()),
    ];

    for counter in counters {
        if let Err(err) = prometheus::default_registry().register(counter) {
            Err(ServerError::PrometheusRegistrationError(err.to_string()))?;
        }
    };

    let gauges = vec![
        Box::new(CURRENT_CLIENT_COUNT_GAUGE.clone()),
    ];

    for gauge in gauges {
        if let Err(err) = prometheus::default_registry().register(gauge) {
            Err(ServerError::PrometheusRegistrationError(err.to_string()))?;
        }
    };

    return Ok(())
}


pub async fn prometheus_metrics_handler() -> impl IntoResponse {
    SCRAPE_COUNTER.inc();

    let encoder = TextEncoder::new();
    let mut buffer = vec![];

    let metrics = prometheus::gather();
    encoder.encode(&metrics, &mut buffer).unwrap();

    (StatusCode::OK, buffer)
}
