//! Type agnostic columnar data structure.
pub use crate::prelude::ChunkCompare;
use crate::prelude::*;
use arrow::{array::ArrayRef, buffer::Buffer};
use std::mem;
pub(crate) mod aggregate;
pub(crate) mod arithmetic;
mod comparison;
pub mod implementations;
pub(crate) mod iterator;
#[allow(clippy::missing_safety_doc)]
pub mod ops;

use self::ops::SeriesOps;
use crate::chunked_array::builder::get_list_builder;
use crate::fmt::FmtList;
use crate::frame::group_by::IntoGroupTuples;
use crate::series::implementations::Wrap;
use arrow::array::ArrayDataRef;
use std::sync::Arc;

pub(crate) mod private {
    use super::*;
    use crate::frame::group_by::PivotAgg;

    pub(crate) trait Agg {
        fn agg_mean(&self, groups: &[(usize, Vec<usize>)]) -> Option<Arc<dyn SeriesTrait>>;
        fn agg_min(&self, groups: &[(usize, Vec<usize>)]) -> Option<Arc<dyn SeriesTrait>>;
        fn agg_max(&self, groups: &[(usize, Vec<usize>)]) -> Option<Arc<dyn SeriesTrait>>;
        fn agg_sum(&self, groups: &[(usize, Vec<usize>)]) -> Option<Arc<dyn SeriesTrait>>;
        fn agg_first(&self, groups: &[(usize, Vec<usize>)]) -> Arc<dyn SeriesTrait>;
        fn agg_last(&self, groups: &[(usize, Vec<usize>)]) -> Arc<dyn SeriesTrait>;
        fn agg_n_unique(&self, groups: &[(usize, Vec<usize>)]) -> Option<UInt32Chunked>;
        fn agg_list(&self, groups: &[(usize, Vec<usize>)]) -> Option<Arc<dyn SeriesTrait>>;
        fn agg_quantile(
            &self,
            groups: &[(usize, Vec<usize>)],
            _quantile: f64,
        ) -> Option<Arc<dyn SeriesTrait>>;
        fn agg_median(&self, groups: &[(usize, Vec<usize>)]) -> Option<Arc<dyn SeriesTrait>>;
        fn pivot<'a>(
            &self,
            pivot_series: &'a (dyn SeriesTrait + 'a),
            keys: Vec<Arc<dyn SeriesTrait>>,
            groups: &[(usize, Vec<usize>)],
            agg_type: PivotAgg,
        ) -> Result<DataFrame>;

        fn pivot_count<'a>(
            &self,
            pivot_series: &'a (dyn SeriesTrait + 'a),
            keys: Vec<Arc<dyn SeriesTrait>>,
            groups: &[(usize, Vec<usize>)],
        ) -> Result<DataFrame>;
    }
}

pub trait SeriesTrait: Send + Sync + private::Agg {
    fn group_tuples(&self) -> Vec<(usize, Vec<usize>)>;
    /// Get Arrow ArrayData
    fn array_data(&self) -> Vec<ArrayDataRef> {
        unimplemented!()
    }

    /// Get the lengths of the underlying chunks
    fn chunk_lengths(&self) -> &Vec<usize> {
        unimplemented!()
    }
    /// Name of series.
    fn name(&self) -> &str {
        unimplemented!()
    }

    /// Rename series.
    fn rename(&mut self, _name: &str) -> &mut dyn SeriesTrait {
        unimplemented!()
    }

    /// Get field (used in schema)
    fn field(&self) -> &Field {
        unimplemented!()
    }

    /// Get datatype of series.
    fn dtype(&self) -> &ArrowDataType {
        self.field().data_type()
    }

    /// Underlying chunks.
    fn chunks(&self) -> &Vec<ArrayRef> {
        unimplemented!()
    }

    /// No. of chunks
    fn n_chunks(&self) -> usize {
        self.chunks().len()
    }

    fn i8(&self) -> Result<&Int8Chunked> {
        unimplemented!()
    }

    fn i16(&self) -> Result<&Int16Chunked> {
        unimplemented!()
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

    fn append_array(&mut self, _other: ArrayRef) -> Result<&mut dyn SeriesTrait> {
        unimplemented!()
    }

    /// Take `num_elements` from the top as a zero copy view.
    fn limit(&self, _num_elements: usize) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    /// Get a zero copy view of the data.
    fn slice(&self, offset: usize, length: usize) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    /// Append a Series of the same type in place.
    fn append(&mut self, other: &dyn SeriesTrait) -> Result<&mut dyn SeriesTrait>;

    /// Filter by boolean mask. This operation clones data.
    fn filter(&self, _filter: &BooleanChunked) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    /// Take by index from an iterator. This operation clones the data.
    ///
    /// # Safety
    ///
    /// Out of bounds access doesn't Error but will return a Null value
    fn take_iter(
        &self,
        _iter: &mut dyn Iterator<Item = usize>,
        _capacity: Option<usize>,
    ) -> Arc<dyn SeriesTrait> {
        unimplemented!()
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
    ) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }

