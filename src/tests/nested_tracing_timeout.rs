use std::time::Duration;

use serial_test::serial;
use tokio::time::sleep;
use tracing::instrument;

use crate::{
    TimeoutElapsed,
    tests::{insta_trace_filters, run_with_tracing},
    timeout,
    trace::CaptureSpanAndStackTrace,
};

#[tokio::test]
#[serial]
async fn with_timeouts() {
    let result = run_with_tracing(Duration::from_millis(100), do_f()).await;

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
async fn do_f() {
    _ = timeout(Duration::from_secs(1), CaptureSpanAndStackTrace, do_g()).await;
}

#[instrument]
async fn do_g() {
    sleep(Duration::from_secs(2)).await;
}
