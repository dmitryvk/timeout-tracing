use std::time::Duration;

use timeout_tracing::{CaptureSpanTrace, timeout};
use tokio::time::sleep;
use tracing::instrument;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // (1)
    tracing_subscriber::registry()
        .with(ErrorLayer::default())
        .init();

    // (2)
    match timeout(Duration::from_secs(1), CaptureSpanTrace, computation(25)).await {
        Ok(()) => println!("Completed"),
        Err(elapsed) => println!("{elapsed}"), // (4)
    }
}

// (5)
#[instrument]
async fn computation(n: i32) {
    for i in 0..n {
        step(i).await;
    }
}

#[instrument]
async fn step(i: i32) {
    sleep(Duration::from_millis(100)).await;
}
