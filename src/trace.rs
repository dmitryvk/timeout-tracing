use std::{backtrace::Backtrace, fmt::Display};

use tracing_error::SpanTrace;

/// A trait to support custom implementations of traces
pub trait CaptureTrace {
    /// Representation for captured trace
    type Trace;
    /// Capture trace at the current moment.
    fn capture(&self) -> Self::Trace;
}

/// Implementation of [`CaptureTrace`] that captures span trace using [`tracing_error::SpanTrace`].
/// [`tracing`] must be initialized with [`tracing_error::ErrorLayer`] for the trace to be captured successfully.
pub struct CaptureSpanTrace;

impl CaptureTrace for CaptureSpanTrace {
    type Trace = SpanTrace;

    fn capture(&self) -> Self::Trace {
        SpanTrace::capture()
    }
}

/// Implementation of [`CaptureTrace`] that captures both a span trace using [`tracing_error::SpanTrace`]
/// and a stack trace.
/// [`tracing`] must be initialized with [`tracing_error::ErrorLayer`] for the span trace to be captured successfully
/// and `RUST_BACKTRACE` environment variable must be set for the stack trace to be captured.
pub struct CaptureSpanAndStackTrace;

impl CaptureTrace for CaptureSpanAndStackTrace {
    type Trace = StackAndSpanTrace;

    fn capture(&self) -> Self::Trace {
        StackAndSpanTrace::capture()
    }
}

#[derive(Debug)]
pub struct StackAndSpanTrace {
    pub(crate) stack_trace: Backtrace,
    pub(crate) span_trace: SpanTrace,
}

impl StackAndSpanTrace {
    pub(crate) fn capture() -> Self {
        Self {
            span_trace: SpanTrace::capture(),
            stack_trace: Backtrace::capture(),
        }
    }

    pub fn stack_trace(&self) -> &Backtrace {
        &self.stack_trace
    }

    pub fn span_trace(&self) -> &SpanTrace {
        &self.span_trace
    }
}

impl Display for StackAndSpanTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "span trace:\n{span_trace}\nstack trace:\n{stack_trace}",
            span_trace = self.span_trace,
            stack_trace = self.stack_trace,
        )
    }
}
