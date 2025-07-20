use std::time::Duration;

use serial_test::serial;
use tokio::{join, time::sleep};
use tracing::{info, instrument};

use crate::{
    TimeoutElapsed,
    tests::{insta_trace_filters, run_with_tracing},
};

#[tokio::test]
#[serial]
async fn with_join() {
    let result = run_with_tracing(Duration::from_millis(100), do_sleep()).await;

    assert!(matches!(result, Err(TimeoutElapsed { .. })));
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
