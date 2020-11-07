// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::prelude::*;
use arrow::datatypes::SchemaRef;
use arrow::error::Result as ArrowResult;
use csv::{Position, StringRecord, StringRecordsIntoIter};
use csv_index::RandomAccessSimple;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::{Regex, RegexBuilder};
use seahash::SeaHasher;
use std::collections::HashSet;
use std::fmt;
use std::hash::BuildHasherDefault;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::Arc;

lazy_static! {
    static ref DECIMAL_RE: Regex = Regex::new(r"^-?(\d+\.\d+)$").unwrap();
    static ref INTEGER_RE: Regex = Regex::new(r"^-?(\d+)$").unwrap();
    static ref BOOLEAN_RE: Regex = RegexBuilder::new(r"^(true)$|^(false)$")
        .case_insensitive(true)
        .build()
        .unwrap();
}

pub trait IntoDF {
    fn into_df(&mut self) -> Result<DataFrame>;
}

/// Infer the data type of a record
fn infer_field_schema(string: &str) -> ArrowDataType {
    // when quoting is enabled in the reader, these quotes aren't escaped, we default to
    // Utf8 for them
    if string.starts_with('"') {
        return ArrowDataType::Utf8;
    }
    // match regex in a particular order
    if BOOLEAN_RE.is_match(string) {
        ArrowDataType::Boolean
    } else if DECIMAL_RE.is_match(string) {
        ArrowDataType::Float64
    } else if INTEGER_RE.is_match(string) {
        ArrowDataType::Int64
    } else {
        ArrowDataType::Utf8
    }
}

/// Infer the schema of a CSV file by reading through the first n records of the file,
/// with `max_read_records` controlling the maximum number of records to read.
///
/// If `max_read_records` is not set, the whole file is read to infer its schema.
///
/// Return infered schema and number of records used for inference.
fn infer_file_schema<R: Read + Seek>(
    reader: &mut R,
    delimiter: u8,
    max_read_records: Option<usize>,
    has_header: bool,
) -> ArrowResult<(Schema, usize)> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_reader(reader);

    // get or create header names
    // when has_header is false, creates default column names with column_ prefix
    let headers: Vec<String> = if has_header {
        let headers = csv_reader.headers()?;
        headers.iter().map(|s| s.to_string()).collect()
    } else {
        let first_record_count = &csv_reader.headers()?.len();
        (0..*first_record_count)
            .map(|i| format!("column_{}", i + 1))
            .collect()
    };

    // save the csv reader position after reading headers
    let position = csv_reader.position().clone();

    let header_length = headers.len();
    // keep track of inferred field types
    let mut column_types: Vec<HashSet<ArrowDataType, BuildHasherDefault<SeaHasher>>> =
        vec![HashSet::with_hasher(BuildHasherDefault::default()); header_length];
    // keep track of columns with nulls
    let mut nulls: Vec<bool> = vec![false; header_length];

    // return csv reader position to after headers
    csv_reader.seek(position)?;

    let mut records_count = 0;
    let mut fields = vec![];

    for result in csv_reader
        .records()
        .take(max_read_records.unwrap_or(std::usize::MAX))
    {
        let record = result?;
        records_count += 1;

        for i in 0..header_length {
            if let Some(string) = record.get(i) {
                if string == "" {
                    nulls[i] = true;
                } else {
                    column_types[i].insert(infer_field_schema(string));
                }
            }
        }
    }

    // build schema from inference results
    for i in 0..header_length {
        let possibilities = &column_types[i];
        let has_nulls = nulls[i];
        let field_name = &headers[i];

        // determine data type based on possible types
        // if there are incompatible types, use DataType::Utf8
        match possibilities.len() {
            1 => {
                for dtype in possibilities.iter() {
                    fields.push(Field::new(&field_name, dtype.clone(), has_nulls));
                }
            }
            2 => {
                if possibilities.contains(&ArrowDataType::Int64)
                    && possibilities.contains(&ArrowDataType::Float64)
                {
                    // we have an integer and double, fall down to double
                    fields.push(Field::new(&field_name, ArrowDataType::Float64, has_nulls));
                } else {
                    // default to Utf8 for conflicting datatypes (e.g bool and int)
                    fields.push(Field::new(&field_name, ArrowDataType::Utf8, has_nulls));
                }
            }
            _ => fields.push(Field::new(&field_name, ArrowDataType::Utf8, has_nulls)),
        }
    }

    // return the reader seek back to the start
    csv_reader.into_inner().seek(SeekFrom::Start(0))?;

    Ok((Schema::new(fields), records_count))
}

