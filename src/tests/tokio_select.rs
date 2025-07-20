use std::time::Duration;

use futures::FutureExt;
use serial_test::serial;
use tokio::time::sleep;
use tracing::{info, instrument};

use crate::{
    TimeoutElapsed,
    tests::{insta_trace_filters, run_with_tracing},
};

#[tokio::test]
#[serial]
async fn with_select() {
    let mut result = run_with_tracing(Duration::from_millis(100), do_sleep()).await;

    assert!(matches!(result, Err(TimeoutElapsed { .. })));
    result
        .as_mut()
        .err()
        .unwrap()
        .active_traces
        .sort_by_cached_key(|trace| trace.span_trace.to_string());
    assert_eq!(result.as_ref().err().unwrap().active_traces.len(), 2);
    assert!(
        result
            .as_ref()
            .err()
            .unwrap()
            .active_traces
            .iter()
            .any(|e| e.stack_trace.to_string().contains("do_sleep_a"))
    );
    assert!(
        result
            .as_ref()
            .err()
            .unwrap()
            .active_traces
            .iter()
            .any(|e| e.stack_trace.to_string().contains("do_sleep_b"))
    );
    insta::with_settings!({
        filters => insta_trace_filters()
    }, {
        insta::assert_debug_snapshot!(result);
    });
}

#[instrument]
async fn do_sleep() {
    tokio::select! {
        _ = do_sleep_a().fuse() => {},
        _ = do_sleep_b().fuse() => {}
    }
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
