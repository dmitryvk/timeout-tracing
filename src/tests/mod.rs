use std::{fmt::Debug, time::Duration};

use itertools::Itertools;
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;

use crate::{
    TimeoutElapsed,
    trace::{CaptureSpanAndStackTrace, StackAndSpanTrace},
};

mod custom_future;
mod format_values;
mod futures_select;
mod futures_unordered;
mod join;
mod nested_tracing_timeout;
mod reqwest;
mod sleep;
mod span_trace;
mod sqlx;
mod tokio_select;

async fn run_with_tracing<Fut>(
    duration: Duration,
    f: Fut,
) -> Result<Fut::Output, TimeoutElapsed<StackAndSpanTrace>>
where
    Fut: Future,
    Fut::Output: Debug,
{
    let subscriber = tracing_subscriber::registry()
        .with(ErrorLayer::default())
        // .with(tracing_subscriber::fmt::layer().compact().with_ansi(false))
        ;

    let _guard = tracing::subscriber::set_default(subscriber);
    let old_val = std::env::var_os("RUST_BACKTRACE");
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };

    info!("before call");
    let result = crate::timeout(duration, CaptureSpanAndStackTrace, f).await;
    info!("after call");

    match old_val {
        Some(old_val) => unsafe { std::env::set_var("RUST_BACKTRACE", old_val) },
        None => unsafe {
            std::env::remove_var("RUST_BACKTRACE");
        },
    }

    match &result {
        Ok(value) => info!("got ok {value:#?}"),
        Err(elapsed) => info!(
            "got timeout\n{} traces:\n{}",
            elapsed.active_traces.len(),
            elapsed
                .active_traces
                .iter()
                .enumerate()
                .map(|(idx, trace)| format!("async trace #{idx}:\n{}", trace.span_trace))
                .join("\n")
        ),
    }

    result
}

fn insta_trace_filters() -> Vec<(&'static str, &'static str)> {
    vec![
        (r#".*/rustlib/src/rust/.*\n"#, r#""#),
        (r#".*/rustc/[0-9a-z]+/.*\n"#, r#""#),
        (
            r#""[^"]*/index.crates.io-[0-9a-z]*/([^/]+)-[^-/]+/"#,
            r#""[crates]/$1-[ver]/"#,
        ),
        (
            r#"at .*/index.crates.io-[0-9a-z]*/([^/]+)-[^-/]+/"#,
            r#"at [crates]/$1-[ver]/"#,
        ),
        (r#"line: [0-9]+"#, r#"line: [NNN]"#),
        (r#"\.rs:[0-9]+:[0-9]+"#, r#".rs:[NNN]:[NNN]"#),
        (r#"\.rs:[0-9]+"#, r#".rs:[NNN]"#),
    ]
}
