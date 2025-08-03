#![doc = include_str!("../README.md")]

use std::{
    error::Error,
    fmt::Display,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use pin_project_lite::pin_project;
use tracing::{Level, span};

use crate::waker::{TracingTimeoutWaker, TracingTimeoutWakerInner};

pub use crate::{
    trace::CaptureSpanAndStackTrace, trace::CaptureSpanTrace, trace::CaptureTrace,
    trace::StackAndSpanTrace,
};

#[cfg(test)]
mod tests;
mod trace;
mod waker;

/// Drive the future `fut` to completion, while limiting its run time to `duration`.
/// If `fut` fails to finish within `furation`, returns span traces for all active
/// await points within `fut`.
/// Use `capture` to specify which kind of trace should be captured.
///
/// # Examples
/// ```rust
/// # use std::time::Duration;
/// # use tokio::time::sleep;
/// # use timeout_tracing::{CaptureSpanTrace, timeout};
/// # use tracing::instrument;
/// # use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
/// # tokio::runtime::Runtime::new()
/// #     .unwrap()
/// #     .block_on(async {
/// tracing_subscriber::registry()
///     .with(tracing_error::ErrorLayer::default())
///     .init();
///
/// let result = timeout(Duration::from_secs(1), CaptureSpanTrace, long_computation()).await;
/// assert_eq!(filter_span_trace(result.err().unwrap().to_string()), "timeout elapsed at:
/// trace 0:
///    0: example::step_2
///              at example.rs:123
///    1: example::long_computation
///              at example.rs:123
/// ");
///
/// #[instrument]
/// async fn long_computation() {
///     step_1().await;
///     step_2().await;
/// }
/// #[instrument]
/// async fn step_1() {
///     sleep(Duration::from_millis(10)).await;
/// }
/// #[instrument]
/// async fn step_2() {
///     sleep(Duration::from_secs(2)).await;
/// }
/// # fn filter_span_trace(trace: String) -> String {
/// #     use regex::Regex;
/// #
/// #     let re = Regex::new(r"at .*\.rs:([0-9_]+)").unwrap();
/// #     let trace = re.replace_all(&trace, "at example.rs:123").to_string();
/// #     let re = Regex::new(r"[a-z0-9:_]+::([a-z]+)").unwrap();
/// #     let trace = re.replace_all(&trace, "example::$1").to_string();
/// #     trace
/// # }
/// # });
/// ```
pub fn timeout<C, Fut>(duration: Duration, capture: C, fut: Fut) -> TimeoutFuture<C, Fut> {
    let deadline = tokio::time::sleep(duration);
    TimeoutFuture {
        deadline,
        capture: Some(capture),
        inner: fut,
    }
}

pin_project! {
    pub struct TimeoutFuture<C, Fut> {
        #[pin]
        deadline: tokio::time::Sleep,
        capture: Option<C>,
        #[pin]
        inner: Fut,
    }
}

impl<C, Fut> Future for TimeoutFuture<C, Fut>
where
    C: CaptureTrace + Send + 'static,
    Fut: Future,
{
    type Output = Result<Fut::Output, TimeoutElapsed<C::Trace>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        // A span just so that nested timeouts had some
        let deadline_span = span!(Level::TRACE, "deadline");
        let guard = deadline_span.enter();
        match this.deadline.poll(cx) {
            Poll::Ready(()) => {
                drop(guard);

                // We hit the timeout. Do one final poll for the inner future, but collect the traces this time.
                // TODO: we don't have to call cx.waker.clone(), but it probably does not matter much since we already hit timeout
                let Some(capture) = this.capture.take() else {
                    return Poll::Ready(Err(TimeoutElapsed {
                        active_traces: Vec::new(),
                    }));
                };
                let waker_inner = TracingTimeoutWakerInner::new(capture, cx.waker().clone());
                let waker = TracingTimeoutWaker::new_std_waker(waker_inner.clone());
                let mut cx2 = Context::from_waker(&waker);
                match this.inner.poll(&mut cx2) {
                    Poll::Pending => {}
                    Poll::Ready(result) => return Poll::Ready(Ok(result)),
                }
                let active_traces: Vec<_> = waker_inner.traces();
                return Poll::Ready(Err(TimeoutElapsed { active_traces }));
            }
            Poll::Pending => {}
        }
        drop(guard);
        match this.inner.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => Poll::Ready(Ok(result)),
        }
    }
}

#[derive(Debug)]
pub struct TimeoutElapsed<Trace> {
    pub active_traces: Vec<Trace>,
}

impl<Trace: Display> Display for TimeoutElapsed<Trace> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.active_traces.is_empty() {
            f.write_str("timeout elapsed")?;
        } else {
            f.write_str("timeout elapsed at:\n")?;
            for (idx, trace) in self.active_traces.iter().enumerate() {
                writeln!(f, "trace {idx}:\n{trace}")?;
            }
        }
        Ok(())
    }
}

impl<Trace> Error for TimeoutElapsed<Trace> where Trace: std::fmt::Debug + std::fmt::Display {}
