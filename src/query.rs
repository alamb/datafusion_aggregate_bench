use std::sync::Arc;

use datafusion::{arrow::util::pretty::pretty_format_batches, datasource::TableProvider, prelude::ExecutionContext};


#[allow(dead_code)]
pub async fn run_query(csvdata: Arc<dyn TableProvider>, query: &str)  {
    let mut ctx = ExecutionContext::new();
    ctx.register_table("t", csvdata).unwrap();

    let results = ctx.sql(query)
        .unwrap()
        .collect()
        .await
        .unwrap();

    let pretty = pretty_format_batches(&results).unwrap();
    println!("{}\n{}", query, pretty);
}


#[allow(dead_code)]
pub async fn run_query_silently(csvdata: Arc<dyn TableProvider>, query: &str)  {
    let mut ctx = ExecutionContext::new();
    ctx.register_table("t", csvdata).unwrap();

    ctx.sql(query)
        .unwrap()
        .collect()
        .await
        .unwrap();
}
