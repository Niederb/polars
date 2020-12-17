use crate::prelude::*;
use crate::series::SeriesTrait;
use crate::utils::get_supertype;
use num::{Num, NumCast, ToPrimitive};
use std::borrow::Cow;
use std::fmt::Debug;
use std::ops;

pub trait NumOpsDispatch: Debug {
    fn subtract(&self, rhs: &dyn SeriesTrait) -> Result<Arc<dyn SeriesTrait>> {
        Err(PolarsError::InvalidOperation(
            format!(
                "subtraction operation not supported for {:?} and {:?}",
                self, rhs
            )
            .into(),
        ))
    }
    fn add_to(&self, rhs: &dyn SeriesTrait) -> Result<Arc<dyn SeriesTrait>> {
        Err(PolarsError::InvalidOperation(
            format!(
                "addition operation not supported for {:?} and {:?}",
                self, rhs
            )
            .into(),
        ))
    }
    fn multiply(&self, rhs: &dyn SeriesTrait) -> Result<Arc<dyn SeriesTrait>> {
        Err(PolarsError::InvalidOperation(
            format!(
                "multiplication operation not supported for {:?} and {:?}",
                self, rhs
            )
            .into(),
        ))
    }
    fn divide(&self, rhs: &dyn SeriesTrait) -> Result<Arc<dyn SeriesTrait>> {
        Err(PolarsError::InvalidOperation(
            format!(
                "division operation not supported for {:?} and {:?}",
                self, rhs
            )
            .into(),
        ))
    }
    fn remainder(&self, rhs: &dyn SeriesTrait) -> Result<Arc<dyn SeriesTrait>> {
        Err(PolarsError::InvalidOperation(
            format!(
                "remainder operation not supported for {:?} and {:?}",
                self, rhs
            )
            .into(),
        ))
    }
}

impl<T> NumOpsDispatch for ChunkedArray<T>
where
    T: PolarsNumericType,
    T::Native: ops::Add<Output = T::Native>
        + ops::Sub<Output = T::Native>
        + ops::Mul<Output = T::Native>
        + ops::Div<Output = T::Native>
        + ops::Rem<Output = T::Native>
        + num::Zero
        + num::One,
    ChunkedArray<T>: IntoSeries,
{
    fn subtract(&self, rhs: &dyn SeriesTrait) -> Result<Arc<dyn SeriesTrait>> {
        let rhs = self.unpack_series_matching_type(rhs)?;
        let out = self - rhs;
        Ok(out.into_series())
    }
    fn add_to(&self, rhs: &dyn SeriesTrait) -> Result<Arc<dyn SeriesTrait>> {
        let rhs = self.unpack_series_matching_type(rhs)?;
        let out = self + rhs;
        Ok(out.into_series())
    }
    fn multiply(&self, rhs: &dyn SeriesTrait) -> Result<Arc<dyn SeriesTrait>> {
        let rhs = self.unpack_series_matching_type(rhs)?;
        let out = self * rhs;
        Ok(out.into_series())
    }
    fn divide(&self, rhs: &dyn SeriesTrait) -> Result<Arc<dyn SeriesTrait>> {
        let rhs = self.unpack_series_matching_type(rhs)?;
        let out = self / rhs;
        Ok(out.into_series())
    }
    fn remainder(&self, rhs: &dyn SeriesTrait) -> Result<Arc<dyn SeriesTrait>> {
        let rhs = self.unpack_series_matching_type(rhs)?;
        let out = self % rhs;
        Ok(out.into_series())
    }
}

impl NumOpsDispatch for Utf8Chunked {
    fn add_to(&self, rhs: &dyn SeriesTrait) -> Result<Arc<dyn SeriesTrait>> {
        let rhs = self.unpack_series_matching_type(rhs)?;
        let out = self + rhs;
        Ok(out.into_series())
    }
}
impl NumOpsDispatch for BooleanChunked {}
impl NumOpsDispatch for ListChunked {}

pub(crate) fn coerce_lhs_rhs<'a>(
    lhs: &'a dyn SeriesTrait,
    rhs: &'a dyn SeriesTrait,
) -> Result<(Arc<dyn SeriesTrait>, Arc<dyn SeriesTrait>)> {
    let dtype = get_supertype(lhs.dtype(), rhs.dtype())?;
    let left = if lhs.dtype() == &dtype {
        lhs.clone()
    } else {
        lhs.cast_with_arrow_datatype(&dtype)?
    };
    let right = if rhs.dtype() == &dtype {
        rhs.clone()
    } else {
        rhs.cast_with_arrow_datatype(&dtype)?
    };
    Ok((left, right))
}

impl<'a> ops::Sub for &(dyn SeriesTrait + 'a) {
    type Output = Arc<dyn SeriesTrait>;

