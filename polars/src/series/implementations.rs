use super::SeriesTrait;
use crate::datatypes::ArrowDataType;
use crate::prelude::*;
use crate::prelude::*;
use arrow::array::{ArrayDataRef, ArrayRef};
use arrow::buffer::Buffer;
use regex::internal::Input;

impl<'a, T> AsRef<ChunkedArray<T>> for dyn SeriesTrait + 'a
where
    T: 'static + PolarsDataType,
{
    fn as_ref(&self) -> &ChunkedArray<T> {
        if &T::get_data_type() == self.dtype() {
            unsafe { &*(self as *const dyn SeriesTrait as *const ChunkedArray<T>) }
        } else {
            panic!("implementation error")
        }
    }
}

impl<T> SeriesTrait for ChunkedArray<T>
where
    T: 'static + PolarsDataType,
    ChunkedArray<T>: ChunkFilter<T>
        + ChunkTake
        + ChunkOps
        + ChunkExpandAtIndex<T>
        + ToDummies<T>
        + ChunkUnique<T>
        + ChunkSort<T>
        + ChunkReverse<T>
        + ChunkShift<T>
        + ChunkFillNone,
{
    fn array_data(&self) -> Vec<ArrayDataRef> {
        self.array_data()
    }

    /// Get the lengths of the underlying chunks
    fn chunk_lengths(&self) -> &Vec<usize> {
        self.chunk_id()
    }
    /// Name of series.
    fn name(&self) -> &str {
        self.name()
    }

    /// Rename series.
    fn rename(&mut self, name: &str) -> &mut dyn SeriesTrait {
        self.rename(name);
        self
    }

    /// Get field (used in schema)
    fn field(&self) -> &Field {
        self.ref_field()
    }

    /// Get datatype of series.
    fn dtype(&self) -> &ArrowDataType {
        self.field().data_type()
    }

    /// Underlying chunks.
    fn chunks(&self) -> &Vec<ArrayRef> {
        self.chunks()
    }

    /// No. of chunks
    fn n_chunks(&self) -> usize {
        self.chunks().len()
    }

    fn i8(&self) -> Result<&Int8Chunked> {
        if matches!(T::get_data_type(), ArrowDataType::Int8) {
            unsafe { Ok(&*(self as *const dyn SeriesTrait as *const Int8Chunked)) }
        } else {
            Err(PolarsError::DataTypeMisMatch(
                format!(
                    "cannot unpack Series: {:?} of type {:?} into i8",
                    self.name(),
                    self.dtype(),
                )
                .into(),
            ))
        }
    }

    fn i16(&self) -> Result<&Int16Chunked> {
        if matches!(T::get_data_type(), ArrowDataType::Int16) {
            unsafe { Ok(&*(self as *const dyn SeriesTrait as *const Int16Chunked)) }
        } else {
            Err(PolarsError::DataTypeMisMatch(
                format!(
                    "cannot unpack Series: {:?} of type {:?} into i16",
                    self.name(),
                    self.dtype(),
                )
                .into(),
            ))
        }
    }

    /// Unpack to ChunkedArray
    /// ```
    /// # use polars::prelude::*;
    /// let s: Series = [1, 2, 3].iter().collect();
    /// let s_squared: Series = s.i32()
    ///     .unwrap()
    ///     .into_iter()
    ///     .map(|opt_v| {
    ///         match opt_v {
    ///             Some(v) => Some(v * v),
    ///             None => None, // null value
    ///         }
    /// }).collect();
    /// ```
    fn i32(&self) -> Result<&Int32Chunked> {
        unimplemented!()
    }

    /// Unpack to ChunkedArray
    fn i64(&self) -> Result<&Int64Chunked> {
        unimplemented!()
    }

    /// Unpack to ChunkedArray
    fn f32(&self) -> Result<&Float32Chunked> {
        unimplemented!()
    }

    /// Unpack to ChunkedArray
    fn f64(&self) -> Result<&Float64Chunked> {
        unimplemented!()
    }

    /// Unpack to ChunkedArray
    fn u8(&self) -> Result<&UInt8Chunked> {
        unimplemented!()
    }

    /// Unpack to ChunkedArray
    fn u16(&self) -> Result<&UInt16Chunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== u16", self.dtype()).into(),
        ))
    }

    /// Unpack to ChunkedArray
    fn u32(&self) -> Result<&UInt32Chunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== u32", self.dtype()).into(),
        ))
    }

    /// Unpack to ChunkedArray
    fn u64(&self) -> Result<&UInt64Chunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== u32", self.dtype()).into(),
        ))
    }

    /// Unpack to ChunkedArray
    fn bool(&self) -> Result<&BooleanChunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== bool", self.dtype()).into(),
        ))
    }

    /// Unpack to ChunkedArray
    fn utf8(&self) -> Result<&Utf8Chunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== utf8", self.dtype()).into(),
        ))
    }

    /// Unpack to ChunkedArray
    fn date32(&self) -> Result<&Date32Chunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== date32", self.dtype()).into(),
        ))
    }

    /// Unpack to ChunkedArray
    fn date64(&self) -> Result<&Date64Chunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== date64", self.dtype()).into(),
        ))
    }

    /// Unpack to ChunkedArray
    fn time64_nanosecond(&self) -> Result<&Time64NanosecondChunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== time64", self.dtype()).into(),
        ))
    }

    /// Unpack to ChunkedArray
    fn duration_nanosecond(&self) -> Result<&DurationNanosecondChunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== duration_nanosecond", self.dtype()).into(),
        ))
    }

    /// Unpack to ChunkedArray
    fn duration_millisecond(&self) -> Result<&DurationMillisecondChunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== duration_millisecond", self.dtype()).into(),
        ))
    }

    /// Unpack to ChunkedArray
    #[cfg(feature = "dtype-interval")]
    fn interval_daytime(&self) -> Result<&IntervalDayTimeChunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== interval_daytime", self.dtype()).into(),
        ))
    }

    /// Unpack to ChunkedArray
    #[cfg(feature = "dtype-interval")]
    fn interval_year_month(&self) -> Result<&IntervalYearMonthChunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== interval_yearmonth", self.dtype()).into(),
        ))
    }

    /// Unpack to ChunkedArray
    fn list(&self) -> Result<&ListChunked> {
        Err(PolarsError::DataTypeMisMatch(
            format!("{:?} !== list", self.dtype()).into(),
        ))
    }

    fn append_array(&mut self, other: ArrayRef) -> Result<&mut dyn SeriesTrait> {
        self.append_array(other)?;
        Ok(self)
    }

    /// Take `num_elements` from the top as a zero copy view.
    fn limit(&self, num_elements: usize) -> Result<Box<dyn SeriesTrait>> {
        self.limit(num_elements)
            .map(|ca| Box::new(ca) as Box<dyn SeriesTrait>)
    }

    /// Get a zero copy view of the data.
    fn slice(&self, offset: usize, length: usize) -> Result<Box<dyn SeriesTrait>> {
        self.slice(offset, length)
            .map(|ca| Box::new(ca) as Box<dyn SeriesTrait>)
    }

    /// Append a Series of the same type in place.
    fn append(&mut self, other: &dyn SeriesTrait) -> Result<&mut dyn SeriesTrait> {
        if self.dtype() == other.dtype() {
            // todo! add object
            self.append(other.as_ref());
            Ok(self)
        } else {
            Err(PolarsError::DataTypeMisMatch(
                "cannot append Series; data types don't match".into(),
            ))
        }
    }

    /// Filter by boolean mask. This operation clones data.
    fn filter(&self, filter: &BooleanChunked) -> Result<Box<dyn SeriesTrait>> {
        ChunkFilter::filter(self, filter).map(|ca| Box::new(ca) as Box<dyn SeriesTrait>)
    }

    /// Take by index from an iterator. This operation clones the data.
    ///
    /// # Safety
    ///
    /// Out of bounds access doesn't Error but will return a Null value
    fn take_iter(
        &self,
        iter: &mut dyn Iterator<Item = usize>,
        capacity: Option<usize>,
    ) -> Box<dyn SeriesTrait> {
        Box::new(ChunkTake::take(self, iter, capacity))
    }

    /// Take by index from an iterator. This operation clones the data.
    ///
    /// # Safety
    ///
    /// This doesn't check any bounds or null validity.
    unsafe fn take_iter_unchecked(
        &self,
        iter: &mut dyn Iterator<Item = usize>,
        capacity: Option<usize>,
    ) -> Box<dyn SeriesTrait> {
        Box::new(ChunkTake::take_unchecked(self, iter, capacity))
    }

    /// Take by index if ChunkedArray contains a single chunk.
    ///
    /// # Safety
    /// This doesn't check any bounds. Null validity is checked.
    unsafe fn take_from_single_chunked(&self, idx: &UInt32Chunked) -> Result<Box<dyn SeriesTrait>> {
        ChunkTake::take_from_single_chunked(self, idx)
            .map(|ca| Box::new(ca) as Box<dyn SeriesTrait>)
    }

    /// Take by index from an iterator. This operation clones the data.
    ///
    /// # Safety
    ///
    /// This doesn't check any bounds or null validity.
    unsafe fn take_opt_iter_unchecked(
        &self,
        iter: &mut dyn Iterator<Item = Option<usize>>,
        capacity: Option<usize>,
    ) -> Box<dyn SeriesTrait> {
        Box::new(ChunkTake::take_opt_unchecked(self, iter, capacity))
    }

    /// Take by index from an iterator. This operation clones the data.
    ///
    /// # Safety
    ///
    /// Out of bounds access doesn't Error but will return a Null value
    fn take_opt_iter(
        &self,
        iter: &mut dyn Iterator<Item = Option<usize>>,
        capacity: Option<usize>,
    ) -> Box<dyn SeriesTrait> {
        Box::new(ChunkTake::take_opt(self, iter, capacity))
    }

    /// Take by index. This operation is clone.
    ///
    /// # Safety
    ///
    /// Out of bounds access doesn't Error but will return a Null value
    fn take(&self, indices: &dyn AsTakeIndex) -> Box<dyn SeriesTrait> {
        let mut iter = indices.as_take_iter();
        let capacity = indices.take_index_len();
        self.take_iter(&mut iter, Some(capacity))
    }

    /// Get length of series.
    fn len(&self) -> usize {
        self.len()
    }

    /// Check if Series is empty.
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    /// Aggregate all chunks to a contiguous array of memory.
    fn rechunk(&self, chunk_lengths: Option<&[usize]>) -> Result<Box<dyn SeriesTrait>> {
        ChunkOps::rechunk(self, chunk_lengths).map(|ca| Box::new(ca) as Box<dyn SeriesTrait>)
    }

    /// Get the head of the Series.
    fn head(&self, length: Option<usize>) -> Box<dyn SeriesTrait> {
        Box::new(self.head(length))
    }

    /// Get the tail of the Series.
    fn tail(&self, length: Option<usize>) -> Box<dyn SeriesTrait> {
        Box::new(self.tail(length))
    }

    /// Drop all null values and return a new Series.
    fn drop_nulls(&self) -> Box<dyn SeriesTrait> {
        if self.null_count() == 0 {
            Box::new(self.clone())
        } else {
            Box::new(ChunkFilter::filter(self, &self.is_not_null()).unwrap())
        }
    }

    /// Create a new Series filled with values at that index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use polars::prelude::*;
    /// let s = Series::new("a", [0i32, 1, 8]);
    /// let expanded = s.expand_at_index(2, 4);
    /// assert_eq!(Vec::from(expanded.i32().unwrap()), &[Some(8), Some(8), Some(8), Some(8)])
    /// ```
    fn expand_at_index(&self, index: usize, length: usize) -> Box<dyn SeriesTrait> {
        Box::new(ChunkExpandAtIndex::expand_at_index(self, index, length))
    }

    fn cast_with_arrow_datatype(&self, data_type: &ArrowDataType) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }

    /// Create dummy variables. See [DataFrame](DataFrame::to_dummies)
    fn to_dummies(&self) -> Result<DataFrame> {
        ToDummies::to_dummies(self)
    }

    fn value_counts(&self) -> Result<DataFrame> {
        ChunkUnique::value_counts(self)
    }

    /// Get a single value by index. Don't use this operation for loops as a runtime cast is
    /// needed for every iteration.
    fn get(&self, index: usize) -> AnyType {
        self.get_any(index)
    }

    /// Sort in place.
    fn sort_in_place(&mut self, reverse: bool) -> &mut dyn SeriesTrait {
        ChunkSort::sort_in_place(self, reverse);
        self
    }

    fn sort(&self, reverse: bool) -> Box<dyn SeriesTrait> {
        Box::new(ChunkSort::sort(self, reverse))
    }

    /// Retrieve the indexes needed for a sort.
    fn argsort(&self, reverse: bool) -> Vec<usize> {
        ChunkSort::argsort(self, reverse)
    }

    /// Count the null values.
    fn null_count(&self) -> usize {
        self.null_count()
    }

    /// Get unique values in the Series.
    fn unique(&self) -> Result<Box<dyn SeriesTrait>> {
        ChunkUnique::unique(self).map(|ca| Box::new(ca) as Box<dyn SeriesTrait>)
    }

    /// Get unique values in the Series.
    fn n_unique(&self) -> Result<usize> {
        ChunkUnique::n_unique(self)
    }

    /// Get first indexes of unique values.
    fn arg_unique(&self) -> Result<Vec<usize>> {
        ChunkUnique::arg_unique(self)
    }

    /// Get indexes that evaluate true
    fn arg_true(&self) -> Result<UInt32Chunked> {
        unimplemented!()
    }

    /// Get a mask of the null values.
    fn is_null(&self) -> BooleanChunked {
        self.is_null()
    }

    /// Get a mask of the non-null values.
    fn is_not_null(&self) -> BooleanChunked {
        self.is_not_null()
    }

    /// Get a mask of all the unique values.
    fn is_unique(&self) -> Result<BooleanChunked> {
        ChunkUnique::is_unique(self)
    }

    /// Get a mask of all the duplicated values.
    fn is_duplicated(&self) -> Result<BooleanChunked> {
        ChunkUnique::is_duplicated(self)
    }

    /// Get the bits that represent the null values of the underlying ChunkedArray
    fn null_bits(&self) -> Vec<(usize, Option<Buffer>)> {
        self.null_bits()
    }

    /// return a Series in reversed order
    fn reverse(&self) -> Box<dyn SeriesTrait> {
        Box::new(ChunkReverse::reverse(self))
    }

    /// Rechunk and return a pointer to the start of the Series.
    /// Only implemented for numeric types
    fn as_single_ptr(&mut self) -> usize {
        unimplemented!()
    }

    /// Shift the values by a given period and fill the parts that will be empty due to this operation
    /// with `Nones`.
    ///
    /// *NOTE: If you want to fill the Nones with a value use the
    /// [`shift` operation on `ChunkedArray<T>`](../chunked_array/ops/trait.ChunkShift.html).*
    ///
    /// # Example
    ///
    /// ```rust
    /// # use polars::prelude::*;
    /// fn example() -> Result<()> {
    ///     let s = Series::new("series", &[1, 2, 3]);
    ///
    ///     let shifted = s.shift(1)?;
    ///     assert_eq!(Vec::from(shifted.i32()?), &[None, Some(1), Some(2)]);
    ///
    ///     let shifted = s.shift(-1)?;
    ///     assert_eq!(Vec::from(shifted.i32()?), &[Some(2), Some(3), None]);
    ///
    ///     let shifted = s.shift(2)?;
    ///     assert_eq!(Vec::from(shifted.i32()?), &[None, None, Some(1)]);
    ///
    ///     Ok(())
    /// }
    /// example();
    /// ```
    fn shift(&self, periods: i32) -> Result<Box<dyn SeriesTrait>> {
        ChunkShift::shift(self, periods).map(|ca| Box::new(ca) as Box<dyn SeriesTrait>)
    }

    /// Replace None values with one of the following strategies:
    /// * Forward fill (replace None with the previous value)
    /// * Backward fill (replace None with the next value)
    /// * Mean fill (replace None with the mean of the whole array)
    /// * Min fill (replace None with the minimum of the whole array)
    /// * Max fill (replace None with the maximum of the whole array)
    ///
    /// *NOTE: If you want to fill the Nones with a value use the
    /// [`fill_none` operation on `ChunkedArray<T>`](../chunked_array/ops/trait.ChunkFillNone.html)*.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use polars::prelude::*;
    /// fn example() -> Result<()> {
    ///     let s = Series::new("some_missing", &[Some(1), None, Some(2)]);
    ///
    ///     let filled = s.fill_none(FillNoneStrategy::Forward)?;
    ///     assert_eq!(Vec::from(filled.i32()?), &[Some(1), Some(1), Some(2)]);
    ///
    ///     let filled = s.fill_none(FillNoneStrategy::Backward)?;
    ///     assert_eq!(Vec::from(filled.i32()?), &[Some(1), Some(2), Some(2)]);
    ///
    ///     let filled = s.fill_none(FillNoneStrategy::Min)?;
    ///     assert_eq!(Vec::from(filled.i32()?), &[Some(1), Some(1), Some(2)]);
    ///
    ///     let filled = s.fill_none(FillNoneStrategy::Max)?;
    ///     assert_eq!(Vec::from(filled.i32()?), &[Some(1), Some(2), Some(2)]);
    ///
    ///     let filled = s.fill_none(FillNoneStrategy::Mean)?;
    ///     assert_eq!(Vec::from(filled.i32()?), &[Some(1), Some(1), Some(2)]);
    ///
    ///     Ok(())
    /// }
    /// example();
    /// ```
    fn fill_none(&self, strategy: FillNoneStrategy) -> Result<Box<dyn SeriesTrait>> {
        ChunkFillNone::fill_none(self, strategy).map(|ca| Box::new(ca) as Box<dyn SeriesTrait>)
    }

    /// Create a new ChunkedArray with values from self where the mask evaluates `true` and values
    /// from `other` where the mask evaluates `false`
    fn zip_with(&self, mask: &BooleanChunked, other: &Series) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }

    /// Get the sum of the Series as a new Series of length 1.
    fn sum_as_series(&self) -> Series {
        unimplemented!()
    }
    /// Get the max of the Series as a new Series of length 1.
    fn max_as_series(&self) -> Series {
        unimplemented!()
    }
    /// Get the min of the Series as a new Series of length 1.
    fn min_as_series(&self) -> Series {
        unimplemented!()
    }
    /// Get the mean of the Series as a new Series of length 1.
    fn mean_as_series(&self) -> Series {
        unimplemented!()
    }
    /// Get the median of the Series as a new Series of length 1.
    fn median_as_series(&self) -> Series {
        unimplemented!()
    }
    /// Get the variance of the Series as a new Series of length 1.
    fn var_as_series(&self) -> Series {
        unimplemented!()
    }
    /// Get the standard deviation of the Series as a new Series of length 1.
    fn std_as_series(&self) -> Series {
        unimplemented!()
    }
    /// Get the quantile of the ChunkedArray as a new Series of length 1.
    fn quantile_as_series(&self, quantile: f64) -> Result<Series> {
        unimplemented!()
    }
    /// Apply a rolling mean to a Series. See:
    /// [ChunkedArray::rolling_mean](crate::prelude::ChunkWindow::rolling_mean).
    fn rolling_mean(
        &self,
        window_size: usize,
        weight: Option<&[f64]>,
        ignore_null: bool,
    ) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }
    /// Apply a rolling sum to a Series. See:
    /// [ChunkedArray::rolling_mean](crate::prelude::ChunkWindow::rolling_sum).
    fn rolling_sum(
        &self,
        window_size: usize,
        weight: Option<&[f64]>,
        ignore_null: bool,
    ) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }
    /// Apply a rolling min to a Series. See:
    /// [ChunkedArray::rolling_mean](crate::prelude::ChunkWindow::rolling_min).
    fn rolling_min(
        &self,
        window_size: usize,
        weight: Option<&[f64]>,
        ignore_null: bool,
    ) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }
    /// Apply a rolling max to a Series. See:
    /// [ChunkedArray::rolling_mean](crate::prelude::ChunkWindow::rolling_max).
    fn rolling_max(
        &self,
        window_size: usize,
        weight: Option<&[f64]>,
        ignore_null: bool,
    ) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }

    // fn fmt_list(&self) -> String {
    //     unimplemented!()
    // }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract hour from underlying NaiveDateTime representation.
    /// Returns the hour number from 0 to 23.
    fn hour(&self) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract minute from underlying NaiveDateTime representation.
    /// Returns the minute number from 0 to 59.
    fn minute(&self) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract second from underlying NaiveDateTime representation.
    /// Returns the second number from 0 to 59.
    fn second(&self) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract second from underlying NaiveDateTime representation.
    /// Returns the number of nanoseconds since the whole non-leap second.
    /// The range from 1,000,000,000 to 1,999,999,999 represents the leap second.
    fn nanosecond(&self) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract day from underlying NaiveDateTime representation.
    /// Returns the day of month starting from 1.
    ///
    /// The return value ranges from 1 to 31. (The last day of month differs by months.)
    fn day(&self) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Returns the day of year starting from 1.
    ///
    /// The return value ranges from 1 to 366. (The last day of year differs by years.)
    fn ordinal_day(&self) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract month from underlying NaiveDateTime representation.
    /// Returns the month number starting from 1.
    ///
    /// The return value ranges from 1 to 12.
    fn month(&self) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract month from underlying NaiveDateTime representation.
    /// Returns the year number in the calendar date.
    fn year(&self) -> Result<Box<dyn SeriesTrait>> {
        unimplemented!()
    }
}