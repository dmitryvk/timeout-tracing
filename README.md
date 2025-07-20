`timeout-tracing` allows execution an async function with a timeout (much like `tokio::time::timeout`),
and when the timeout happens it returns the exact location where the async code was awaiting at that specific moment.

# Basic usage

The basic usage looks as follows:

```rust,no_run
use std::time::Duration;

use timeout_tracing::{DefaultTraceCapturer, timeout};
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
    match timeout_tracing::timeout(
        Duration::from_secs(1),
        DefaultTraceCapturer, // (3)
        computation(25),
    )
    .await
    {
        Ok(()) => println!("Completed"),
        Err(elapsed) => println!("Timed out at \n{}", elapsed.active_traces[0].span_trace()), // (4)
    }
}

#[instrument] // (5)
async fn computation(n: i32) {
    for i in 0..n {
        step(i).await;
    }
}

#[instrument]
async fn step(i: i32) {
    sleep(Duration::from_millis(100)).await;
}
```

This prints out:

```skip
Timed out at
   0: basic::step
           with i=9
             at examples/basic.rs:34
   1: basic::computation
           with n=25
             at examples/basic.rs:27
```

1. `tracing-error` must be initialized, as it is used (by default) to gather span traces.
2. `timeout_tracing::timeout` executes the future with a timeout
3. `DefaultTraceCapturer` is the object that captures the stack. The default implementation captures span trace (via `tracing-error`) and stack trace (via Rust standard library; the `RUST_BACKTRACE=1` environment variable must be set for stack trace capture to work)
4. If the future does not complete within the given time limit, an error is returned. It contains a set of traces for each active leaf await point within the future.
5. The executed functions should be instrumented with `tokio-tracing` spans (for example, by using the `#[tokio-tracing::instrument]` macro) for span trace to work.