    fn sub(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = coerce_lhs_rhs(self, rhs).expect("cannot coerce datatypes");
        lhs.subtract(rhs.as_ref()).expect("data types don't match")
    }
}

impl<'a> ops::Add for &(dyn SeriesTrait + 'a) {
    type Output = Arc<dyn SeriesTrait>;

    fn add(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = coerce_lhs_rhs(self, rhs).expect("cannot coerce datatypes");
        lhs.add_to(rhs.as_ref()).expect("data types don't match")
    }
}

impl<'a> std::ops::Mul for &(dyn SeriesTrait + 'a) {
    type Output = Arc<dyn SeriesTrait>;

    /// ```
    /// # use polars::prelude::*;
    /// let s: Series = [1, 2, 3].iter().collect();
    /// let out = &s * &s;
    /// ```
    fn mul(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = coerce_lhs_rhs(self, rhs).expect("cannot coerce datatypes");
        lhs.multiply(rhs.as_ref()).expect("data types don't match")
    }
}

impl<'a> std::ops::Div for &(dyn SeriesTrait + 'a) {
    type Output = Arc<dyn SeriesTrait>;

    /// ```
    /// # use polars::prelude::*;
    /// let s: Series = [1, 2, 3].iter().collect();
    /// let out = &s / &s;
    /// ```
    fn div(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = coerce_lhs_rhs(self, rhs).expect("cannot coerce datatypes");
        lhs.divide(rhs.as_ref()).expect("data types don't match")
    }
}

impl<'a> std::ops::Rem for &(dyn SeriesTrait + 'a) {
    type Output = Arc<dyn SeriesTrait>;

    /// ```
    /// # use polars::prelude::*;
    /// let s: Series = [1, 2, 3].iter().collect();
    /// let out = &s / &s;
    /// ```
    fn rem(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = coerce_lhs_rhs(self, rhs).expect("cannot coerce datatypes");
        lhs.remainder(rhs.as_ref()).expect("data types don't match")
    }
}

// Series +-/* numbers instead of Series

pub(super) trait NumOpsDispatchSeriesSingleNumber {
    fn subtract_number<N: Num + NumCast>(&self, _rhs: N) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }
    fn add_number<N: Num + NumCast>(&self, _rhs: N) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }
    fn multiply_number<N: Num + NumCast>(&self, _rhs: N) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }
    fn divide_number<N: Num + NumCast>(&self, _rhs: N) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }
}

impl NumOpsDispatchSeriesSingleNumber for BooleanChunked {}
impl NumOpsDispatchSeriesSingleNumber for Utf8Chunked {}
impl NumOpsDispatchSeriesSingleNumber for ListChunked {}
impl<T> NumOpsDispatchSeriesSingleNumber for ObjectChunked<T> {}

impl<T> NumOpsDispatchSeriesSingleNumber for ChunkedArray<T>
where
    T: PolarsNumericType,
    T::Native: Num
        + NumCast
        + ops::Add<Output = T::Native>
        + ops::Sub<Output = T::Native>
        + ops::Mul<Output = T::Native>
        + ops::Div<Output = T::Native>,
    ChunkedArray<T>: IntoSeries,
{
    fn subtract_number<N: Num + NumCast>(&self, rhs: N) -> Arc<dyn SeriesTrait> {
        let rhs: T::Native =
            NumCast::from(rhs).unwrap_or_else(|| panic!("could not cast".to_string()));
        let mut ca: ChunkedArray<T> = self
            .into_iter()
            .map(|opt_v| opt_v.map(|v| v - rhs))
            .collect();
        ca.rename(self.name());
        ca.into_series()
    }

    fn add_number<N: Num + NumCast>(&self, rhs: N) -> Arc<dyn SeriesTrait> {
        let rhs: T::Native =
            NumCast::from(rhs).unwrap_or_else(|| panic!("could not cast".to_string()));
        let mut ca: ChunkedArray<T> = self
            .into_iter()
            .map(|opt_v| opt_v.map(|v| v + rhs))
            .collect();
        ca.rename(self.name());
        ca.into_series()
    }
    fn multiply_number<N: Num + NumCast>(&self, rhs: N) -> Arc<dyn SeriesTrait> {
        let rhs: T::Native =
            NumCast::from(rhs).unwrap_or_else(|| panic!("could not cast".to_string()));
        let mut ca: ChunkedArray<T> = self
            .into_iter()
            .map(|opt_v| opt_v.map(|v| v * rhs))
            .collect();
        ca.rename(self.name());
        ca.into_series()
    }
    fn divide_number<N: Num + NumCast>(&self, rhs: N) -> Arc<dyn SeriesTrait> {
        let rhs: T::Native =
            NumCast::from(rhs).unwrap_or_else(|| panic!("could not cast".to_string()));
        let mut ca: ChunkedArray<T> = self
            .into_iter()
            .map(|opt_v| opt_v.map(|v| v / rhs))
            .collect();
        ca.rename(self.name());
        ca.into_series()
    }
}

