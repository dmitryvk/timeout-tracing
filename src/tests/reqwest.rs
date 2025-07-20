use std::time::Duration;

use serial_test::serial;
use tracing::instrument;

use crate::{
    TimeoutElapsed,
    tests::{insta_trace_filters, run_with_tracing},
};

#[tokio::test]
#[ignore] // TODO: run a test server in docker-compose.deps.yml in CI
#[serial]
async fn with_reqwest() {
    let result = run_with_tracing(Duration::from_millis(100), do_reqwest()).await;

    assert!(matches!(result, Err(TimeoutElapsed { .. })));
    let traces = &result.as_ref().err().unwrap().active_traces;
    for (trace_idx, trace) in traces.iter().enumerate() {
        assert!(
            trace.span_trace.to_string().contains("do_reqwest"),
            "trace #{trace_idx}"
        );
        assert!(
            trace.stack_trace.to_string().contains("do_reqwest"),
            "trace #{trace_idx}"
        );
    }
    assert!(
        traces.iter().any(|trace| trace
            .stack_trace
            .to_string()
            .contains("reqwest::dns::resolve::Resolve")),
        "no trace contains reqwest::dns::resolve::Resolve"
    );
    assert!(
        traces
            .iter()
            .any(|trace| trace.stack_trace.to_string().contains("do_reqwest")),
        "no trace contains do_reqwest"
    );
    insta::with_settings!({
        filters => insta_trace_filters()
    }, {
        insta::assert_debug_snapshot!(result);
    });
}

#[instrument]
async fn do_reqwest() {
    let response = reqwest::get("http://loclahost").await.unwrap();
    response.bytes().await.unwrap();
}