fn init_csv_reader<R: Read>(reader: R, has_header: bool, delimiter: u8) -> csv::Reader<R> {
    let mut reader_builder = csv::ReaderBuilder::new();
    reader_builder.has_headers(has_header);
    reader_builder.delimiter(delimiter);
    reader_builder.from_reader(reader)
}

fn take_projection(projection: &mut Option<Vec<usize>>, schema: &SchemaRef) -> Vec<usize> {
    match projection.take() {
        Some(v) => v,
        None => schema.fields().iter().enumerate().map(|(i, _)| i).collect(),
    }
}

fn init_builders(
    projection: &[usize],
    capacity: usize,
    schema: &SchemaRef,
) -> Result<Vec<Builder>> {
    projection
        .iter()
        .map(|&i| field_to_builder(i, capacity, schema))
        .collect()
}

fn accumulate_dataframes(dfs: Vec<DataFrame>) -> Result<DataFrame> {
    let mut iter = dfs.into_iter();
    let mut acc_df = iter.next().unwrap();
    while let Some(df) = iter.next() {
        acc_df.vstack(&df)?;
    }
    Ok(acc_df)
}

macro_rules! impl_add_to_builders {
    ($self:expr, $projection:expr, $rows:expr, $builders:expr) => {{
        $projection
            .par_iter()
            .zip($builders)
            .map(|(i, builder)| {
                let field = $self.schema.field(*i);
                match field.data_type() {
                    ArrowDataType::Boolean => $self.add_to_primitive($rows, *i, builder.bool()),
                    ArrowDataType::Int32 => $self.add_to_primitive($rows, *i, builder.i32()),
                    ArrowDataType::Int64 => $self.add_to_primitive($rows, *i, builder.i64()),
                    ArrowDataType::Float32 => $self.add_to_primitive($rows, *i, builder.f32()),
                    ArrowDataType::Float64 => $self.add_to_primitive($rows, *i, builder.f64()),
                    ArrowDataType::Utf8 => add_to_utf8_builder($rows, *i, builder.utf8()),
                    _ => todo!(),
                }
            })
            .collect::<Result<_>>()?;

        Ok(())
    }};
}

fn field_to_builder(i: usize, capacity: usize, schema: &SchemaRef) -> Result<Builder> {
    let field = schema.field(i);
    let name = field.name();

    let builder = match field.data_type() {
        &ArrowDataType::Boolean => Builder::Boolean(PrimitiveChunkedBuilder::new(name, capacity)),
        &ArrowDataType::Int32 => Builder::Int32(PrimitiveChunkedBuilder::new(name, capacity)),
        &ArrowDataType::Int64 => Builder::Int64(PrimitiveChunkedBuilder::new(name, capacity)),
        &ArrowDataType::Float32 => Builder::Float32(PrimitiveChunkedBuilder::new(name, capacity)),
        &ArrowDataType::Float64 => Builder::Float64(PrimitiveChunkedBuilder::new(name, capacity)),
        &ArrowDataType::Utf8 => Builder::Utf8(Utf8ChunkedBuilder::new(name, capacity)),
        other => {
            return Err(PolarsError::Other(
                format!("Unsupported data type {:?} when reading a csv", other).into(),
            ))
        }
    };
    Ok(builder)
}

