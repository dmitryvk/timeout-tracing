use std::{backtrace::Backtrace, fmt::Display};

use tracing_error::SpanTrace;

pub trait CaptureTrace {
    type Trace;
    fn capture(&self) -> Self::Trace;
}

pub struct DefaultTraceCapturer;

impl CaptureTrace for DefaultTraceCapturer {
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
