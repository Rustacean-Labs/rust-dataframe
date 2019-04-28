//! Lazy dataframe

use crate::expression::*;
use arrow::datatypes::{DataType, Schema};
use arrow::error::ArrowError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A lazy dataframe
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LazyFrame {
    id: String,
    pub(crate) expression: Expression,
    output: Dataset,
}

impl LazyFrame {
    /// lazily read a dataset
    ///
    /// This should eventually take the inputs that make up the computation
    pub fn read(computation: Computation) -> Self {
        Self {
            id: "".to_owned(),
            output: computation.output.clone(),
            expression: Expression::Read(computation),
        }
    }

    pub fn write(&self) -> Result<(), ()> {
        // a write operation evaluates the expression, and returns a write status
        Err(())
    }

    /// Prints a subset of the data frame to console
    pub fn display(&self, limit: usize) -> Result<(), ()> {
        // display is like write, except it just shows results as a table
        self.limit(limit);
        Err(())
    }

    pub fn schema(&self) -> Arc<Schema> {
        unimplemented!()
    }

    /// Create a column from the operation
    pub fn with_column(
        &self,
        col_name: &str,
        function: Function,
        input_col_names: Vec<&str>,
        as_type: Option<DataType>,
    ) -> Result<Self, ArrowError> {
        // the columns that make the output dataset
        let ops = Operation::calculate(
            &self.output,
            input_col_names,
            Function::Scalar(ScalarFunction::Sine),
            Some(col_name.to_owned()),
            as_type,
        )?;
        let mut out_dataset: Dataset = self.output.clone();
        for tfm in &ops {
            match tfm {
                Transformation::Aggregate => panic!("can't create column from aggregation"),
                Transformation::Calculate(op) => {
                    out_dataset = out_dataset.append_column(op.output.clone())
                }
                _ => panic!("can't create column from {:?}", tfm),
            }
        }
        Ok(Self {
            id: "".to_owned(),
            output: out_dataset.clone(),
            expression: Expression::Compute(
                Box::new(self.expression.clone()),
                Computation {
                    input: vec![self.output.clone()],
                    transformations: ops.clone(),
                    output: out_dataset,
                },
            ),
        })
    }

    pub fn with_column_renamed(&self, old_name: &str, new_name: &str) -> Self {
        // create a transformation that renames a column's name
        let column = self.output.get_column(old_name);
        match column {
            Some((index, column)) => {
                let mut columns = self.output.columns.clone();
                // rename column
                let rename = Operation::rename(column, new_name);
                columns[index] = Column {
                    name: new_name.to_owned(),
                    column_type: column.column_type.clone(),
                };
                let output = Dataset {
                    name: "renamed_dataset".to_owned(),
                    columns: columns.clone(),
                };

                let computation = Computation {
                    input: vec![self.output.clone()],
                    transformations: vec![Transformation::Calculate(rename)],
                    output: output.clone(),
                };
                let expression =
                    Expression::Compute(Box::new(self.expression.clone()), computation);
                Self {
                    id: "renamed_frame".to_owned(),
                    expression,
                    output,
                }
            }
            None => self.clone(),
        }
    }

    pub fn limit(&self, limit: usize) -> Self {
        unimplemented!()
    }

    /// project columns
    pub fn select(&self, col_names: Vec<&str>) -> Self {
        unimplemented!()
    }

    pub fn drop(&self, col_names: Vec<&str>) -> Self {
        unimplemented!()
    }

    pub fn join(&self, other: &Self) -> Self {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_pipeline() {
        let reader = Reader {
            source: DataSourceType::Csv(
                "./test/data/uk_cities_with_headers.csv".to_owned(),
                CsvReadOptions {
                    has_headers: true,
                    batch_size: 1024,
                    delimiter: None,
                    max_records: Some(1024),
                    projection: None,
                },
            ),
        };
        let compute = Computation::compute_read(&reader);
        // read data
        let mut frame = LazyFrame::read(compute);
        // rename a column
        frame = frame.with_column_renamed("city", "town");
        // add a column as a calculation of 2 columns
        frame = frame
            .with_column(
                "sin_lat",
                Function::Scalar(ScalarFunction::Sine),
                vec!["lat"],
                None,
            )
            .unwrap();
        frame = frame
            .with_column(
                "sin_lng",
                Function::Scalar(ScalarFunction::Sine),
                vec!["lng"],
                None,
            )
            .unwrap();
        dbg!(frame);
    }
}