fn add_to_utf8_builder(
    rows: &[StringRecord],
    col_idx: usize,
    builder: &mut Utf8ChunkedBuilder,
) -> Result<()> {
    for row in rows.iter() {
        let v = row.get(col_idx);
        builder.append_option(v);
    }
    Ok(())
}

fn builders_to_df(builders: Vec<Builder>) -> DataFrame {
    let columns = builders.into_iter().map(|b| b.into_series()).collect();
    DataFrame::new_no_checks(columns)
}

pub struct ParReader {
    schema: SchemaRef,
    projection: Option<Vec<usize>>,
    batch_size: usize,
    ignore_parser_errors: bool,
    header_offset: usize,
    skip_rows: usize,
    n_rows: Option<usize>,
    file_path: String,
    has_header: bool,
    delimiter: u8,
    indexed_csv: RandomAccessSimple<io::Cursor<Vec<u8>>>,
    n_threads: usize,
}

impl ParReader {
    pub fn from_reader<R: Read>(
        reader: R,
        schema: SchemaRef,
        has_header: bool,
        delimiter: u8,
        batch_size: usize,
        projection: Option<Vec<usize>>,
        ignore_parser_errors: bool,
        n_rows: Option<usize>,
        skip_rows: usize,
        file_path: String,
        n_threads: usize,
    ) -> Self {
        let mut wtr = io::Cursor::new(Vec::with_capacity(batch_size * 128));
        let mut csv_reader = init_csv_reader(reader, has_header, delimiter);
        RandomAccessSimple::create(&mut csv_reader, &mut wtr)
            .expect("could not create index for csv file");
        let indexed_csv = RandomAccessSimple::open(wtr).expect("could not open csv index");
        let header_offset = if has_header { 1 } else { 0 };

        Self {
            schema,
            projection,
            batch_size,
            ignore_parser_errors,
            header_offset,
            skip_rows,
            n_rows,
            file_path,
            has_header,
            delimiter,
            indexed_csv,
            n_threads,
        }
    }

    fn next_rows(
        &self,
        rows: &mut Vec<StringRecord>,
        record_iter: &mut impl Iterator<Item = csv::Result<StringRecord>>,
    ) -> Result<()> {
        for _ in 0..self.batch_size {
            match record_iter.next() {
                Some(Ok(r)) => {
                    rows.push(r);
                }
                Some(Err(e)) => {
                    if self.ignore_parser_errors {
                        continue;
                    } else {
                        return Err(PolarsError::Other(
                            format!("Error parsing line {:?}", e).into(),
                        ));
                    }
                }
                None => break,
            }
        }
        Ok(())
    }

    fn add_to_builders(
        &self,
        builders: &mut [Builder],
        projection: &[usize],
        rows: &[StringRecord],
    ) -> Result<()> {
        impl_add_to_builders!(self, projection, rows, builders)
    }

    fn add_to_primitive<T>(
        &self,
        rows: &[StringRecord],
        col_idx: usize,
        builder: &mut PrimitiveChunkedBuilder<T>,
    ) -> Result<()>
    where
        T: ArrowPrimitiveType,
    {
        let is_boolean_type = *self.schema.field(col_idx).data_type() == ArrowDataType::Boolean;

        for row in rows.iter() {
            match row.get(col_idx) {
                Some(s) => {
                    if s.is_empty() {
                        builder.append_null();
                        continue;
                    }
                    let parsed = if is_boolean_type {
                        s.to_lowercase().parse::<T::Native>()
                    } else {
                        s.parse::<T::Native>()
                    };
                    match parsed {
                        Ok(e) => builder.append_value(e),
                        Err(_) => {
                            if self.ignore_parser_errors {
                                builder.append_null();
                                continue;
                            }
                            return Err(PolarsError::Other(
                                format!("Error while parsing value {} for column {}", s, col_idx,)
                                    .into(),
                            ));
                        }
                    }
                }
                None => builder.append_null(),
            }
        }
        Ok(())
    }

