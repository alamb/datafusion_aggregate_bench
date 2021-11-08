use std::sync::Arc;

use datafusion::{
    arrow::util::pretty::pretty_format_batches,
    datasource::TableProvider,
    prelude::{ExecutionConfig, ExecutionContext},
};

fn make_ctx() -> ExecutionContext {
    // hardcode 1 thread to avoid consuming all CPUs on system which
    // both overheats my laptop causing power throttling as well as
    // makes the results more subject to other workloads on the
    // machine.
    let config = ExecutionConfig::new().with_target_partitions(1);
    ExecutionContext::with_config(config)
}

pub async fn run_query(csvdata: Arc<dyn TableProvider>, query: &str) {
    let mut ctx = make_ctx();
    ctx.register_table("t", csvdata).unwrap();

    let results = ctx.sql(query).await.unwrap().collect().await.unwrap();

    let pretty = pretty_format_batches(&results).unwrap();
    println!("{}\n{}", query, pretty);
}

pub async fn run_query_silently(csvdata: Arc<dyn TableProvider>, query: &str) {
    let mut ctx = make_ctx();
    ctx.register_table("t", csvdata).unwrap();

    ctx.sql(query).await.unwrap().collect().await.unwrap();
}
