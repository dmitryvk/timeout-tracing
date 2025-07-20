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
    assert_eq!(result.as_ref().err().unwrap().active_traces.len(), 1);
    assert!(
        result
            .as_ref()
            .err()
            .unwrap()
            .active_traces
            .iter()
            .any(|e| e.stack_trace.to_string().contains("do_sleep_inner")
                && e.span_trace.to_string().contains("n=123")
                && e.span_trace.to_string().contains("do_sleep_inner"))
    );
    insta::with_settings!({
        filters => insta_trace_filters()
    }, {
        insta::assert_debug_snapshot!(result);
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