    /// Take by index if ChunkedArray contains a single chunk.
    ///
    /// # Safety
    /// This doesn't check any bounds. Null validity is checked.
    unsafe fn take_from_single_chunked(&self, idx: &UInt32Chunked) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
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
    ) -> Arc<dyn SeriesTrait> {
        unimplemented!()
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
    ) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }

    /// Take by index. This operation is clone.
    ///
    /// # Safety
    ///
    /// Out of bounds access doesn't Error but will return a Null value
    fn take(&self, indices: &dyn AsTakeIndex) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }

    /// Get length of series.
    fn len(&self) -> usize {
        unimplemented!()
    }

    /// Check if Series is empty.
    fn is_empty(&self) -> bool {
        unimplemented!()
    }

    /// Aggregate all chunks to a contiguous array of memory.
    fn rechunk(&self, chunk_lengths: Option<&[usize]>) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    /// Get the head of the Series.
    fn head(&self, length: Option<usize>) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }

    /// Get the tail of the Series.
    fn tail(&self, length: Option<usize>) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }

    /// Drop all null values and return a new Series.
    fn drop_nulls(&self) -> Arc<dyn SeriesTrait> {
        unimplemented!()
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
    fn expand_at_index(&self, index: usize, length: usize) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }

    fn cast_with_arrow_datatype(&self, data_type: &ArrowDataType) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    /// Create dummy variables. See [DataFrame](DataFrame::to_dummies)
    fn to_dummies(&self) -> Result<DataFrame> {
        unimplemented!()
    }

    fn value_counts(&self) -> Result<DataFrame> {
        unimplemented!()
    }

    /// Get a single value by index. Don't use this operation for loops as a runtime cast is
    /// needed for every iteration.
    fn get(&self, index: usize) -> AnyType {
        unimplemented!()
    }

    /// Sort in place.
    fn sort_in_place(&mut self, reverse: bool) -> &mut dyn SeriesTrait {
        unimplemented!()
    }

    fn sort(&self, reverse: bool) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }

    /// Retrieve the indexes needed for a sort.
    fn argsort(&self, reverse: bool) -> Vec<usize> {
        unimplemented!()
    }

    /// Count the null values.
    fn null_count(&self) -> usize {
        unimplemented!()
    }

    /// Get unique values in the Series.
    fn unique(&self) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    /// Get unique values in the Series.
    fn n_unique(&self) -> Result<usize> {
        unimplemented!()
    }

    /// Get first indexes of unique values.
    fn arg_unique(&self) -> Result<Vec<usize>> {
        unimplemented!()
    }

    /// Get indexes that evaluate true
    fn arg_true(&self) -> Result<UInt32Chunked> {
        unimplemented!()
    }

    /// Get a mask of the null values.
    fn is_null(&self) -> BooleanChunked {
        unimplemented!()
    }

    /// Get a mask of the non-null values.
    fn is_not_null(&self) -> BooleanChunked {
        unimplemented!()
    }

    /// Get a mask of all the unique values.
    fn is_unique(&self) -> Result<BooleanChunked> {
        unimplemented!()
    }

    /// Get a mask of all the duplicated values.
    fn is_duplicated(&self) -> Result<BooleanChunked> {
        unimplemented!()
    }

    /// Get the bits that represent the null values of the underlying ChunkedArray
    fn null_bits(&self) -> Vec<(usize, Option<Buffer>)> {
        unimplemented!()
    }

    /// return a Series in reversed order
    fn reverse(&self) -> Arc<dyn SeriesTrait> {
        unimplemented!()
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
    fn shift(&self, periods: i32) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
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
    fn fill_none(&self, strategy: FillNoneStrategy) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    /// Create a new ChunkedArray with values from self where the mask evaluates `true` and values
    /// from `other` where the mask evaluates `false`
    fn zip_with(
        &self,
        mask: &BooleanChunked,
        other: &dyn SeriesTrait,
    ) -> Result<Arc<dyn SeriesTrait>>;

    /// Get the sum of the Series as a new Series of length 1.
    fn sum_as_series(&self) -> Arc<dyn SeriesTrait>;
    /// Get the max of the Series as a new Series of length 1.
    fn max_as_series(&self) -> Arc<dyn SeriesTrait>;
    /// Get the min of the Series as a new Series of length 1.
    fn min_as_series(&self) -> Arc<dyn SeriesTrait>;
    /// Get the mean of the Series as a new Series of length 1.
    fn mean_as_series(&self) -> Arc<dyn SeriesTrait>;
    /// Get the median of the Series as a new Series of length 1.
    fn median_as_series(&self) -> Arc<dyn SeriesTrait>;
    /// Get the variance of the Series as a new Series of length 1.
    fn var_as_series(&self) -> Arc<dyn SeriesTrait>;
    /// Get the standard deviation of the Series as a new Series of length 1.
    fn std_as_series(&self) -> Arc<dyn SeriesTrait>;
    /// Get the quantile of the ChunkedArray as a new Series of length 1.
    fn quantile_as_series(&self, quantile: f64) -> Result<Arc<dyn SeriesTrait>>;
    /// Apply a rolling mean to a Series. See:
    /// [ChunkedArray::rolling_mean](crate::prelude::ChunkWindow::rolling_mean).
    fn rolling_mean(
        &self,
        window_size: usize,
        weight: Option<&[f64]>,
        ignore_null: bool,
    ) -> Result<Arc<dyn SeriesTrait>>;
    /// Apply a rolling sum to a Series. See:
    /// [ChunkedArray::rolling_mean](crate::prelude::ChunkWindow::rolling_sum).
    fn rolling_sum(
        &self,
        window_size: usize,
        weight: Option<&[f64]>,
        ignore_null: bool,
    ) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }
    /// Apply a rolling min to a Series. See:
    /// [ChunkedArray::rolling_mean](crate::prelude::ChunkWindow::rolling_min).
    fn rolling_min(
        &self,
        window_size: usize,
        weight: Option<&[f64]>,
        ignore_null: bool,
    ) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }
    /// Apply a rolling max to a Series. See:
    /// [ChunkedArray::rolling_mean](crate::prelude::ChunkWindow::rolling_max).
    fn rolling_max(
        &self,
        window_size: usize,
        weight: Option<&[f64]>,
        ignore_null: bool,
    ) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    fn fmt_list(&self) -> String {
        "fmt implemented".into()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract hour from underlying NaiveDateTime representation.
    /// Returns the hour number from 0 to 23.
    fn hour(&self) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract minute from underlying NaiveDateTime representation.
    /// Returns the minute number from 0 to 59.
    fn minute(&self) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract second from underlying NaiveDateTime representation.
    /// Returns the second number from 0 to 59.
    fn second(&self) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract second from underlying NaiveDateTime representation.
    /// Returns the number of nanoseconds since the whole non-leap second.
    /// The range from 1,000,000,000 to 1,999,999,999 represents the leap second.
    fn nanosecond(&self) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract day from underlying NaiveDateTime representation.
    /// Returns the day of month starting from 1.
    ///
    /// The return value ranges from 1 to 31. (The last day of month differs by months.)
    fn day(&self) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Returns the day of year starting from 1.
    ///
    /// The return value ranges from 1 to 366. (The last day of year differs by years.)
    fn ordinal_day(&self) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract month from underlying NaiveDateTime representation.
    /// Returns the month number starting from 1.
    ///
    /// The return value ranges from 1 to 12.
    fn month(&self) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract month from underlying NaiveDateTime representation.
    /// Returns the year number in the calendar date.
    fn year(&self) -> Result<Arc<dyn SeriesTrait>> {
        unimplemented!()
    }
    fn clone(&self) -> Arc<dyn SeriesTrait>;
}

impl dyn SeriesTrait {
    pub fn unpack<N>(&self) -> Result<&ChunkedArray<N>>
    where
        N: PolarsDataType,
    {
        if N::get_data_type() == self.dtype() {
            Ok(self.as_ref())
        } else {
            Err(PolarsError::DataTypeMisMatch(
                "cannot unpack Series; data types don't match".into(),
            ))
        }
    }
}

