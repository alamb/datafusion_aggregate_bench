use std::{sync::Arc, time::Instant};

use datafusion::arrow::{
    array::{ArrayRef, DictionaryArray, Float64Array, Int64Array, StringArray},
    datatypes::Int32Type,
    record_batch::RecordBatch,
};
use rand::{
    prelude::{SliceRandom, StdRng},
    Rng, SeedableRng,
};

use crate::{
    query::{run_query, run_query_silently},
    repeat::RepeatedTable,
};

mod query;
mod repeat;

const NUM_ITERATIONS: i64 = 5;
const BATCH_SIZE: usize = 10000;
const NUM_DISTINCT_KEYS: i64 = 100;
const KEY_NULL_DENSITY: f64 = 0.10; // 10% nulls
const VALUE_NULL_DENSITY: f64 = 0.01; // 1% nulls

#[tokio::main]
async fn main() {
    println!("Starting tests");
    //low cardinality
    benchmark_with_cardinality(NUM_DISTINCT_KEYS).await;
    //low cardinality
    benchmark_with_cardinality(NUM_DISTINCT_KEYS * 100).await;
}

async fn benchmark_with_cardinality(num_distinct_keys: i64) {
    run_benchmark(
        format!("{} Groups; 100M rows, int64_keys(10% nulls), f64 values(1% nulls)",num_distinct_keys).as_str(),
        10000,
        num_distinct_keys,
        "select int64_key, count(*), avg(f64) from t group by int64_key",
    )
    .await;

    run_benchmark(
        format!("{} Groups; 100M rows, utf8_keys(10% nulls), f64 values(1% nulls)",num_distinct_keys).as_str(),
        10000,
        num_distinct_keys,
        "select utf8_key, count(*), avg(f64) from t group by utf8_key",
    )
    .await;

    run_benchmark(
        format!("{} Groups; 100M rows, dictionary(utf8, int32) keys(10% nulls), f64 values(1% nulls)",num_distinct_keys).as_str(),
        10000,
        num_distinct_keys,
        "select dict_key, count(*), avg(f64) from t group by dict_key",
    )
    .await;
}

/// create a seedable [`StdRng`](rand::StdRng)
fn seedable_rng() -> StdRng {
    StdRng::seed_from_u64(42)
}

/// creates a test batch of data that has:
/// "int64": int64 keys
/// "utf8_key": utf8 keys
/// "dict_key": dictionary<int32, utf8> keys
/// "f64": Float values
fn make_batch(num_distinct_keys: i64) -> RecordBatch {
    let mut rng = seedable_rng();

    let key_values = (0..num_distinct_keys)
        .map(|i| {
            if rng.gen::<f64>() < KEY_NULL_DENSITY {
                None
            } else {
                Some(i)
            }
        })
        .collect::<Vec<Option<i64>>>();

    let int64_array: Int64Array = (0..BATCH_SIZE)
        .map(|_| {
            // use random numbers to avoid spurious compiler optimizations wrt to branching
            let v: Option<i64> = *key_values.choose(&mut rng).unwrap();
            v
        })
        .collect();

    let key_strings: Vec<Option<String>> = key_values
        .iter()
        .map(|v| v.map(|v| format!("host-foo-bar-baz-{:?}", v)))
        .collect();

    let utf8_array: StringArray = (0..BATCH_SIZE)
        .map(|_| {
            // use random numbers to avoid spurious compiler optimizations wrt to branching
            let v: Option<&String> = key_strings.choose(&mut rng).unwrap().as_ref();
            v
        })
        .collect();

    let dict_array: DictionaryArray<Int32Type> = (0..BATCH_SIZE)
        .map(|_| {
            // use random numbers to avoid spurious compiler optimizations wrt to branching
            let v: Option<&String> = key_strings.choose(&mut rng).unwrap().as_ref();
            v.map(|v| v.as_str())
        })
        .collect();

    let f64_array: Float64Array = create_data(BATCH_SIZE, VALUE_NULL_DENSITY)
        .into_iter()
        .collect();

    RecordBatch::try_from_iter(vec![
        ("int64_key", Arc::new(int64_array) as ArrayRef),
        ("utf8_key", Arc::new(utf8_array) as ArrayRef),
        ("dict_key", Arc::new(dict_array) as ArrayRef),
        ("f64", Arc::new(f64_array) as ArrayRef),
    ])
    .unwrap()
}

fn create_data(size: usize, null_density: f64) -> Vec<Option<f64>> {
    // use random numbers to avoid spurious compiler optimizations wrt to branching
    let mut rng = seedable_rng();

    (0..size)
        .map(|_| {
            if rng.gen::<f64>() > null_density {
                None
            } else {
                Some(rng.gen::<f64>())
            }
        })
        .collect()
}

/// Run the specified query and num batches
async fn run_benchmark(name: &str, num_batches: usize, num_distinct_keys: i64, query: &str) {
    println!("-------------------");
    println!("{}", name);
    println!("-------------------");
    let batch = make_batch(num_distinct_keys);
    let total_rows = num_batches * batch.num_rows();
    println!("Benchmarking {}", query);
    println!(
        "  {} batches of {} rows = {} total rows",
        num_batches,
        batch.num_rows(),
        total_rows
    );

    let table = RepeatedTable::new(batch, num_batches);
    let table = Arc::new(table);
    run_query(table.clone(), &format!("explain {}", query)).await;

    let mut total_duration = std::time::Duration::new(0, 0);
    for _ in 0..NUM_ITERATIONS {
        let start = Instant::now();
        run_query_silently(table.clone(), query).await;
        let duration = Instant::now() - start;
        println!("Completed query in {:?}", duration);
        total_duration += duration;
    }
    let nanos_per_second: f64 = 1_000_000_000.0;
    let rows_per_second: f64 = ((total_rows as f64) / (total_duration.as_nanos() as f64))
        * nanos_per_second
        * (num_iterations as f64);

    println!("---------------");
    println!(
        "Completed {} iterations query in {:?} {} rows/sec",
        num_iterations, total_duration, rows_per_second
    );
}