impl<T> ops::Sub<T> for &dyn SeriesTrait
where
    T: Num + NumCast,
{
    type Output = Series;

    fn sub(self, rhs: T) -> Self::Output {
        // apply_method_all_arrow_series!(self, subtract_number, rhs)
        todo!()
    }
}

impl<T> ops::Add<T> for &dyn SeriesTrait
where
    T: Num + NumCast,
{
    type Output = Series;

    fn add(self, rhs: T) -> Self::Output {
        // apply_method_all_arrow_series!(self, add_number, rhs)
        todo!()
    }
}

impl<T> ops::Div<T> for &dyn SeriesTrait
where
    T: Num + NumCast,
{
    type Output = Series;

    fn div(self, rhs: T) -> Self::Output {
        // apply_method_all_arrow_series!(self, divide_number, rhs)
        todo!()
    }
}

impl<T> ops::Mul<T> for &dyn SeriesTrait
where
    T: Num + NumCast,
{
    type Output = Series;

    fn mul(self, rhs: T) -> Self::Output {
        // apply_method_all_arrow_series!(self, multiply_number, rhs)
        todo!()
    }
}

/// We cannot override the left hand side behaviour. So we create a trait Lhs num ops.
/// This allows for 1.add(&dyn SeriesTrait)

pub(super) trait LhsNumOpsDispatch {
    fn lhs_subtract_number<N: Num + NumCast>(&self, _lhs: N) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }
    fn lhs_add_number<N: Num + NumCast>(&self, _lhs: N) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }
    fn lhs_multiply_number<N: Num + NumCast>(&self, _lhs: N) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }
    fn lhs_divide_number<N: Num + NumCast>(&self, _lhs: N) -> Arc<dyn SeriesTrait> {
        unimplemented!()
    }
}

impl LhsNumOpsDispatch for BooleanChunked {}
impl LhsNumOpsDispatch for Utf8Chunked {}
impl LhsNumOpsDispatch for ListChunked {}
impl<T> LhsNumOpsDispatch for ObjectChunked<T> {}

impl<T> LhsNumOpsDispatch for ChunkedArray<T>
where
    T: PolarsNumericType,
    T::Native: Num
        + NumCast
        + ops::Add<Output = T::Native>
        + ops::Sub<Output = T::Native>
        + ops::Mul<Output = T::Native>
        + ops::Div<Output = T::Native>,
    ChunkedArray<T>: IntoSeries,
{
    fn lhs_subtract_number<N: Num + NumCast>(&self, lhs: N) -> Arc<dyn SeriesTrait> {
        let lhs: T::Native =
            NumCast::from(lhs).unwrap_or_else(|| panic!("could not cast".to_string()));
        let mut ca: ChunkedArray<T> = self
            .into_iter()
            .map(|opt_v| opt_v.map(|v| lhs - v))
            .collect();
        ca.rename(self.name());
        ca.into_series()
    }

    fn lhs_add_number<N: Num + NumCast>(&self, lhs: N) -> Arc<dyn SeriesTrait> {
        let lhs: T::Native =
            NumCast::from(lhs).unwrap_or_else(|| panic!("could not cast".to_string()));
        let mut ca: ChunkedArray<T> = self
            .into_iter()
            .map(|opt_v| opt_v.map(|v| lhs + v))
            .collect();
        ca.rename(self.name());
        ca.into_series()
    }
    fn lhs_multiply_number<N: Num + NumCast>(&self, lhs: N) -> Arc<dyn SeriesTrait> {
        let lhs: T::Native =
            NumCast::from(lhs).unwrap_or_else(|| panic!("could not cast".to_string()));
        let mut ca: ChunkedArray<T> = self
            .into_iter()
            .map(|opt_v| opt_v.map(|v| lhs * v))
            .collect();
        ca.rename(self.name());
        ca.into_series()
    }
    fn lhs_divide_number<N: Num + NumCast>(&self, lhs: N) -> Arc<dyn SeriesTrait> {
        let lhs: T::Native =
            NumCast::from(lhs).unwrap_or_else(|| panic!("could not cast".to_string()));
        let mut ca: ChunkedArray<T> = self
            .into_iter()
            .map(|opt_v| opt_v.map(|v| lhs / v))
            .collect();
        ca.rename(self.name());
        ca.into_series()
    }
}

pub trait LhsNumOps {
    type Output;

