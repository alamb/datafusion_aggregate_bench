use std::{sync::{Arc}, time::Instant};

use datafusion::arrow::{array::{ArrayRef, DictionaryArray, Float64Array, Int64Array, StringArray}, datatypes::Int32Type, record_batch::RecordBatch};
use rand::{Rng, SeedableRng, prelude::{SliceRandom, StdRng}};

use crate::{query::{run_query, run_query_silently}, repeat::RepeatedTable};

mod query;
mod repeat;

#[tokio::main]
async fn main() {
    println!("Starting tests");
    run_benchmark("100 Groups; 1B rows, int64_keys(10% nulls), f64 values(1% nulls)",
                  100000,
                  "select int64_key, count(*), avg(f64) from t group by int64_key").await;

    run_benchmark("100 Groups; 1B rows, utf8_keys(10% nulls), f64 values(1% nulls)",
                  100000, "select utf8_key, count(*), avg(f64) from t group by utf8_key").await;
    run_benchmark("100 Groups; 1B rows, dictionary(utf8, int32) keys(10% nulls), f64 values(1% nulls)",
                  100000, "select dict_key, count(*), avg(f64) from t group by dict_key").await;
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
fn make_batch() -> RecordBatch {
    let mut rng = seedable_rng();
    let batch_size = 10000;
    let num_distinct_keys = 100;
    let key_null_density = 0.10; // 10% nulls
    let value_null_density = 0.01; // 1% nulls

    let key_values = (0..num_distinct_keys)
        .map(|i| {
            if rng.gen::<f64>() < key_null_density {
                None
            } else {
                Some(i)
            }
        })
        .collect::<Vec<Option<i64>>>();

    let int64_array: Int64Array = (0..batch_size)
        .map(|_| {
            // use random numbers to avoid spurious compiler optimizations wrt to branching
            let v: Option<i64> = *key_values.choose(&mut rng).unwrap();
            v
        })
        .collect();

    let key_strings: Vec<Option<String>> = key_values.iter()
        .map(|v| {
            v.map(|v| {
                format!("host-foo-bar-baz-{:?}", v)
            })
        })
        .collect();

    let utf8_array: StringArray = (0..batch_size)
        .map(|_| {
            // use random numbers to avoid spurious compiler optimizations wrt to branching
            let v: Option<&String> = key_strings.choose(&mut rng).unwrap().as_ref();
            v
        })
        .collect();

    let dict_array: DictionaryArray<Int32Type> = (0..batch_size)
        .map(|_| {
            // use random numbers to avoid spurious compiler optimizations wrt to branching
            let v: Option<&String> = key_strings.choose(&mut rng).unwrap().as_ref();
            v.map(|v| v.as_str())
        })
        .collect();

    let f64_array: Float64Array = create_data(batch_size, value_null_density)
        .into_iter()
        .collect();

    RecordBatch::try_from_iter(vec![
        ("int64_key", Arc::new(int64_array) as ArrayRef),
        ("utf8_key", Arc::new(utf8_array) as ArrayRef),
        ("dict_key", Arc::new(dict_array) as ArrayRef),
        ("f64", Arc::new(f64_array) as ArrayRef)
    ]).unwrap()
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
async fn run_benchmark(name: &str, num_batches: usize, query: &str) {
    println!("-------------------");
    println!("{}", name);
    println!("-------------------");
    let batch = make_batch();
    let total_rows = num_batches * batch.num_rows();
    println!("Benchmarking {}", query);
    println!("  {} batches of {} rows = {} total rows",
             num_batches, batch.num_rows(), total_rows);

    let table = RepeatedTable::new(batch, num_batches);
    let table = Arc::new(table);
        run_query(table.clone(), &format!("explain {}", query)).await;

    let num_iterations = 5;
    let mut total_duration = std::time::Duration::new(0,0);
    for _ in 0..num_iterations {
        let start = Instant::now();
        run_query_silently(table.clone(), query).await;
        let duration = Instant::now() - start;
        println!("Completed query in {:?}", duration);
        total_duration += duration;
    }
    let nanos_per_second: f64 = 1_000_000_000.0;
    let rows_per_second: f64 = ((total_rows as f64) / (total_duration.as_nanos() as f64)) *
        nanos_per_second *
        (num_iterations as f64);

    println!("---------------");
    println!("Completed {} iterations query in {:?} {} rows/sec", num_iterations, total_duration, rows_per_second);
}
