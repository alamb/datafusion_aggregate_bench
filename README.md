# DataFusion Synthetic Aggregate Benchmark


## Why
This benchmark focuses on the actual performance of the grouping/aggregation algorithms in DataFusion

It does not attempt to mimic real world usecases, but instead is designed to understand how DataFusion aggregation performs with various shapes of synthetic input data


## Running
1. Edit the Cargo.toml file to point at the version of DataFusion you wish to benchmark

```shell
cargo run --release
```

# Example Output

## Case Descriptions:

This code feeds the same record batch into a DataFusion query, some number of times.

int64:
