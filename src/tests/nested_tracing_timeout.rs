use std::time::Duration;

use serial_test::serial;
use tokio::time::sleep;
use tracing::instrument;

use crate::{
    TimeoutElapsed,
    tests::{insta_trace_filters, run_with_tracing},
    timeout,
    trace::DefaultTraceCapturer,
};

#[tokio::test]
#[serial]
async fn with_timeouts() {
    let result = run_with_tracing(Duration::from_millis(100), do_f()).await;

    assert!(matches!(result, Err(TimeoutElapsed { .. })));
    assert_eq!(result.as_ref().err().unwrap().active_traces.len(), 2);
    assert!(
        result
            .as_ref()
            .err()
            .unwrap()
            .active_traces
            .iter()
            .all(|e| e.stack_trace.to_string().contains("do_f"))
    );
    assert!(
        result
            .as_ref()
            .err()
            .unwrap()
            .active_traces
            .iter()
            .any(|e| e.stack_trace.to_string().contains("do_g"))
    );
    insta::with_settings!({
        filters => insta_trace_filters()
    }, {
        insta::assert_debug_snapshot!(result);
    });
}

#[instrument]
async fn do_f() {
    _ = timeout(Duration::from_secs(1), DefaultTraceCapturer, do_g()).await;
}

#[instrument]
async fn do_g() {
    sleep(Duration::from_secs(2)).await;
}
