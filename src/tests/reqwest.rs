use std::time::Duration;

use serial_test::serial;
use tracing::instrument;

use crate::{
    TimeoutElapsed,
    tests::{insta_trace_filters, run_with_tracing},
};

#[tokio::test]
#[ignore = "fails in CI with some different error"] // TODO: run a test server in docker-compose.deps.yml in CI
#[serial]
async fn with_reqwest() {
    let result = run_with_tracing(Duration::from_millis(100), do_reqwest()).await;

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
async fn do_reqwest() {
    let response = reqwest::get("http://loclahost").await.unwrap();
    response.bytes().await.unwrap();
}