/// # Series
/// The columnar data type for a DataFrame. The [Series enum](enum.Series.html) consists
/// of typed [ChunkedArray](../chunked_array/struct.ChunkedArray.html)'s. To quickly cast
/// a `Series` to a `ChunkedArray` you can call the method with the name of the type:
///
/// ```
/// # use polars::prelude::*;
/// let s: Series = [1, 2, 3].iter().collect();
/// // Quickly obtain the ChunkedArray wrapped by the Series.
/// let chunked_array = s.i32().unwrap();
/// ```
///
/// ## Arithmetic
///
/// You can do standard arithmetic on series.
/// ```
/// # use polars::prelude::*;
/// let s: Series = [1, 2, 3].iter().collect();
/// let out_add = &s + &s;
/// let out_sub = &s - &s;
/// let out_div = &s / &s;
/// let out_mul = &s * &s;
/// ```
///
/// Or with series and numbers.
///
/// ```
/// # use polars::prelude::*;
/// let s: Series = (1..3).collect();
/// let out_add_one = &s + 1;
/// let out_multiply = &s * 10;
///
/// // Could not overload left hand side operator.
/// let out_divide = 1.div(&s);
/// let out_add = 1.add(&s);
/// let out_subtract = 1.sub(&s);
/// let out_multiply = 1.mul(&s);
/// ```
///
/// ## Comparison
/// You can obtain boolean mask by comparing series.
///
/// ```
/// # use polars::prelude::*;
/// use itertools::Itertools;
/// let s = Series::new("dollars", &[1, 2, 3]);
/// let mask = s.eq(1);
/// let valid = [true, false, false].iter();
/// assert!(mask
///     .into_iter()
///     .map(|opt_bool| opt_bool.unwrap()) // option, because series can be null
///     .zip(valid)
///     .all(|(a, b)| a == *b))
/// ```
///
/// See all the comparison operators in the [CmpOps trait](../chunked_array/comparison/trait.CmpOps.html)
///
/// ## Iterators
/// The Series variants contain differently typed [ChunkedArray's](../chunked_array/struct.ChunkedArray.html).
/// These structs can be turned into iterators, making it possible to use any function/ closure you want
/// on a Series.
///
/// These iterators return an `Option<T>` because the values of a series may be null.
///
/// ```
/// use polars::prelude::*;
/// let pi = 3.14;
/// let s = Series::new("angle", [2f32 * pi, pi, 1.5 * pi].as_ref());
/// let s_cos: Series = s.f32()
///                     .expect("series was not an f32 dtype")
///                     .into_iter()
///                     .map(|opt_angle| opt_angle.map(|angle| angle.cos()))
///                     .collect();
/// ```
///
/// ## Creation
/// Series can be create from different data structures. Below we'll show a few ways we can create
/// a Series object.
///
/// ```
/// # use polars::prelude::*;
/// // Series van be created from Vec's, slices and arrays
/// Series::new("boolean series", &vec![true, false, true]);
/// Series::new("int series", &[1, 2, 3]);
/// // And can be nullable
/// Series::new("got nulls", &[Some(1), None, Some(2)]);
///
/// // Series can also be collected from iterators
/// let from_iter: Series = (0..10)
///     .into_iter()
///     .collect();
///
/// ```
pub enum Series {
    UInt8(ChunkedArray<UInt8Type>),
    UInt16(ChunkedArray<UInt16Type>),
    UInt32(ChunkedArray<UInt32Type>),
    UInt64(ChunkedArray<UInt64Type>),
    Int8(ChunkedArray<Int8Type>),
    Int16(ChunkedArray<Int16Type>),
    Int32(ChunkedArray<Int32Type>),
    Int64(ChunkedArray<Int64Type>),
    Float32(ChunkedArray<Float32Type>),
    Float64(ChunkedArray<Float64Type>),
    Utf8(ChunkedArray<Utf8Type>),
    Bool(ChunkedArray<BooleanType>),
    Date32(ChunkedArray<Date32Type>),
    Date64(ChunkedArray<Date64Type>),
    Time64Nanosecond(ChunkedArray<Time64NanosecondType>),
    DurationNanosecond(ChunkedArray<DurationNanosecondType>),
    DurationMillisecond(DurationMillisecondChunked),
    #[cfg(feature = "dtype-interval")]
    #[doc(cfg(feature = "dtype-interval"))]
    IntervalDayTime(IntervalDayTimeChunked),
    #[cfg(feature = "dtype-interval")]
    #[doc(cfg(feature = "dtype-interval"))]
    IntervalYearMonth(IntervalYearMonthChunked),
    List(ListChunked),
    Object(Box<dyn SeriesOps>),
}

impl Clone for Series {
    fn clone(&self) -> Self {
        if let Series::Object(ca) = self {
            Series::Object((*ca).clone())
        } else {
            apply_method_all_arrow_series_and_return!(self, clone, [],)
        }
    }
}

macro_rules! unpack_series {
    ($self:ident, $variant:ident, $ty:expr) => {
        if let Series::$variant(ca) = $self {
            Ok(ca)
        } else {
            Err(PolarsError::DataTypeMisMatch(
                format!(
                    "cannot unpack Series: {:?} of type {:?} into {:?}",
                    $self.name(),
                    $self.dtype(),
                    $ty
                )
                .into(),
            ))
        }
    };
}

impl Series {
    /// Get Arrow ArrayData
    pub fn array_data(&self) -> Vec<ArrayDataRef> {
        // apply_method_all_arrow_series!(self, array_data,)
        unimplemented!()
    }

    pub fn from_chunked_array<T: PolarsDataType>(ca: ChunkedArray<T>) -> Self {
        pack_ca_to_series(ca)
    }

    /// Get the lengths of the underlying chunks
    pub fn chunk_lengths(&self) -> &Vec<usize> {
        apply_method_all_series!(self, chunk_id,)
    }
    /// Name of series.
    pub fn name(&self) -> &str {
        apply_method_all_series!(self, name,)
    }

    /// Rename series.
    pub fn rename(&mut self, name: &str) -> &mut Self {
        apply_method_all_series!(self, rename, name);
        self
    }

    /// Get field (used in schema)
    pub fn field(&self) -> &Field {
        apply_method_all_series!(self, ref_field,)
    }

    /// Get datatype of series.
    pub fn dtype(&self) -> &ArrowDataType {
        self.field().data_type()
    }

    /// Underlying chunks.
    pub fn chunks(&self) -> &Vec<ArrayRef> {
        apply_method_all_series!(self, chunks,)
    }

    /// No. of chunks
    pub fn n_chunks(&self) -> usize {
        self.chunks().len()
    }

    pub fn i8(&self) -> Result<&Int8Chunked> {
        unpack_series!(self, Int8, "i8")
    }

