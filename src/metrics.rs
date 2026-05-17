use axum::{routing::get, Router};
use once_cell::sync::Lazy;
use prometheus::{Counter, Encoder, Gauge, TextEncoder, register_counter, register_gauge};
use std::sync::OnceLock;

static INITIATED: OnceLock<Counter> = OnceLock::new();
static COMPLETED: OnceLock<Counter> = OnceLock::new();
static IN_FLIGHT: OnceLock<Gauge> = OnceLock::new();

pub fn init_metrics() {
    let _ = INITIATED.set(register_counter!("avalanche_atomic_swaps_initiated_total", "Total initiated swaps").unwrap());
    let _ = COMPLETED.set(register_counter!("avalanche_atomic_swaps_completed_total", "Total completed swaps").unwrap());
    let _ = IN_FLIGHT.set(register_gauge!("avalanche_atomic_swaps_in_flight", "Current in-flight swaps").unwrap());
}

pub fn inc_initiated() { if let Some(c) = INITIATED.get() { c.inc(); } }
pub fn inc_completed() { if let Some(c) = COMPLETED.get() { c.inc(); } }
pub fn set_in_flight(n: usize) { if let Some(g) = IN_FLIGHT.get() { g.set(n as f64); } }

pub async fn start_metrics_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new().route("/metrics", get(metrics_handler));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!(" Prometheus metrics available at http://0.0.0.0:{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder.encode(&prometheus::gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}