    fn process_thread(
        &self,
        start_pos: Position,
        chunk_size: usize,
        projection: &[usize],
    ) -> Result<DataFrame> {
        let f = std::fs::File::open(&self.file_path).expect("csv file");
        let mut csv_reader = init_csv_reader(f, false, self.delimiter);
        csv_reader
            .seek(start_pos)
            .expect("position should be there");

        let mut builders = init_builders(projection, chunk_size, &self.schema)?;
        let mut record_iter = csv_reader.records().take(chunk_size);
        // we reuse this container to amortize allocations
        let mut rows = Vec::with_capacity(self.batch_size);

        loop {
            rows.clear();
            self.next_rows(&mut rows, &mut record_iter)?;
            // stop when the whole file is processed
            if rows.len() == 0 {
                break;
            }

            self.add_to_builders(&mut builders, &projection, &rows)?;
        }
        Ok(builders_to_df(builders))
    }
}

impl IntoDF for ParReader {
    fn into_df(&mut self) -> Result<DataFrame> {
        // only take projections once
        let projection = take_projection(&mut self.projection, &self.schema);
        let offset = self.skip_rows + self.header_offset;
        let end = match self.n_rows {
            Some(n) => std::cmp::min(offset + n, self.indexed_csv.len() as usize),
            None => self.indexed_csv.len() as usize,
        };

        let chunk_size = (end - offset) / self.n_threads;
        let positions = (0..self.n_threads)
            .into_iter()
            .map(|i| {
                let start_row = offset + i * chunk_size;
                let start_pos = self.indexed_csv.get(start_row as u64)?;
                Ok(start_pos)
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        // batches should not be bigger than chunks
        self.batch_size = std::cmp::min(self.batch_size, chunk_size);

        let mut dfs = positions
            .into_iter()
            .enumerate()
            .map(|(idx, start_pos)| {
                self.process_thread(start_pos, chunk_size, &projection)
                    .map(|df| (idx, df))
            })
            .collect::<Result<Vec<_>>>()?;

        dfs.sort_by_key(|tpl| tpl.0);

        accumulate_dataframes(dfs.into_iter().map(|(_, df)| df).collect())
    }
}

/// CSV file reader
pub struct SequentialReader<R: Read> {
    /// Explicit schema for the CSV file
    schema: SchemaRef,
    /// Optional projection for which columns to load (zero-based column indices)
    projection: Option<Vec<usize>>,
    /// File reader
    record_iter: StringRecordsIntoIter<R>,
    /// Batch size (number of records to load each time)
    batch_size: usize,
    /// Current line number, used in error reporting
    line_number: usize,
    ignore_parser_errors: bool,
    header_offset: usize,
    skip_rows: usize,
    n_rows: Option<usize>,
    capacity: usize,
}

impl<R: Read + Sync> IntoDF for SequentialReader<R> {
    fn into_df(&mut self) -> Result<DataFrame> {
        let mut total_capacity = self.capacity;
        if self.skip_rows > 0 {
            for _ in 0..self.skip_rows {
                self.line_number += 1;
                let _ = self.record_iter.next();
            }
            total_capacity += self.skip_rows;
        }

        // only take projections once
        let projection = take_projection(&mut self.projection, &self.schema);
        self.batch_size = std::cmp::min(self.batch_size, self.capacity);
        if let Some(n) = self.n_rows {
            self.batch_size = std::cmp::min(self.batch_size, n);
        }

        let mut builders = init_builders(&projection, self.capacity, &self.schema)?;
        // we reuse this container to amortize allocations
        let mut rows = Vec::with_capacity(self.batch_size);
        let mut parsed_dfs = Vec::with_capacity(128);
        loop {
            rows.clear();
            self.next_rows(&mut rows)?;
            // stop when the whole file is processed
            if rows.len() == 0 {
                break;
            }
            if (self.line_number - self.header_offset) > total_capacity {
                let mut builders_tmp = init_builders(&projection, self.capacity, &self.schema)?;
                std::mem::swap(&mut builders_tmp, &mut builders);
                parsed_dfs.push(builders_to_df(builders_tmp));
                total_capacity += self.capacity;
            }

            self.add_to_builders(&mut builders, &projection, &rows)?;

            // stop after n_rows are processed
            if let Some(n_rows) = self.n_rows {
                if self.line_number >= n_rows {
                    break;
                }
            }
        }
        parsed_dfs.push(builders_to_df(builders));
        accumulate_dataframes(parsed_dfs)
    }
}

impl<R> fmt::Debug for SequentialReader<R>
where
    R: Read,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reader")
            .field("schema", &self.schema)
            .field("projection", &self.projection)
            .field("batch_size", &self.batch_size)
            .field("line_number", &self.line_number)
            .finish()
    }
}