    pub fn i16(&self) -> Result<&Int16Chunked> {
        unpack_series!(self, Int16, "i16")
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
    pub fn i32(&self) -> Result<&Int32Chunked> {
        unpack_series!(self, Int32, "i32")
    }

    /// Unpack to ChunkedArray
    pub fn i64(&self) -> Result<&Int64Chunked> {
        unpack_series!(self, Int64, "i64")
    }

    /// Unpack to ChunkedArray
    pub fn f32(&self) -> Result<&Float32Chunked> {
        unpack_series!(self, Float32, "f32")
    }

    /// Unpack to ChunkedArray
    pub fn f64(&self) -> Result<&Float64Chunked> {
        unpack_series!(self, Float64, "f64")
    }

    /// Unpack to ChunkedArray
    pub fn u8(&self) -> Result<&UInt8Chunked> {
        unpack_series!(self, UInt8, "u8")
    }

    /// Unpack to ChunkedArray
    pub fn u16(&self) -> Result<&UInt16Chunked> {
        unpack_series!(self, UInt16, "u16")
    }

    /// Unpack to ChunkedArray
    pub fn u32(&self) -> Result<&UInt32Chunked> {
        unpack_series!(self, UInt32, "u32")
    }

    /// Unpack to ChunkedArray
    pub fn u64(&self) -> Result<&UInt64Chunked> {
        unpack_series!(self, UInt64, "u64")
    }

    /// Unpack to ChunkedArray
    pub fn bool(&self) -> Result<&BooleanChunked> {
        unpack_series!(self, Bool, "bool")
    }

    /// Unpack to ChunkedArray
    pub fn utf8(&self) -> Result<&Utf8Chunked> {
        unpack_series!(self, Utf8, "utf8")
    }

    /// Unpack to ChunkedArray
    pub fn date32(&self) -> Result<&Date32Chunked> {
        unpack_series!(self, Date32, "date32")
    }

    /// Unpack to ChunkedArray
    pub fn date64(&self) -> Result<&Date64Chunked> {
        unpack_series!(self, Date64, "date64")
    }

    /// Unpack to ChunkedArray
    pub fn time64_nanosecond(&self) -> Result<&Time64NanosecondChunked> {
        unpack_series!(self, Time64Nanosecond, "time64nanosecond")
    }

    /// Unpack to ChunkedArray
    pub fn duration_nanosecond(&self) -> Result<&DurationNanosecondChunked> {
        unpack_series!(self, DurationNanosecond, "durationnanosecond")
    }

    /// Unpack to ChunkedArray
    pub fn duration_millisecond(&self) -> Result<&DurationMillisecondChunked> {
        unpack_series!(self, DurationMillisecond, "durationmillisecond")
    }

    /// Unpack to ChunkedArray
    #[cfg(feature = "dtype-interval")]
    #[doc(cfg(feature = "dtype-interval"))]
    pub fn interval_daytime(&self) -> Result<&IntervalDayTimeChunked> {
        unpack_series!(self, IntervalDayTime, "intervaldaytime")
    }

    /// Unpack to ChunkedArray
    #[cfg(feature = "dtype-interval")]
    #[doc(cfg(feature = "dtype-interval"))]
    pub fn interval_year_month(&self) -> Result<&IntervalYearMonthChunked> {
        unpack_series!(self, IntervalYearMonth, "intervalyearmonth")
    }

    /// Unpack to ChunkedArray
    pub fn list(&self) -> Result<&ListChunked> {
        unpack_series!(self, List, "list")
    }

    pub fn append_array(&mut self, other: ArrayRef) -> Result<&mut Self> {
        // apply_method_all_arrow_series!(self, append_array, other)?;
        // Ok(self)
        unimplemented!()
    }

    /// Take `num_elements` from the top as a zero copy view.
    pub fn limit(&self, num_elements: usize) -> Result<Self> {
        // Ok(apply_method_all_series_and_return!(self, limit, [num_elements], ?))
        unimplemented!()
    }

    /// Get a zero copy view of the data.
    pub fn slice(&self, offset: usize, length: usize) -> Result<Self> {
        // Ok(apply_method_all_series_and_return!(self, slice, [offset, length], ?))
        unimplemented!()
    }

    /// Append a Series of the same type in place.
    pub fn append(&mut self, other: &Self) -> Result<&mut Self> {
        // if self.dtype() == other.dtype() {
        //     // todo! add object
        //     apply_method_all_arrow_series!(self, append, other.as_ref());
        //     Ok(self)
        // } else {
        //     Err(PolarsError::DataTypeMisMatch(
        //         "cannot append Series; data types don't match".into(),
        //     ))
        // }
        unimplemented!()
    }

    /// Filter by boolean mask. This operation clones data.
    pub fn filter<T: AsRef<BooleanChunked>>(&self, filter: T) -> Result<Self> {
        // Ok(apply_method_all_series_and_return!(self, filter, [filter.as_ref()], ?))
        unimplemented!()
    }

    /// Take by index from an iterator. This operation clones the data.
    ///
    /// # Safety
    ///
    /// Out of bounds access doesn't Error but will return a Null value
    pub fn take_iter(
        &self,
        mut iter: impl Iterator<Item = usize>,
        capacity: Option<usize>,
    ) -> Self {
        // if let Series::Object(so) = self {
        //     so.take(&mut iter, capacity).into()
        // } else {
        //     apply_method_all_arrow_series_and_return!(self, take, [iter, capacity],)
        // }
        unimplemented!()
    }

    /// Take by index from an iterator. This operation clones the data.
    ///
    /// # Safety
    ///
    /// This doesn't check any bounds or null validity.
    pub unsafe fn take_iter_unchecked(
        &self,
        mut iter: impl Iterator<Item = usize>,
        capacity: Option<usize>,
    ) -> Self {
        // if let Series::Object(so) = self {
        //     so.take_unchecked(&mut iter, capacity).into()
        // } else {
        //     apply_method_all_arrow_series_and_return!(self, take_unchecked, [iter, capacity],)
        // }
        unimplemented!()
    }

    /// Take by index if ChunkedArray contains a single chunk.
    ///
    /// # Safety
    /// This doesn't check any bounds. Null validity is checked.
    pub unsafe fn take_from_single_chunked(&self, idx: &UInt32Chunked) -> Result<Self> {
        // let s = apply_method_all_arrow_series_and_return!(self, take_from_single_chunked, [idx], ?);
        // Ok(s)
        unimplemented!()
    }

    /// Take by index from an iterator. This operation clones the data.
    ///
    /// # Safety
    ///
    /// This doesn't check any bounds or null validity.
    pub unsafe fn take_opt_iter_unchecked(
        &self,
        mut iter: impl Iterator<Item = Option<usize>>,
        capacity: Option<usize>,
    ) -> Self {
        // if let Series::Object(so) = self {
        //     so.take_opt_unchecked(&mut iter, capacity).into()
        // } else {
        //     apply_method_all_arrow_series_and_return!(self, take_opt_unchecked, [iter, capacity],)
        // }
        unimplemented!()
    }

    /// Take by index from an iterator. This operation clones the data.
    ///
    /// # Safety
    ///
    /// Out of bounds access doesn't Error but will return a Null value
    pub fn take_opt_iter(
        &self,
        mut iter: impl Iterator<Item = Option<usize>>,
        capacity: Option<usize>,
    ) -> Self {
        // if let Series::Object(so) = self {
        //     so.take_opt(&mut iter, capacity).into()
        // } else {
        //     apply_method_all_arrow_series_and_return!(self, take_opt, [iter, capacity],)
        // }
        unimplemented!()
    }

    /// Take by index. This operation is clone.
    ///
    /// # Safety
    ///
    /// Out of bounds access doesn't Error but will return a Null value
    pub fn take<T: AsTakeIndex>(&self, indices: &T) -> Self {
        let mut iter = indices.as_take_iter();
        let capacity = indices.take_index_len();
        self.take_iter(&mut iter, Some(capacity))
    }

    /// Get length of series.
    pub fn len(&self) -> usize {
        // apply_method_all_series!(self, len,)
        unimplemented!()
    }

    /// Check if Series is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Aggregate all chunks to a contiguous array of memory.
    pub fn rechunk(&self, chunk_lengths: Option<&[usize]>) -> Result<Self> {
        // Ok(apply_method_all_series_and_return!(self, rechunk, [chunk_lengths], ?))
        unimplemented!()
    }

    /// Get the head of the Series.
    pub fn head(&self, length: Option<usize>) -> Self {
        // apply_method_all_series_and_return!(self, head, [length],)
        unimplemented!()
    }

    /// Get the tail of the Series.
    pub fn tail(&self, length: Option<usize>) -> Self {
        // apply_method_all_series_and_return!(self, tail, [length],)
        unimplemented!()
    }

    /// Drop all null values and return a new Series.
    pub fn drop_nulls(&self) -> Self {
        if self.null_count() == 0 {
            self.clone()
        } else {
            self.filter(&self.is_not_null()).unwrap()
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
    pub fn expand_at_index(&self, index: usize, length: usize) -> Self {
        // apply_method_all_series_and_return!(self, expand_at_index, [index, length],)
        unimplemented!()
    }

    /// Cast to some primitive type.
    pub fn cast<N>(&self) -> Result<Self>
    where
        N: PolarsDataType,
    {
        let s = match self {
            Series::Bool(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::Utf8(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::UInt8(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::UInt16(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::UInt32(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::UInt64(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::Int8(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::Int16(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::Int32(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::Int64(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::Float32(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::Float64(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::Date32(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::Date64(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::Time64Nanosecond(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::DurationNanosecond(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::DurationMillisecond(arr) => pack_ca_to_series(arr.cast::<N>()?),
            #[cfg(feature = "dtype-interval")]
            Series::IntervalDayTime(arr) => pack_ca_to_series(arr.cast::<N>()?),
            #[cfg(feature = "dtype-interval")]
            Series::IntervalYearMonth(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::List(arr) => pack_ca_to_series(arr.cast::<N>()?),
            Series::Object(_) => return Err(PolarsError::Other("cannot cast object".into())),
        };
        Ok(s)
    }

    pub fn cast_with_arrow_datatype(&self, data_type: &ArrowDataType) -> Result<Self> {
        use ArrowDataType::*;
        match data_type {
            Boolean => self.cast::<BooleanType>(),
            Utf8 => self.cast::<Utf8Type>(),
            UInt8 => self.cast::<UInt8Type>(),
            UInt16 => self.cast::<UInt16Type>(),
            UInt32 => self.cast::<UInt32Type>(),
            UInt64 => self.cast::<UInt64Type>(),
            Int8 => self.cast::<Int8Type>(),
            Int16 => self.cast::<Int16Type>(),
            Int32 => self.cast::<Int32Type>(),
            Int64 => self.cast::<Int64Type>(),
            Float32 => self.cast::<Float32Type>(),
            Float64 => self.cast::<Float64Type>(),
            Date32(_) => self.cast::<Date32Type>(),
            Date64(_) => self.cast::<Date64Type>(),
            Time64(TimeUnit::Nanosecond) => self.cast::<Time64NanosecondType>(),
            Duration(TimeUnit::Nanosecond) => self.cast::<DurationNanosecondType>(),
            Duration(TimeUnit::Millisecond) => self.cast::<DurationMillisecondType>(),
            #[cfg(feature = "dtype-interval")]
            Interval(IntervalUnit::DayTime) => self.cast::<IntervalDayTimeType>(),
            #[cfg(feature = "dtype-interval")]
            Interval(IntervalUnit::YearMonth) => self.cast::<IntervalYearMonthType>(),
            List(_) => self.cast::<ListType>(),
            dt => Err(PolarsError::Other(
                format!("Casting to {:?} is not supported", dt).into(),
            )),
        }
    }

    /// Create dummy variables. See [DataFrame](DataFrame::to_dummies)
    pub fn to_dummies(&self) -> Result<DataFrame> {
        // apply_method_all_arrow_series!(self, to_dummies,)
        unimplemented!()
    }

    pub fn value_counts(&self) -> Result<DataFrame> {
        // apply_method_all_arrow_series!(self, value_counts,)
        unimplemented!()
    }

    /// Get the `ChunkedArray` for some `PolarsDataType`
    #[allow(clippy::transmute_ptr_to_ptr)]
    pub fn unpack<N>(&self) -> Result<&ChunkedArray<N>>
    where
        N: PolarsDataType,
    {
        macro_rules! unpack_if_match {
            ($ca:ident) => {{
                if *$ca.dtype() == N::get_data_type() {
                    unsafe { Ok(mem::transmute::<_, &ChunkedArray<N>>($ca)) }
                } else {
                    Err(PolarsError::DataTypeMisMatch(
                        "cannot unpack Series; data types don't match".into(),
                    ))
                }
            }};
        }
        match self {
            Series::Bool(arr) => unpack_if_match!(arr),
            Series::Utf8(arr) => unpack_if_match!(arr),
            Series::UInt8(arr) => unpack_if_match!(arr),
            Series::UInt16(arr) => unpack_if_match!(arr),
            Series::UInt32(arr) => unpack_if_match!(arr),
            Series::UInt64(arr) => unpack_if_match!(arr),
            Series::Int8(arr) => unpack_if_match!(arr),
            Series::Int16(arr) => unpack_if_match!(arr),
            Series::Int32(arr) => unpack_if_match!(arr),
            Series::Int64(arr) => unpack_if_match!(arr),
            Series::Float32(arr) => unpack_if_match!(arr),
            Series::Float64(arr) => unpack_if_match!(arr),
            Series::Date32(arr) => unpack_if_match!(arr),
            Series::Date64(arr) => unpack_if_match!(arr),
            Series::Time64Nanosecond(arr) => unpack_if_match!(arr),
            Series::DurationNanosecond(arr) => unpack_if_match!(arr),
            Series::DurationMillisecond(arr) => unpack_if_match!(arr),
            #[cfg(feature = "dtype-interval")]
            Series::IntervalDayTime(arr) => unpack_if_match!(arr),
            #[cfg(feature = "dtype-interval")]
            Series::IntervalYearMonth(arr) => unpack_if_match!(arr),
            Series::List(arr) => unpack_if_match!(arr),
            Series::Object(arr) => unpack_if_match!(arr),
        }
    }

    /// Get a single value by index. Don't use this operation for loops as a runtime cast is
    /// needed for every iteration.
    pub fn get(&self, index: usize) -> AnyType {
        // apply_method_all_series!(self, get_any, index)
        unimplemented!()
    }

    /// Sort in place.
    pub fn sort_in_place(&mut self, reverse: bool) -> &mut Self {
        // apply_method_all_arrow_series!(self, sort_in_place, reverse);
        // self
        unimplemented!()
    }

    pub fn sort(&self, reverse: bool) -> Self {
        // apply_method_all_arrow_series_and_return!(self, sort, [reverse],)
        unimplemented!()
    }

    /// Retrieve the indexes needed for a sort.
    pub fn argsort(&self, reverse: bool) -> Vec<usize> {
        // apply_method_all_arrow_series!(self, argsort, reverse)
        unimplemented!()
    }

    /// Count the null values.
    pub fn null_count(&self) -> usize {
        // apply_method_all_series!(self, null_count,)
        unimplemented!()
    }

    /// Get unique values in the Series.
    pub fn unique(&self) -> Result<Self> {
        // Ok(apply_method_all_arrow_series_and_return!(self, unique, [],?))
        unimplemented!()
    }

    /// Get unique values in the Series.
    pub fn n_unique(&self) -> Result<usize> {
        // apply_method_all_arrow_series!(self, n_unique,)
        unimplemented!()
    }

    /// Get first indexes of unique values.
    pub fn arg_unique(&self) -> Result<Vec<usize>> {
        // apply_method_all_arrow_series!(self, arg_unique,)
        unimplemented!()
    }

    /// Get indexes that evaluate true
    pub fn arg_true(&self) -> Result<UInt32Chunked> {
        match self {
            Series::Bool(ca) => Ok(ca.arg_true()),
            _ => Err(PolarsError::DataTypeMisMatch(
                format!("Expected Boolean, got {:?}", self.dtype()).into(),
            )),
        }
    }

    /// Get a mask of the null values.
    pub fn is_null(&self) -> BooleanChunked {
        // apply_method_all_series!(self, is_null,)
        unimplemented!()
    }

    /// Get a mask of the non-null values.
    pub fn is_not_null(&self) -> BooleanChunked {
        // apply_method_all_series!(self, is_not_null,)
        unimplemented!()
    }

    /// Get a mask of all the unique values.
    pub fn is_unique(&self) -> Result<BooleanChunked> {
        // apply_method_all_arrow_series!(self, is_unique,)
        unimplemented!()
    }

    /// Get a mask of all the duplicated values.
    pub fn is_duplicated(&self) -> Result<BooleanChunked> {
        // apply_method_all_arrow_series!(self, is_duplicated,)
        unimplemented!()
    }

    /// Get the bits that represent the null values of the underlying ChunkedArray
    pub fn null_bits(&self) -> Vec<(usize, Option<Buffer>)> {
        // apply_method_all_arrow_series!(self, null_bits,)
        unimplemented!()
    }

    /// return a Series in reversed order
    pub fn reverse(&self) -> Self {
        // apply_method_all_series_and_return!(self, reverse, [],)
        unimplemented!()
    }

    /// Rechunk and return a pointer to the start of the Series.
    /// Only implemented for numeric types
    pub fn as_single_ptr(&mut self) -> usize {
        // apply_method_numeric_series!(self, as_single_ptr,)
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
    pub fn shift(&self, periods: i32) -> Result<Self> {
        // Ok(apply_method_all_series_and_return!(self, shift, [periods, &None],?))
        unimplemented!()
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
    pub fn fill_none(&self, strategy: FillNoneStrategy) -> Result<Self> {
        // Ok(apply_method_all_series_and_return!(self, fill_none, [strategy],?))
        unimplemented!()
    }

    /// Create a new ChunkedArray with values from self where the mask evaluates `true` and values
    /// from `other` where the mask evaluates `false`
    pub fn zip_with(&self, mask: &BooleanChunked, other: &Series) -> Result<Self> {
        // Ok(apply_method_all_series_and_return!(self, zip_with_series, [mask, other],?))
        unimplemented!()
    }

    /// Get the sum of the Series as a new Series of length 1.
    pub fn sum_as_series(&self) -> Series {
        // apply_method_all_arrow_series!(self, sum_as_series,)
        unimplemented!()
    }
    /// Get the max of the Series as a new Series of length 1.
    pub fn max_as_series(&self) -> Series {
        // apply_method_all_arrow_series!(self, max_as_series,)
        unimplemented!()
    }
    /// Get the min of the Series as a new Series of length 1.
    pub fn min_as_series(&self) -> Series {
        // apply_method_all_arrow_series!(self, min_as_series,)
        unimplemented!()
    }
    /// Get the mean of the Series as a new Series of length 1.
    pub fn mean_as_series(&self) -> Series {
        // apply_method_all_arrow_series!(self, mean_as_series,)
        unimplemented!()
    }
    /// Get the median of the Series as a new Series of length 1.
    pub fn median_as_series(&self) -> Series {
        // apply_method_all_arrow_series!(self, median_as_series,)
        unimplemented!()
    }
    /// Get the variance of the Series as a new Series of length 1.
    pub fn var_as_series(&self) -> Series {
        // apply_method_all_arrow_series!(self, var_as_series,)
        unimplemented!()
    }
    /// Get the standard deviation of the Series as a new Series of length 1.
    pub fn std_as_series(&self) -> Series {
        // apply_method_all_arrow_series!(self, std_as_series,)
        unimplemented!()
    }
    /// Get the quantile of the ChunkedArray as a new Series of length 1.
    pub fn quantile_as_series(&self, quantile: f64) -> Result<Series> {
        // apply_method_all_arrow_series!(self, quantile_as_series, quantile)
        unimplemented!()
    }
    /// Apply a rolling mean to a Series. See:
    /// [ChunkedArray::rolling_mean](crate::prelude::ChunkWindow::rolling_mean).
    pub fn rolling_mean(
        &self,
        window_size: usize,
        weight: Option<&[f64]>,
        ignore_null: bool,
    ) -> Result<Self> {
        // let s = apply_method_all_arrow_series_and_return!(self, rolling_mean, [window_size, weight, ignore_null], ?);
        // Ok(s)
        unimplemented!()
    }
    /// Apply a rolling sum to a Series. See:
    /// [ChunkedArray::rolling_mean](crate::prelude::ChunkWindow::rolling_sum).
    pub fn rolling_sum(
        &self,
        window_size: usize,
        weight: Option<&[f64]>,
        ignore_null: bool,
    ) -> Result<Self> {
        // let s = apply_method_all_arrow_series_and_return!(self, rolling_sum, [window_size, weight, ignore_null], ?);
        // Ok(s)
        unimplemented!()
    }
    /// Apply a rolling min to a Series. See:
    /// [ChunkedArray::rolling_mean](crate::prelude::ChunkWindow::rolling_min).
    pub fn rolling_min(
        &self,
        window_size: usize,
        weight: Option<&[f64]>,
        ignore_null: bool,
    ) -> Result<Self> {
        // let s = apply_method_all_arrow_series_and_return!(self, rolling_min, [window_size, weight, ignore_null], ?);
        // Ok(s)

        unimplemented!()
    }
    /// Apply a rolling max to a Series. See:
    /// [ChunkedArray::rolling_mean](crate::prelude::ChunkWindow::rolling_max).
    pub fn rolling_max(
        &self,
        window_size: usize,
        weight: Option<&[f64]>,
        ignore_null: bool,
    ) -> Result<Self> {
        // let s = apply_method_all_arrow_series_and_return!(self, rolling_max, [window_size, weight, ignore_null], ?);
        // Ok(s)
        unimplemented!()
    }

    pub(crate) fn fmt_list(&self) -> String {
        apply_method_all_series!(self, fmt_list,)
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract hour from underlying NaiveDateTime representation.
    /// Returns the hour number from 0 to 23.
    pub fn hour(&self) -> Result<Self> {
        if let Series::Date64(ca) = self {
            Ok(Series::UInt32(ca.hour()))
        } else {
            Err(PolarsError::InvalidOperation(
                format!("operation not supported on dtype {:?}", self.dtype()).into(),
            ))
        }
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract minute from underlying NaiveDateTime representation.
    /// Returns the minute number from 0 to 59.
    pub fn minute(&self) -> Result<Self> {
        if let Series::Date64(ca) = self {
            Ok(Series::UInt32(ca.minute()))
        } else {
            Err(PolarsError::InvalidOperation(
                format!("operation not supported on dtype {:?}", self.dtype()).into(),
            ))
        }
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract second from underlying NaiveDateTime representation.
    /// Returns the second number from 0 to 59.
    pub fn second(&self) -> Result<Self> {
        if let Series::Date64(ca) = self {
            Ok(Series::UInt32(ca.second()))
        } else {
            Err(PolarsError::InvalidOperation(
                format!("operation not supported on dtype {:?}", self.dtype()).into(),
            ))
        }
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract second from underlying NaiveDateTime representation.
    /// Returns the number of nanoseconds since the whole non-leap second.
    /// The range from 1,000,000,000 to 1,999,999,999 represents the leap second.
    pub fn nanosecond(&self) -> Result<Self> {
        if let Series::Date64(ca) = self {
            Ok(Series::UInt32(ca.nanosecond()))
        } else {
            Err(PolarsError::InvalidOperation(
                format!("operation not supported on dtype {:?}", self.dtype()).into(),
            ))
        }
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract day from underlying NaiveDateTime representation.
    /// Returns the day of month starting from 1.
    ///
    /// The return value ranges from 1 to 31. (The last day of month differs by months.)
    pub fn day(&self) -> Result<Self> {
        match self {
            Series::Date32(ca) => Ok(Series::UInt32(ca.day())),
            Series::Date64(ca) => Ok(Series::UInt32(ca.day())),
            _ => Err(PolarsError::InvalidOperation(
                format!("operation not supported on dtype {:?}", self.dtype()).into(),
            )),
        }
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Returns the day of year starting from 1.
    ///
    /// The return value ranges from 1 to 366. (The last day of year differs by years.)
    pub fn ordinal_day(&self) -> Result<Self> {
        match self {
            Series::Date32(ca) => Ok(Series::UInt32(ca.ordinal())),
            Series::Date64(ca) => Ok(Series::UInt32(ca.ordinal())),
            _ => Err(PolarsError::InvalidOperation(
                format!("operation not supported on dtype {:?}", self.dtype()).into(),
            )),
        }
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract month from underlying NaiveDateTime representation.
    /// Returns the month number starting from 1.
    ///
    /// The return value ranges from 1 to 12.
    pub fn month(&self) -> Result<Self> {
        match self {
            Series::Date32(ca) => Ok(Series::UInt32(ca.month())),
            Series::Date64(ca) => Ok(Series::UInt32(ca.month())),
            _ => Err(PolarsError::InvalidOperation(
                format!("operation not supported on dtype {:?}", self.dtype()).into(),
            )),
        }
    }

    #[cfg(feature = "temporal")]
    #[doc(cfg(feature = "temporal"))]
    /// Extract month from underlying NaiveDateTime representation.
    /// Returns the year number in the calendar date.
    pub fn year(&self) -> Result<Self> {
        match self {
            Series::Date32(ca) => Ok(Series::Int32(ca.year())),
            Series::Date64(ca) => Ok(Series::Int32(ca.year())),
            _ => Err(PolarsError::InvalidOperation(
                format!("operation not supported on dtype {:?}", self.dtype()).into(),
            )),
        }
    }
}

fn pack_ca_to_series<N: PolarsDataType>(ca: ChunkedArray<N>) -> Series {
    unsafe {
        match N::get_data_type() {
            ArrowDataType::Boolean => Series::Bool(mem::transmute(ca)),
            ArrowDataType::Utf8 => Series::Utf8(mem::transmute(ca)),
            ArrowDataType::UInt8 => Series::UInt8(mem::transmute(ca)),
            ArrowDataType::UInt16 => Series::UInt16(mem::transmute(ca)),
            ArrowDataType::UInt32 => Series::UInt32(mem::transmute(ca)),
            ArrowDataType::UInt64 => Series::UInt64(mem::transmute(ca)),
            ArrowDataType::Int8 => Series::Int8(mem::transmute(ca)),
            ArrowDataType::Int16 => Series::Int16(mem::transmute(ca)),
            ArrowDataType::Int32 => Series::Int32(mem::transmute(ca)),
            ArrowDataType::Int64 => Series::Int64(mem::transmute(ca)),
            ArrowDataType::Float32 => Series::Float32(mem::transmute(ca)),
            ArrowDataType::Float64 => Series::Float64(mem::transmute(ca)),
            ArrowDataType::Date32(DateUnit::Day) => Series::Date32(mem::transmute(ca)),
            ArrowDataType::Date64(DateUnit::Millisecond) => Series::Date64(mem::transmute(ca)),
            ArrowDataType::Time64(datatypes::TimeUnit::Nanosecond) => {
                Series::Time64Nanosecond(mem::transmute(ca))
            }
            ArrowDataType::Duration(datatypes::TimeUnit::Nanosecond) => {
                Series::DurationNanosecond(mem::transmute(ca))
            }
            ArrowDataType::Duration(datatypes::TimeUnit::Millisecond) => {
                Series::DurationMillisecond(mem::transmute(ca))
            }
            #[cfg(feature = "dtype-interval")]
            ArrowDataType::Interval(IntervalUnit::YearMonth) => {
                Series::IntervalYearMonth(mem::transmute(ca))
            }
            #[cfg(feature = "dtype-interval")]
            ArrowDataType::Interval(IntervalUnit::DayTime) => {
                Series::IntervalDayTime(mem::transmute(ca))
            }
            ArrowDataType::List(_) => Series::List(mem::transmute(ca)),
            _ => panic!(
                "Not implemented or feature flag toggled off: {:?}",
                N::get_data_type()
            ),
        }
    }
}

pub trait NamedFrom<T, Phantom: ?Sized> {
    /// Initialize by name and values.
    fn new(name: &str, _: T) -> Self;
}

macro_rules! impl_named_from {
    ($type:ty, $series_var:ident, $method:ident) => {
        impl<T: AsRef<$type>> NamedFrom<T, $type> for Series {
            fn new(name: &str, v: T) -> Self {
                Series::$series_var(ChunkedArray::$method(name, v.as_ref()))
            }
        }
    };
}

impl<'a, T: AsRef<[&'a str]>> NamedFrom<T, [&'a str]> for Series {
    fn new(name: &str, v: T) -> Self {
        Series::Utf8(ChunkedArray::new_from_slice(name, v.as_ref()))
    }
}
impl<'a, T: AsRef<[Option<&'a str>]>> NamedFrom<T, [Option<&'a str>]> for Series {
    fn new(name: &str, v: T) -> Self {
        Series::Utf8(ChunkedArray::new_from_opt_slice(name, v.as_ref()))
    }
}

impl_named_from!([String], Utf8, new_from_slice);
impl_named_from!([bool], Bool, new_from_slice);
impl_named_from!([u8], UInt8, new_from_slice);
impl_named_from!([u16], UInt16, new_from_slice);
impl_named_from!([u32], UInt32, new_from_slice);
impl_named_from!([u64], UInt64, new_from_slice);
impl_named_from!([i8], Int8, new_from_slice);
impl_named_from!([i16], Int16, new_from_slice);
impl_named_from!([i32], Int32, new_from_slice);
impl_named_from!([i64], Int64, new_from_slice);
impl_named_from!([f32], Float32, new_from_slice);
impl_named_from!([f64], Float64, new_from_slice);
impl_named_from!([Option<String>], Utf8, new_from_opt_slice);
impl_named_from!([Option<bool>], Bool, new_from_opt_slice);
impl_named_from!([Option<u8>], UInt8, new_from_opt_slice);
impl_named_from!([Option<u16>], UInt16, new_from_opt_slice);
impl_named_from!([Option<u32>], UInt32, new_from_opt_slice);
impl_named_from!([Option<u64>], UInt64, new_from_opt_slice);
impl_named_from!([Option<i8>], Int8, new_from_opt_slice);
impl_named_from!([Option<i16>], Int16, new_from_opt_slice);
impl_named_from!([Option<i32>], Int32, new_from_opt_slice);
impl_named_from!([Option<i64>], Int64, new_from_opt_slice);
impl_named_from!([Option<f32>], Float32, new_from_opt_slice);
impl_named_from!([Option<f64>], Float64, new_from_opt_slice);

impl<T: AsRef<[Series]>> NamedFrom<T, ListType> for Series {
    fn new(name: &str, s: T) -> Self {
        let series_slice = s.as_ref();
        let dt = series_slice[0].dtype();
        let mut builder = get_list_builder(dt, series_slice.len(), name);
        for series in series_slice {
            builder.append_series(series)
        }
        builder.finish().into_series()
    }
}

macro_rules! impl_as_ref_ca {
    ($type:ident, $series_var:ident) => {
        impl AsRef<ChunkedArray<datatypes::$type>> for Series {
            fn as_ref(&self) -> &ChunkedArray<datatypes::$type> {
                match self {
                    Series::$series_var(a) => a,
                    _ => unimplemented!(),
                }
            }
        }
    };
}

impl_as_ref_ca!(UInt8Type, UInt8);
impl_as_ref_ca!(UInt16Type, UInt16);
impl_as_ref_ca!(UInt32Type, UInt32);
impl_as_ref_ca!(UInt64Type, UInt64);
impl_as_ref_ca!(Int8Type, Int8);
impl_as_ref_ca!(Int16Type, Int16);
impl_as_ref_ca!(Int32Type, Int32);
impl_as_ref_ca!(Int64Type, Int64);
impl_as_ref_ca!(Float32Type, Float32);
impl_as_ref_ca!(Float64Type, Float64);
impl_as_ref_ca!(BooleanType, Bool);
impl_as_ref_ca!(Utf8Type, Utf8);
impl_as_ref_ca!(Date32Type, Date32);
impl_as_ref_ca!(Date64Type, Date64);
impl_as_ref_ca!(Time64NanosecondType, Time64Nanosecond);
impl_as_ref_ca!(DurationNanosecondType, DurationNanosecond);
impl_as_ref_ca!(DurationMillisecondType, DurationMillisecond);
#[cfg(feature = "dtype-interval")]
impl_as_ref_ca!(IntervalDayTimeType, IntervalDayTime);
#[cfg(feature = "dtype-interval")]
impl_as_ref_ca!(IntervalYearMonthType, IntervalYearMonth);
impl_as_ref_ca!(ListType, List);

impl AsRef<Box<dyn SeriesOps>> for Series {
    fn as_ref(&self) -> &Box<dyn SeriesOps> {
        match self {
            Series::Object(a) => a,
            _ => unimplemented!(),
        }
    }
}

macro_rules! impl_as_mut_ca {
    ($type:ident, $series_var:ident) => {
        impl AsMut<ChunkedArray<datatypes::$type>> for Series {
            fn as_mut(&mut self) -> &mut ChunkedArray<datatypes::$type> {
                match self {
                    Series::$series_var(a) => a,
                    _ => unimplemented!(),
                }
            }
        }
    };
}

impl_as_mut_ca!(UInt8Type, UInt8);
impl_as_mut_ca!(UInt16Type, UInt16);
impl_as_mut_ca!(UInt32Type, UInt32);
impl_as_mut_ca!(UInt64Type, UInt64);
impl_as_mut_ca!(Int8Type, Int8);
impl_as_mut_ca!(Int16Type, Int16);
impl_as_mut_ca!(Int32Type, Int32);
impl_as_mut_ca!(Int64Type, Int64);
impl_as_mut_ca!(Float32Type, Float32);
impl_as_mut_ca!(Float64Type, Float64);
impl_as_mut_ca!(BooleanType, Bool);
impl_as_mut_ca!(Utf8Type, Utf8);
impl_as_mut_ca!(Date32Type, Date32);
impl_as_mut_ca!(Date64Type, Date64);
impl_as_mut_ca!(Time64NanosecondType, Time64Nanosecond);
impl_as_mut_ca!(DurationNanosecondType, DurationNanosecond);
impl_as_mut_ca!(DurationMillisecondType, DurationMillisecond);
#[cfg(feature = "dtype-interval")]
impl_as_mut_ca!(IntervalDayTimeType, IntervalDayTime);
#[cfg(feature = "dtype-interval")]
impl_as_mut_ca!(IntervalYearMonthType, IntervalYearMonth);
impl_as_mut_ca!(ListType, List);

macro_rules! from_series_to_ca {
    ($variant:ident, $ca:ident) => {
        impl<'a> From<&'a Series> for &'a $ca {
            fn from(s: &'a Series) -> Self {
                match s {
                    Series::$variant(ca) => ca,
                    _ => unimplemented!(),
                }
            }
        }
    };
}
from_series_to_ca!(UInt8, UInt8Chunked);
from_series_to_ca!(UInt16, UInt16Chunked);
from_series_to_ca!(UInt32, UInt32Chunked);
from_series_to_ca!(UInt64, UInt64Chunked);
from_series_to_ca!(Int8, Int8Chunked);
from_series_to_ca!(Int16, Int16Chunked);
from_series_to_ca!(Int32, Int32Chunked);
from_series_to_ca!(Int64, Int64Chunked);
from_series_to_ca!(Float32, Float32Chunked);
from_series_to_ca!(Float64, Float64Chunked);
from_series_to_ca!(Bool, BooleanChunked);
from_series_to_ca!(Utf8, Utf8Chunked);
from_series_to_ca!(Date32, Date32Chunked);
from_series_to_ca!(Date64, Date64Chunked);
from_series_to_ca!(Time64Nanosecond, Time64NanosecondChunked);
from_series_to_ca!(DurationMillisecond, DurationMillisecondChunked);
from_series_to_ca!(DurationNanosecond, DurationNanosecondChunked);
#[cfg(feature = "dtype-interval")]
from_series_to_ca!(IntervalDayTime, IntervalDayTimeChunked);
#[cfg(feature = "dtype-interval")]
from_series_to_ca!(IntervalYearMonth, IntervalYearMonthChunked);
from_series_to_ca!(List, ListChunked);

// TODO: add types
impl From<(&str, ArrayRef)> for Wrap<Arc<dyn SeriesTrait>> {
    fn from(name_arr: (&str, ArrayRef)) -> Self {
        let (name, arr) = name_arr;
        let chunk = vec![arr];
        match chunk[0].data_type() {
            ArrowDataType::Utf8 => Utf8Chunked::new_from_chunks(name, chunk).into_series(),
            ArrowDataType::Boolean => BooleanChunked::new_from_chunks(name, chunk).into_series(),
            ArrowDataType::UInt8 => UInt8Chunked::new_from_chunks(name, chunk).into_series(),
            ArrowDataType::UInt16 => UInt16Chunked::new_from_chunks(name, chunk).into_series(),
            ArrowDataType::UInt32 => UInt32Chunked::new_from_chunks(name, chunk).into_series(),
            ArrowDataType::UInt64 => UInt64Chunked::new_from_chunks(name, chunk).into_series(),
            ArrowDataType::Int8 => Int8Chunked::new_from_chunks(name, chunk).into_series(),
            ArrowDataType::Int16 => Int16Chunked::new_from_chunks(name, chunk).into_series(),
            ArrowDataType::Int32 => Int32Chunked::new_from_chunks(name, chunk).into_series(),
            ArrowDataType::Int64 => Int64Chunked::new_from_chunks(name, chunk).into_series(),
            ArrowDataType::Float32 => Float32Chunked::new_from_chunks(name, chunk).into_series(),
            ArrowDataType::Float64 => Float64Chunked::new_from_chunks(name, chunk).into_series(),
            ArrowDataType::Date32(DateUnit::Day) => {
                Date32Chunked::new_from_chunks(name, chunk).into_series()
            }
            ArrowDataType::Date64(DateUnit::Millisecond) => {
                Date64Chunked::new_from_chunks(name, chunk).into_series()
            }
            ArrowDataType::Time64(TimeUnit::Nanosecond) => {
                Time64NanosecondChunked::new_from_chunks(name, chunk).into_series()
            }
            #[cfg(feature = "dtype-interval")]
            ArrowDataType::Interval(IntervalUnit::DayTime) => {
                IntervalDayTimeChunked::new_from_chunks(name, chunk).into_series()
            }
            #[cfg(feature = "dtype-interval")]
            ArrowDataType::Interval(IntervalUnit::YearMonth) => {
                IntervalYearMonthChunked::new_from_chunks(name, chunk).into_series()
            }
            ArrowDataType::Duration(TimeUnit::Nanosecond) => {
                DurationNanosecondChunked::new_from_chunks(name, chunk).into_series()
            }
            ArrowDataType::Duration(TimeUnit::Millisecond) => {
                DurationMillisecondChunked::new_from_chunks(name, chunk).into_series()
            }
            ArrowDataType::List(_) => ListChunked::new_from_chunks(name, chunk).into_series(),
            _ => unimplemented!(),
        }
    }
}

impl Default for Series {
    fn default() -> Self {
        Series::Int8(ChunkedArray::default())
    }
}

impl<T> From<ChunkedArray<T>> for Series
where
    T: PolarsDataType,
{
    fn from(ca: ChunkedArray<T>) -> Self {
        Series::from_chunked_array(ca)
    }
}

impl From<Box<dyn SeriesOps>> for Series {
    fn from(ca: Box<dyn SeriesOps>) -> Self {
        Series::Object(ca)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn cast() {
        let ar = ChunkedArray::<Int32Type>::new_from_slice("a", &[1, 2]);
        let s = Series::Int32(ar);
        let s2 = s.cast::<Int64Type>().unwrap();
        match s2 {
            Series::Int64(_) => assert!(true),
            _ => assert!(false),
        }
        let s2 = s.cast::<Float32Type>().unwrap();
        match s2 {
            Series::Float32(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn new_series() {
        Series::new("boolean series", &vec![true, false, true]);
        Series::new("int series", &[1, 2, 3]);
        let ca = Int32Chunked::new_from_slice("a", &[1, 2, 3]);
        ca.into_series();
    }

    #[test]
    fn series_append() {
        let mut s1 = Series::new("a", &[1, 2]);
        let s2 = Series::new("b", &[3]);
        s1.append(&s2).unwrap();
        assert_eq!(s1.len(), 3);

        // add wrong type
        let s2 = Series::new("b", &[3.0]);
        assert!(s1.append(&s2).is_err())
    }
}
