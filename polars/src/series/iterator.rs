use crate::prelude::*;
use crate::series::implementations::Wrap;
use crate::utils::Xob;
use std::iter::FromIterator;

macro_rules! from_iterator {
    ($native:ty, $variant:ident) => {
        impl FromIterator<Option<$native>> for Series {
            fn from_iter<I: IntoIterator<Item = Option<$native>>>(iter: I) -> Self {
                let ca = iter.into_iter().collect();
                Series::$variant(ca)
            }
        }

        impl FromIterator<$native> for Series {
            fn from_iter<I: IntoIterator<Item = $native>>(iter: I) -> Self {
                let ca: Xob<ChunkedArray<_>> = iter.into_iter().collect();
                Series::$variant(ca.into_inner())
            }
        }

        impl<'a> FromIterator<&'a $native> for Series {
            fn from_iter<I: IntoIterator<Item = &'a $native>>(iter: I) -> Self {
                let ca = iter.into_iter().map(|v| Some(*v)).collect();
                Series::$variant(ca)
            }
        }
    };
}

from_iterator!(u8, UInt8);
from_iterator!(u16, UInt16);
from_iterator!(u32, UInt32);
from_iterator!(u64, UInt64);
from_iterator!(i8, Int8);
from_iterator!(i16, Int16);
from_iterator!(i32, Int32);
from_iterator!(i64, Int64);
from_iterator!(f32, Float32);
from_iterator!(f64, Float64);
from_iterator!(bool, Bool);

impl<'a> FromIterator<&'a str> for Wrap<Arc<dyn SeriesTrait>> {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let ca: Utf8Chunked = iter.into_iter().collect();
        Wrap(ca.into_series())
    }
}

impl<'a> FromIterator<&'a dyn SeriesTrait> for Wrap<Arc<dyn SeriesTrait>> {
    fn from_iter<I: IntoIterator<Item = &'a SeriesTrait>>(iter: I) -> Self {
        let ca: ListChunked = iter.into_iter().collect();
        Wrap(ca.into_series())
    }
}

impl<'a> FromIterator<Option<&'a dyn SeriesTrait>> for Wrap<Arc<dyn SeriesTrait>> {
    fn from_iter<I: IntoIterator<Item = Option<&'a dyn SeriesTrait>>>(iter: I) -> Self {
        let ca: ListChunked = iter.into_iter().collect();
        Wrap(ca.into_series())
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_iter() {
        let a = Series::new("age", [23, 71, 9].as_ref());
        let _b = a
            .i32()
            .unwrap()
            .into_iter()
            .map(|opt_v| opt_v.map(|v| v * 2));
    }
}
