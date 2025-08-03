use std::time::Duration;

use pin_project_lite::pin_project;
use serial_test::serial;
use tokio::time::{Sleep, sleep};
use tracing::instrument;

use crate::{
    TimeoutElapsed,
    tests::{insta_trace_filters, run_with_tracing},
};

#[tokio::test]
#[serial]
async fn with_custom_future() {
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
    CustomFut::new(Duration::from_secs(1)).await;
}

pin_project! {
    struct CustomFut {
        #[pin]
        inner: Sleep,
    }
}

impl CustomFut {
    fn new(duration: Duration) -> Self {
        Self {
            inner: sleep(duration),
        }
    }
}

impl Future for CustomFut {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        // Retaining waker is an indicator of await
        // This tests that dropping a waker erases the corresponding trace
        _ = cx.waker().clone();
        self.project().inner.poll(cx)
    }
}
