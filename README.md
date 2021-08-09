# DataFusion Synthetic Aggregate Benchmark


## Why
This benchmark focuses on the actual performance of the
grouping/aggregation algorithms in DataFusion. It does *not* attempt
to mimic real world usecases, but instead is designed to understand
how DataFusion aggregation performs with various shapes of synthetic
input data.

## How

The benchmark code feeds the same `RecordBatch` into a DataFusion
query a large number of times and records the overall execution time


## Running
1. Edit the Cargo.toml file to point at the version of DataFusion you wish to benchmark

2. Run the benchmark
```shell
cargo run --release
```

# Example Output

```
Starting tests
-------------------
100 Groups; 1B rows, dictionary(utf8, int32) keys(10% nulls), f64 values(1% nulls)
-------------------
Benchmarking select dict_key, count(*), avg(f64) from t group by dict_key
  100000 batches of 10000 rows = 1000000000 total rows
explain select dict_key, count(*), avg(f64) from t group by dict_key
+---------------+---------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                          |
+---------------+---------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: #t.dict_key, #COUNT(UInt8(1)), #AVG(t.f64)                                                        |
|               |   Aggregate: groupBy=[[#t.dict_key]], aggr=[[COUNT(UInt8(1)), AVG(#t.f64)]]                                   |
|               |     TableScan: t projection=Some([2, 3])                                                                      |
| physical_plan | ProjectionExec: expr=[dict_key@0 as dict_key, COUNT(UInt8(1))@1 as COUNT(UInt8(1)), AVG(t.f64)@2 as AVG(f64)] |
|               |   HashAggregateExec: mode=Final, gby=[dict_key@0 as dict_key], aggr=[COUNT(UInt8(1)), AVG(f64)]               |
|               |     CoalescePartitionsExec                                                                                    |
|               |       HashAggregateExec: mode=Partial, gby=[dict_key@0 as dict_key], aggr=[COUNT(UInt8(1)), AVG(f64)]         |
|               |         RepartitionExec: partitioning=RoundRobinBatch(16)                                                     |
|               |           RepeatExec repeat=100000                                                                            |
+---------------+---------------------------------------------------------------------------------------------------------------+

Completed query in 9.281193742s
Completed query in 9.456034999s
Completed query in 9.736323143s
Completed query in 10.052034312s
```

The test name
```
-------------------
100 Groups; 1B rows, dictionary(utf8, int32) keys(10% nulls), f64 values(1% nulls)
-------------------
```

Can be read as 1 billion input rows, with 100 distinct group vaues. 10% of the group values are null.
