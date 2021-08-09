# Summary
Here are the results of running 5 iterations of the benchmark Aug 9, 2021:

|test                                                                                    | master  | [arrow-datafusion](https://github.com/apache/arrow-datafusion/pull/808)| `gby_null` / `master` (less than 1 is better) |
| ----------- | ----------- |------------ | ------- |
| `100 Groups; 100M rows, int64_keys(10% nulls), f64 values(1% nulls)`                   | 22.40s  | 16.27s | .73  |
| `100 Groups; 100M rows, utf8_keys(10% nulls),  f64 values(1% nulls)`                   | 29.46s  | 22.73s | .77  |
| `100 Groups; 100M rows, dictionary(utf8, int32) keys(10% nulls), f64 values(1% nulls)` | 31.54s  | 26.96s | .85  |


# Full Results

## master

Using https://github.com/apache/arrow-datafusion/commit/0125451e5fc194b1b1e4828bae5350bcd8ac24f9

With this in `Cargo.toml`:
```toml
datafusion = { git = "https://github.com/apache/arrow-datafusion", branch = "master" }
```

```
Starting tests
-------------------
100 Groups; 100M rows, int64_keys(10% nulls), f64 values(1% nulls)
-------------------
Benchmarking select int64_key, count(*), avg(f64) from t group by int64_key
  10000 batches of 10000 rows = 100000000 total rows
explain select int64_key, count(*), avg(f64) from t group by int64_key
+---------------+-----------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                            |
+---------------+-----------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: #t.int64_key, #COUNT(UInt8(1)), #AVG(t.f64)                                                         |
|               |   Aggregate: groupBy=[[#t.int64_key]], aggr=[[COUNT(UInt8(1)), AVG(#t.f64)]]                                    |
|               |     TableScan: t projection=Some([0, 3])                                                                        |
| physical_plan | ProjectionExec: expr=[int64_key@0 as int64_key, COUNT(UInt8(1))@1 as COUNT(UInt8(1)), AVG(t.f64)@2 as AVG(f64)] |
|               |   HashAggregateExec: mode=Final, gby=[int64_key@0 as int64_key], aggr=[COUNT(UInt8(1)), AVG(f64)]               |
|               |     HashAggregateExec: mode=Partial, gby=[int64_key@0 as int64_key], aggr=[COUNT(UInt8(1)), AVG(f64)]           |
|               |       RepeatExec repeat=10000                                                                                   |
+---------------+-----------------------------------------------------------------------------------------------------------------+

Completed query in 4.375033329s
Completed query in 4.413096795s
Completed query in 4.299649817s
Completed query in 4.506798722s
Completed query in 4.801338377s
---------------
Completed 5 iterations query in 22.39591704s 22325497.951567695 rows/sec
-------------------
100 Groups; 100M rows, utf8_keys(10% nulls), f64 values(1% nulls)
-------------------
Benchmarking select utf8_key, count(*), avg(f64) from t group by utf8_key
  10000 batches of 10000 rows = 100000000 total rows
explain select utf8_key, count(*), avg(f64) from t group by utf8_key
+---------------+---------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                          |
+---------------+---------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: #t.utf8_key, #COUNT(UInt8(1)), #AVG(t.f64)                                                        |
|               |   Aggregate: groupBy=[[#t.utf8_key]], aggr=[[COUNT(UInt8(1)), AVG(#t.f64)]]                                   |
|               |     TableScan: t projection=Some([1, 3])                                                                      |
| physical_plan | ProjectionExec: expr=[utf8_key@0 as utf8_key, COUNT(UInt8(1))@1 as COUNT(UInt8(1)), AVG(t.f64)@2 as AVG(f64)] |
|               |   HashAggregateExec: mode=Final, gby=[utf8_key@0 as utf8_key], aggr=[COUNT(UInt8(1)), AVG(f64)]               |
|               |     HashAggregateExec: mode=Partial, gby=[utf8_key@0 as utf8_key], aggr=[COUNT(UInt8(1)), AVG(f64)]           |
|               |       RepeatExec repeat=10000                                                                                 |
+---------------+---------------------------------------------------------------------------------------------------------------+

Completed query in 5.936522669s
Completed query in 5.491342286s
Completed query in 5.653198467s
Completed query in 5.975725304s
Completed query in 6.403009113s
---------------
Completed 5 iterations query in 29.459797839s 16972282.115869816 rows/sec
-------------------
100 Groups; 100M rows, dictionary(utf8, int32) keys(10% nulls), f64 values(1% nulls)
-------------------
Benchmarking select dict_key, count(*), avg(f64) from t group by dict_key
  10000 batches of 10000 rows = 100000000 total rows
explain select dict_key, count(*), avg(f64) from t group by dict_key
+---------------+---------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                          |
+---------------+---------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: #t.dict_key, #COUNT(UInt8(1)), #AVG(t.f64)                                                        |
|               |   Aggregate: groupBy=[[#t.dict_key]], aggr=[[COUNT(UInt8(1)), AVG(#t.f64)]]                                   |
|               |     TableScan: t projection=Some([2, 3])                                                                      |
| physical_plan | ProjectionExec: expr=[dict_key@0 as dict_key, COUNT(UInt8(1))@1 as COUNT(UInt8(1)), AVG(t.f64)@2 as AVG(f64)] |
|               |   HashAggregateExec: mode=Final, gby=[dict_key@0 as dict_key], aggr=[COUNT(UInt8(1)), AVG(f64)]               |
|               |     HashAggregateExec: mode=Partial, gby=[dict_key@0 as dict_key], aggr=[COUNT(UInt8(1)), AVG(f64)]           |
|               |       RepeatExec repeat=10000                                                                                 |
+---------------+---------------------------------------------------------------------------------------------------------------+

Completed query in 6.16188868s
Completed query in 6.027779697s
Completed query in 6.407653852s
Completed query in 6.686255709s
Completed query in 6.260518519s
---------------
Completed 5 iterations query in 31.544096457s 15850826.498758193 rows/sec

Compilation finished at Mon Aug  9 15:21:35
```






## `alamb/gby_null_new`

Using code in https://github.com/apache/arrow-datafusion/pull/808

https://github.com/apache/arrow-datafusion/pull/808/commits/0051a85c893032c4798ce80ba3cd5190b2805b75

With this in `Cargo.toml`:
```toml
datafusion = { git = "https://github.com/alamb/arrow-datafusion", branch = "alamb/gby_null_new" }
```

```
Starting tests
-------------------
100 Groups; 100M rows, int64_keys(10% nulls), f64 values(1% nulls)
-------------------
Benchmarking select int64_key, count(*), avg(f64) from t group by int64_key
  10000 batches of 10000 rows = 100000000 total rows
explain select int64_key, count(*), avg(f64) from t group by int64_key
+---------------+-----------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                            |
+---------------+-----------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: #t.int64_key, #COUNT(UInt8(1)), #AVG(t.f64)                                                         |
|               |   Aggregate: groupBy=[[#t.int64_key]], aggr=[[COUNT(UInt8(1)), AVG(#t.f64)]]                                    |
|               |     TableScan: t projection=Some([0, 3])                                                                        |
| physical_plan | ProjectionExec: expr=[int64_key@0 as int64_key, COUNT(UInt8(1))@1 as COUNT(UInt8(1)), AVG(t.f64)@2 as AVG(f64)] |
|               |   HashAggregateExec: mode=Final, gby=[int64_key@0 as int64_key], aggr=[COUNT(UInt8(1)), AVG(f64)]               |
|               |     HashAggregateExec: mode=Partial, gby=[int64_key@0 as int64_key], aggr=[COUNT(UInt8(1)), AVG(f64)]           |
|               |       RepeatExec repeat=10000                                                                                   |
+---------------+-----------------------------------------------------------------------------------------------------------------+

Completed query in 3.370525261s
Completed query in 3.15826233s
Completed query in 3.397078165s
Completed query in 3.208393253s
Completed query in 3.131489631s
---------------
Completed 5 iterations query in 16.26574864s 30739439.73106915 rows/sec
-------------------
100 Groups; 100M rows, utf8_keys(10% nulls), f64 values(1% nulls)
-------------------
Benchmarking select utf8_key, count(*), avg(f64) from t group by utf8_key
  10000 batches of 10000 rows = 100000000 total rows
explain select utf8_key, count(*), avg(f64) from t group by utf8_key
+---------------+---------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                          |
+---------------+---------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: #t.utf8_key, #COUNT(UInt8(1)), #AVG(t.f64)                                                        |
|               |   Aggregate: groupBy=[[#t.utf8_key]], aggr=[[COUNT(UInt8(1)), AVG(#t.f64)]]                                   |
|               |     TableScan: t projection=Some([1, 3])                                                                      |
| physical_plan | ProjectionExec: expr=[utf8_key@0 as utf8_key, COUNT(UInt8(1))@1 as COUNT(UInt8(1)), AVG(t.f64)@2 as AVG(f64)] |
|               |   HashAggregateExec: mode=Final, gby=[utf8_key@0 as utf8_key], aggr=[COUNT(UInt8(1)), AVG(f64)]               |
|               |     HashAggregateExec: mode=Partial, gby=[utf8_key@0 as utf8_key], aggr=[COUNT(UInt8(1)), AVG(f64)]           |
|               |       RepeatExec repeat=10000                                                                                 |
+---------------+---------------------------------------------------------------------------------------------------------------+

Completed query in 4.503244288s
Completed query in 4.817943639s
Completed query in 4.638462703s
Completed query in 4.353616266s
Completed query in 4.418183557s
---------------
Completed 5 iterations query in 22.731450453s 21995956.704734262 rows/sec
-------------------
100 Groups; 100M rows, dictionary(utf8, int32) keys(10% nulls), f64 values(1% nulls)
-------------------
Benchmarking select dict_key, count(*), avg(f64) from t group by dict_key
  10000 batches of 10000 rows = 100000000 total rows
explain select dict_key, count(*), avg(f64) from t group by dict_key
+---------------+---------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                          |
+---------------+---------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: #t.dict_key, #COUNT(UInt8(1)), #AVG(t.f64)                                                        |
|               |   Aggregate: groupBy=[[#t.dict_key]], aggr=[[COUNT(UInt8(1)), AVG(#t.f64)]]                                   |
|               |     TableScan: t projection=Some([2, 3])                                                                      |
| physical_plan | ProjectionExec: expr=[dict_key@0 as dict_key, COUNT(UInt8(1))@1 as COUNT(UInt8(1)), AVG(t.f64)@2 as AVG(f64)] |
|               |   HashAggregateExec: mode=Final, gby=[dict_key@0 as dict_key], aggr=[COUNT(UInt8(1)), AVG(f64)]               |
|               |     HashAggregateExec: mode=Partial, gby=[dict_key@0 as dict_key], aggr=[COUNT(UInt8(1)), AVG(f64)]           |
|               |       RepeatExec repeat=10000                                                                                 |
+---------------+---------------------------------------------------------------------------------------------------------------+

Completed query in 5.46970814s
Completed query in 5.475138137s
Completed query in 5.480583278s
Completed query in 5.320554152s
Completed query in 5.21063748s
---------------
Completed 5 iterations query in 26.956621187s 18548318.668406714 rows/sec

Compilation finished at Mon Aug  9 15:18:14

```
