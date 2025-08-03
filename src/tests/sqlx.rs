use std::time::Duration;

use serial_test::serial;
use sqlx::{Connection, PgConnection};
use tracing::instrument;

use crate::{
    TimeoutElapsed,
    tests::{insta_trace_filters, run_with_tracing},
};

#[tokio::test]
#[ignore = "need to run docker-compose.deps.yml in CI"]
#[serial]
async fn with_sqlx() {
    let result = run_with_tracing(Duration::from_millis(1000), do_sqlx()).await;

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
async fn do_sqlx() {
    let mut conn = sqlx::PgConnection::connect("postgresql://pg:123@localhost:15432/pg")
        .await
        .unwrap();
    exec_query(&mut conn, "select pg_sleep(0.001)").await;
    exec_query(&mut conn, "select pg_sleep(2)").await;
}

#[instrument(skip(conn))]
async fn exec_query(conn: &mut PgConnection, sql: &str) {
    let _results: Vec<()> = sqlx::query_as(sql).fetch_all(conn).await.unwrap();
}
