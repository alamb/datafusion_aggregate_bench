//! In-memory data source for presenting the same record batch N times

use async_trait::async_trait;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;

use std::any::Any;
use std::sync::Arc;
use std::task::{Context, Poll};

use datafusion::arrow::datatypes::{Field, Schema, SchemaRef};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::arrow::error::{Result as ArrowResult};

use datafusion::datasource::TableProvider;
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_plan::Expr;
use datafusion::physical_plan::{DisplayFormatType, RecordBatchStream, SendableRecordBatchStream};
use datafusion::physical_plan::ExecutionPlan;
use datafusion::{
    datasource::datasource::Statistics,
    physical_plan::Partitioning,
};
use futures_util::stream::StreamExt;

/// In-memory table that send a record batch N times
pub struct RepeatedTable {
    batch: RecordBatch,
    /// Number of times to send the batch
    num: usize,
}

impl RepeatedTable {
    pub fn new(batch: RecordBatch,  num: usize) -> Self {
        Self { batch, num }
    }
}

impl TableProvider for RepeatedTable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.batch.schema()
    }

    fn scan(
        &self,
        projection: &Option<Vec<usize>>,
        _batch_size: usize,
        _filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let schema = self.schema();
        let columns: Vec<usize> = match projection {
            Some(p) => p.clone(),
            None => {
                let l = schema.fields().len();
                let mut v = Vec::with_capacity(l);
                for i in 0..l {
                    v.push(i);
                }
                v
            }
        };

        let projected_columns: Result<Vec<Field>> = columns
            .iter()
            .map(|i| {
                if *i < schema.fields().len() {
                    Ok(schema.field(*i).clone())
                } else {
                    Err(DataFusionError::Internal(
                        "Projection index out of range".to_string(),
                    ))
                }
            })
            .collect();

        let projected_schema = Arc::new(Schema::new(projected_columns?));

        let batch = RecordBatch::try_new(
            projected_schema,
            columns.iter().map(|i| self.batch.column(*i).clone()).collect(),
        )?;

        Ok(Arc::new(RepeatExec{
            batch,
            num: self.num,
        }))
    }

    fn statistics(&self) -> Statistics {
        Default::default()
    }
}



/// Execution plan for reading in-memory batches of data
#[derive(Debug)]
pub struct RepeatExec {
    /// The partitions to query
    batch: RecordBatch,
    /// number of times to repeat the batch
    num: usize,
}


#[async_trait]
impl ExecutionPlan for RepeatExec {
    /// Return a reference to Any that can be used for downcasting
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Get the schema for this execution plan
    fn schema(&self) -> SchemaRef {
        self.batch.schema()
    }

    fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
        // this is a leaf node and has no children
        vec![]
    }

    /// Get the output partitioning of this plan
    fn output_partitioning(&self) -> Partitioning {
        Partitioning::UnknownPartitioning(1)
    }

    fn with_new_children(
        &self,
        _: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        Err(DataFusionError::Internal(format!(
            "Children cannot be replaced in {:?}",
            self
        )))
    }

    async fn execute(&self, partition: usize) -> Result<SendableRecordBatchStream> {
        assert_eq!(partition, 0);
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let batch = self.batch.clone();
        let total_batches = self.num;

        tokio::task::spawn(async move {
            for i in 0..total_batches {
                if let Err(_) = tx.send(Ok(batch.clone())).await {
                    println!("Repeat generator got hangup after {}/{} batches", i, total_batches);
                }
            }
        });

        Ok(Box::pin(RepeatedStream {
            schema: self.schema(),
            inner: ReceiverStream::new(rx),
        }))
    }

    fn fmt_as(
        &self,
        t: DisplayFormatType,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        match t {
            DisplayFormatType::Default => {
                write!(f, "RepeatExec repeat={}", self.num)
            }
        }
    }
}



struct RepeatedStream {
    schema: SchemaRef,
    inner: ReceiverStream<ArrowResult<RecordBatch>>,
}

impl Stream for RepeatedStream {
    type Item = ArrowResult<RecordBatch>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx)
    }
}

impl RecordBatchStream for RepeatedStream {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
}
