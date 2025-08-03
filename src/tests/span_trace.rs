use std::time::Duration;

use serial_test::serial;
use tokio::{join, time::sleep};
use tracing::{info, instrument};
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;

use crate::{TimeoutElapsed, tests::insta_trace_filters, timeout, trace::CaptureSpanTrace};

#[tokio::test]
#[serial]
async fn format_span_trace() {
    let subscriber = tracing_subscriber::registry().with(ErrorLayer::default());

    let _guard = tracing::subscriber::set_default(subscriber);
    let result = timeout(Duration::from_millis(100), CaptureSpanTrace, do_sleep()).await;

    assert!(matches!(result, Err(TimeoutElapsed { .. })));
    let mut err = result.err().unwrap();
    err.active_traces.sort_by_cached_key(ToString::to_string);
    insta::with_settings!({
        filters => insta_trace_filters()
    }, {
        insta::assert_debug_snapshot!(err);
        insta::assert_snapshot!(err);
    });
}

#[instrument]
async fn do_sleep() {
    info!("sleep before");
    join!(do_sleep_a(), do_sleep_b());
}

#[instrument]
async fn do_sleep_a() {
    info!("sleep a");
    sleep(Duration::from_secs(1)).await;
}

#[instrument]
async fn do_sleep_b() {
    info!("sleep b");
    sleep(Duration::from_secs(1)).await;
}
