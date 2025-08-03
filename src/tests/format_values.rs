use std::time::Duration;

use serial_test::serial;
use tokio::time::sleep;
use tracing::instrument;

use crate::{
    TimeoutElapsed,
    tests::{insta_trace_filters, run_with_tracing},
};

#[tokio::test]
#[serial]
async fn with_values() {
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
    do_sleep_inner(123).await;
}

#[instrument]
async fn do_sleep_inner(n: i32) {
    sleep(Duration::from_secs(1)).await;
}