impl<R: Read + Sync> SequentialReader<R> {
    /// Returns the schema of the reader, useful for getting the schema without reading
    /// record batches
    pub fn schema(&self) -> SchemaRef {
        match &self.projection {
            Some(projection) => {
                let fields = self.schema.fields();
                let projected_fields: Vec<Field> =
                    projection.iter().map(|i| fields[*i].clone()).collect();

                Arc::new(Schema::new(projected_fields))
            }
            None => self.schema.clone(),
        }
    }

    /// Create a new CsvReader from a `BufReader<R: Read>
    ///
    /// This constructor allows you more flexibility in what records are processed by the
    /// csv reader.
    pub fn from_reader(
        reader: R,
        schema: SchemaRef,
        has_header: bool,
        delimiter: u8,
        batch_size: usize,
        projection: Option<Vec<usize>>,
        ignore_parser_errors: bool,
        n_rows: Option<usize>,
        skip_rows: usize,
        capacity: usize,
    ) -> Self {
        let csv_reader = init_csv_reader(reader, has_header, delimiter);
        let record_iter = csv_reader.into_records();

        let header_offset = if has_header { 1 } else { 0 };

        Self {
            schema,
            projection,
            record_iter,
            batch_size,
            line_number: if has_header { 1 } else { 0 },
            ignore_parser_errors,
            header_offset,
            skip_rows,
            n_rows,
            capacity,
        }
    }

    fn next_rows(&mut self, rows: &mut Vec<StringRecord>) -> Result<()> {
        for i in 0..self.batch_size {
            self.line_number += 1;
            match self.record_iter.next() {
                Some(Ok(r)) => {
                    rows.push(r);
                }
                Some(Err(e)) => {
                    if self.ignore_parser_errors {
                        continue;
                    } else {
                        return Err(PolarsError::Other(
                            format!("Error parsing line {}: {:?}", self.line_number + i, e).into(),
                        ));
                    }
                }
                None => break,
            }
        }
        Ok(())
    }

    fn add_to_primitive<T>(
        &self,
        rows: &[StringRecord],
        col_idx: usize,
        builder: &mut PrimitiveChunkedBuilder<T>,
    ) -> Result<()>
    where
        T: ArrowPrimitiveType,
    {
        let is_boolean_type = *self.schema.field(col_idx).data_type() == ArrowDataType::Boolean;

        if (rows.len() + builder.len()) > builder.capacity() {
            builder.reserve(builder.capacity() * 2)
        }

        for (row_index, row) in rows.iter().enumerate() {
            match row.get(col_idx) {
                Some(s) => {
                    if s.is_empty() {
                        builder.append_null();
                        continue;
                    }
                    let parsed = if is_boolean_type {
                        s.to_lowercase().parse::<T::Native>()
                    } else {
                        s.parse::<T::Native>()
                    };
                    match parsed {
                        Ok(e) => builder.append_value(e),
                        Err(_) => {
                            if self.ignore_parser_errors {
                                builder.append_null();
                                continue;
                            }
                            return Err(PolarsError::Other(
                                format!(
                                    // TODO: we should surface the underlying error here.
                                    "Error while parsing value {} for column {} at line {}",
                                    s,
                                    col_idx,
                                    self.line_number + row_index
                                )
                                .into(),
                            ));
                        }
                    }
                }
                None => builder.append_null(),
            }
        }
        Ok(())
    }

