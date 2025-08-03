use std::time::Duration;

use serial_test::serial;
use tokio::time::sleep;
use tracing::{info, instrument};

use crate::{
    TimeoutElapsed,
    tests::{insta_trace_filters, run_with_tracing},
};
#[tokio::test]
#[serial]
async fn with_timeouts() {
    let result = run_with_tracing(Duration::from_millis(100), do_sleep()).await;

    assert!(matches!(result, Err(TimeoutElapsed { .. })));
    let mut err = result.err().unwrap();
    err.active_traces
        .sort_by_cached_key(|trace| trace.span_trace.to_string());
    insta::with_settings!({
        filters => insta_trace_filters()
    }, {
        insta::assert_debug_snapshot!(err);
        insta::assert_snapshot!(err);
    });
}

#[instrument]
async fn do_sleep() {
    info!("p1");
    do_sleep_1().await;
    info!("p2");
    do_sleep_2().await;
    info!("p3");
    do_sleep_3().await;
    info!("p4");
}

#[instrument]
async fn do_sleep_1() {
    sleep(Duration::from_millis(1)).await;
}

#[instrument]
async fn do_sleep_2() {
    sleep(Duration::from_millis(2)).await;
}

#[instrument]
async fn do_sleep_3() {
    sleep(Duration::from_secs(1)).await;
}
