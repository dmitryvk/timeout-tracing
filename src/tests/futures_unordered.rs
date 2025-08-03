use std::time::Duration;

use futures::{StreamExt, stream::FuturesUnordered};
use serial_test::serial;
use tokio::time::sleep;
use tracing::instrument;

use crate::{
    TimeoutElapsed,
    tests::{insta_trace_filters, run_with_tracing},
};

#[tokio::test]
#[serial]
async fn with_futures_unordered() {
    let result = run_with_tracing(Duration::from_millis(100), do_unordered()).await;

    assert!(matches!(result, Err(TimeoutElapsed { .. })));
    // Can't see inside FuturesUnordered, as it manages its own sub-wakers
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
async fn do_unordered() {
    let mut fu = FuturesUnordered::new();
    for i in 0..10 {
        fu.push(inner_fut(i));
    }

    while let Some(()) = fu.next().await {}
}

#[instrument]
async fn inner_fut(idx: u32) {
    _ = idx;
    sleep(Duration::from_secs(1)).await;
}