    fn add_to_builders(
        &self,
        builders: &mut [Builder],
        projection: &[usize],
        rows: &[StringRecord],
    ) -> Result<()> {
        impl_add_to_builders!(self, projection, rows, builders)
    }
}

enum Builder {
    Boolean(PrimitiveChunkedBuilder<BooleanType>),
    Int32(PrimitiveChunkedBuilder<Int32Type>),
    Int64(PrimitiveChunkedBuilder<Int64Type>),
    Float32(PrimitiveChunkedBuilder<Float32Type>),
    Float64(PrimitiveChunkedBuilder<Float64Type>),
    Utf8(Utf8ChunkedBuilder),
}

impl Builder {
    fn bool(&mut self) -> &mut PrimitiveChunkedBuilder<BooleanType> {
        match self {
            Builder::Boolean(builder) => builder,
            _ => panic!("implementation error"),
        }
    }
    fn i32(&mut self) -> &mut PrimitiveChunkedBuilder<Int32Type> {
        match self {
            Builder::Int32(builder) => builder,
            _ => panic!("implementation error"),
        }
    }
    fn i64(&mut self) -> &mut PrimitiveChunkedBuilder<Int64Type> {
        match self {
            Builder::Int64(builder) => builder,
            _ => panic!("implementation error"),
        }
    }
    fn f64(&mut self) -> &mut PrimitiveChunkedBuilder<Float64Type> {
        match self {
            Builder::Float64(builder) => builder,
            _ => panic!("implementation error"),
        }
    }
    fn f32(&mut self) -> &mut PrimitiveChunkedBuilder<Float32Type> {
        match self {
            Builder::Float32(builder) => builder,
            _ => panic!("implementation error"),
        }
    }
    fn utf8(&mut self) -> &mut Utf8ChunkedBuilder {
        match self {
            Builder::Utf8(builder) => builder,
            _ => panic!("implementation error"),
        }
    }

    fn into_series(self) -> Series {
        use Builder::*;
        match self {
            Utf8(b) => b.finish().into(),
            Int32(b) => b.finish().into(),
            Int64(b) => b.finish().into(),
            Float32(b) => b.finish().into(),
            Float64(b) => b.finish().into(),
            Boolean(b) => b.finish().into(),
        }
    }
}

pub fn build_csv_reader<R: 'static + Read + Seek + Sync>(
    mut reader: R,
    n_rows: Option<usize>,
    skip_rows: usize,
    projection: Option<Vec<usize>>,
    batch_size: usize,
    max_records: Option<usize>,
    delimiter: Option<u8>,
    has_header: bool,
    ignore_parser_errors: bool,
    schema: Option<SchemaRef>,
    mut file_name: Option<String>,
    n_threads: Option<usize>,
) -> Result<Box<dyn IntoDF>> {
    // check if schema should be inferred
    let delimiter = delimiter.unwrap_or(b',');
    let schema = match schema {
        Some(schema) => schema,
        None => {
            let (inferred_schema, _) =
                infer_file_schema(&mut reader, delimiter, max_records, has_header)?;

            Arc::new(inferred_schema)
        }
    };

    // We opt for sequential path because parallel does first an expensive indexing.
    // This may be the wrong choice.
    if let Some(_) = n_rows {
        file_name = None
    }

    match file_name {
        Some(file_name) => Ok(Box::new(ParReader::from_reader(
            reader,
            schema,
            has_header,
            delimiter,
            batch_size,
            projection,
            ignore_parser_errors,
            n_rows,
            skip_rows,
            file_name,
            n_threads.unwrap_or(4),
        ))),
        None => {
            let capacity = match n_rows {
                Some(n) => n,
                None => 512 * 1024,
            };
            Ok(Box::new(SequentialReader::from_reader(
                reader,
                schema,
                has_header,
                delimiter,
                batch_size,
                projection,
                ignore_parser_errors,
                n_rows,
                skip_rows,
                capacity,
            )))
        }
    }
}