    fn add(self, rhs: &dyn SeriesTrait) -> Self::Output;
    fn sub(self, rhs: &dyn SeriesTrait) -> Self::Output;
    fn div(self, rhs: &dyn SeriesTrait) -> Self::Output;
    fn mul(self, rhs: &dyn SeriesTrait) -> Self::Output;
}

impl<T> LhsNumOps for T
where
    T: Num + NumCast,
{
    type Output = Arc<dyn SeriesTrait>;

    fn add(self, rhs: &dyn SeriesTrait) -> Self::Output {
        // apply_method_all_arrow_series!(rhs, lhs_add_number, self)
        todo!()
    }
    fn sub(self, rhs: &dyn SeriesTrait) -> Self::Output {
        // apply_method_all_arrow_series!(rhs, lhs_subtract_number, self)
        todo!()
    }
    fn div(self, rhs: &dyn SeriesTrait) -> Self::Output {
        // apply_method_all_arrow_series!(rhs, lhs_divide_number, self)
        todo!()
    }
    fn mul(self, rhs: &dyn SeriesTrait) -> Self::Output {
        // apply_method_all_arrow_series!(rhs, lhs_multiply_number, self)
        todo!()
    }
}

impl Series {
    fn pow<E: Num>(&self, exp: E) -> Series
    where
        E: ToPrimitive,
    {
        match self {
            Series::UInt8(ca) => Series::Float32(ca.pow_f32(exp.to_f32().unwrap())),
            Series::UInt16(ca) => Series::Float32(ca.pow_f32(exp.to_f32().unwrap())),
            Series::UInt32(ca) => Series::Float32(ca.pow_f32(exp.to_f32().unwrap())),
            Series::UInt64(ca) => Series::Float64(ca.pow_f64(exp.to_f64().unwrap())),
            Series::Int8(ca) => Series::Float32(ca.pow_f32(exp.to_f32().unwrap())),
            Series::Int16(ca) => Series::Float32(ca.pow_f32(exp.to_f32().unwrap())),
            Series::Int32(ca) => Series::Float32(ca.pow_f32(exp.to_f32().unwrap())),
            Series::Int64(ca) => Series::Float64(ca.pow_f64(exp.to_f64().unwrap())),
            Series::Float32(ca) => Series::Float32(ca.pow_f32(exp.to_f32().unwrap())),
            Series::Float64(ca) => Series::Float64(ca.pow_f64(exp.to_f64().unwrap())),
            Series::Date32(ca) => Series::Float32(ca.pow_f32(exp.to_f32().unwrap())),
            Series::Date64(ca) => Series::Float64(ca.pow_f64(exp.to_f64().unwrap())),
            Series::Time64Nanosecond(ca) => Series::Float64(ca.pow_f64(exp.to_f64().unwrap())),
            Series::DurationNanosecond(ca) => Series::Float64(ca.pow_f64(exp.to_f64().unwrap())),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_arithmetic_series() {
        // Series +-/* Series
        let s: Series = [1, 2, 3].iter().collect();
        assert_eq!(
            Vec::from((&s * &s).i32().unwrap()),
            [Some(1), Some(4), Some(9)]
        );
        assert_eq!(
            Vec::from((&s / &s).i32().unwrap()),
            [Some(1), Some(1), Some(1)]
        );
        assert_eq!(
            Vec::from((&s - &s).i32().unwrap()),
            [Some(0), Some(0), Some(0)]
        );
        assert_eq!(
            Vec::from((&s + &s).i32().unwrap()),
            [Some(2), Some(4), Some(6)]
        );
        // Series +-/* Number
        assert_eq!(
            Vec::from((&s + 1).i32().unwrap()),
            [Some(2), Some(3), Some(4)]
        );
        assert_eq!(
            Vec::from((&s - 1).i32().unwrap()),
            [Some(0), Some(1), Some(2)]
        );
        assert_eq!(
            Vec::from((&s * 2).i32().unwrap()),
            [Some(2), Some(4), Some(6)]
        );
        assert_eq!(
            Vec::from((&s / 2).i32().unwrap()),
            [Some(0), Some(1), Some(1)]
        );

        // Lhs operations
        assert_eq!(
            Vec::from((1.add(&s)).i32().unwrap()),
            [Some(2), Some(3), Some(4)]
        );
        assert_eq!(
            Vec::from((1.sub(&s)).i32().unwrap()),
            [Some(0), Some(-1), Some(-2)]
        );
        assert_eq!(
            Vec::from((1.div(&s)).i32().unwrap()),
            [Some(1), Some(0), Some(0)]
        );
        assert_eq!(
            Vec::from((1.mul(&s)).i32().unwrap()),
            [Some(1), Some(2), Some(3)]
        );
    }
}
