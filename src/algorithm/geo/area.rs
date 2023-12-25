use crate::algorithm::geo::utils::zeroes;
use crate::array::*;
use crate::chunked_array::chunked_array::{ChunkedArray, ChunkedGeometryArray};
use crate::datatypes::GeoDataType;
use crate::GeometryArrayTrait;
use arrow_array::builder::Float64Builder;
use arrow_array::{Float64Array, OffsetSizeTrait};
use geo::prelude::Area as GeoArea;

/// Signed and unsigned planar area of a geometry.
///
/// # Examples
///
/// ```
/// use geo::polygon;
///
/// use geoarrow::algorithm::geo::Area;
/// use geoarrow::array::PolygonArray;
///
/// let polygon = polygon![
///     (x: 0., y: 0.),
///     (x: 5., y: 0.),
///     (x: 5., y: 6.),
///     (x: 0., y: 6.),
///     (x: 0., y: 0.),
/// ];
///
/// let mut reversed_polygon = polygon.clone();
/// reversed_polygon.exterior_mut(|line_string| {
///     line_string.0.reverse();
/// });
///
/// let polygon_array: PolygonArray<i32> = vec![polygon].as_slice().into();
/// let reversed_polygon_array: PolygonArray<i32> = vec![reversed_polygon].as_slice().into();
///
/// assert_eq!(polygon_array.signed_area().value(0), 30.);
/// assert_eq!(polygon_array.unsigned_area().value(0), 30.);
///
/// assert_eq!(reversed_polygon_array.signed_area().value(0), -30.);
/// assert_eq!(reversed_polygon_array.unsigned_area().value(0), 30.);
/// ```
pub trait Area {
    type Output;

    fn signed_area(&self) -> Self::Output;

    fn unsigned_area(&self) -> Self::Output;
}

// Note: this can't (easily) be parameterized in the macro because PointArray is not generic over O
impl Area for PointArray {
    type Output = Float64Array;

    fn signed_area(&self) -> Self::Output {
        zeroes(self.len(), self.nulls())
    }

    fn unsigned_area(&self) -> Self::Output {
        zeroes(self.len(), self.nulls())
    }
}

/// Implementation where the result is zero.
macro_rules! zero_impl {
    ($type:ty) => {
        impl<O: OffsetSizeTrait> Area for $type {
            type Output = Float64Array;

            fn signed_area(&self) -> Self::Output {
                zeroes(self.len(), self.nulls())
            }

            fn unsigned_area(&self) -> Self::Output {
                zeroes(self.len(), self.nulls())
            }
        }
    };
}

zero_impl!(LineStringArray<O>);
zero_impl!(MultiPointArray<O>);
zero_impl!(MultiLineStringArray<O>);

macro_rules! iter_geo_impl {
    ($type:ty) => {
        impl<O: OffsetSizeTrait> Area for $type {
            type Output = Float64Array;

            fn signed_area(&self) -> Self::Output {
                let mut output_array = Float64Builder::with_capacity(self.len());
                self.iter_geo().for_each(|maybe_g| {
                    output_array.append_option(maybe_g.map(|g| g.signed_area()))
                });
                output_array.finish()
            }

            fn unsigned_area(&self) -> Self::Output {
                let mut output_array = Float64Builder::with_capacity(self.len());
                self.iter_geo().for_each(|maybe_g| {
                    output_array.append_option(maybe_g.map(|g| g.unsigned_area()))
                });
                output_array.finish()
            }
        }
    };
}

iter_geo_impl!(PolygonArray<O>);
iter_geo_impl!(MultiPolygonArray<O>);
iter_geo_impl!(MixedGeometryArray<O>);
iter_geo_impl!(GeometryCollectionArray<O>);
iter_geo_impl!(WKBArray<O>);

impl<O: OffsetSizeTrait> Area for GeometryArray<O> {
    type Output = Float64Array;

    crate::geometry_array_delegate_impl! {
        fn signed_area(&self) -> Float64Array;

        fn unsigned_area(&self) -> Float64Array;
    }
}

impl Area for &dyn GeometryArrayTrait {
    type Output = Float64Array;

    fn signed_area(&self) -> Self::Output {
        match self.data_type() {
            GeoDataType::Point(_) => self.as_point().signed_area(),
            GeoDataType::LineString(_) => self.as_line_string().signed_area(),
            GeoDataType::LargeLineString(_) => self.as_large_line_string().signed_area(),
            GeoDataType::Polygon(_) => self.as_polygon().signed_area(),
            GeoDataType::LargePolygon(_) => self.as_large_polygon().signed_area(),
            GeoDataType::MultiPoint(_) => self.as_multi_point().signed_area(),
            GeoDataType::LargeMultiPoint(_) => self.as_large_multi_point().signed_area(),
            GeoDataType::MultiLineString(_) => self.as_multi_line_string().signed_area(),
            GeoDataType::LargeMultiLineString(_) => self.as_large_multi_line_string().signed_area(),
            GeoDataType::MultiPolygon(_) => self.as_multi_polygon().signed_area(),
            GeoDataType::LargeMultiPolygon(_) => self.as_large_multi_polygon().signed_area(),
            GeoDataType::Mixed(_) => self.as_mixed().signed_area(),
            GeoDataType::LargeMixed(_) => self.as_large_mixed().signed_area(),
            GeoDataType::GeometryCollection(_) => self.as_geometry_collection().signed_area(),
            GeoDataType::LargeGeometryCollection(_) => {
                self.as_large_geometry_collection().signed_area()
            }
            _ => panic!("incorrect type"),
        }
    }

    fn unsigned_area(&self) -> Self::Output {
        match self.data_type() {
            GeoDataType::Point(_) => self.as_point().unsigned_area(),
            GeoDataType::LineString(_) => self.as_line_string().unsigned_area(),
            GeoDataType::LargeLineString(_) => self.as_large_line_string().unsigned_area(),
            GeoDataType::Polygon(_) => self.as_polygon().unsigned_area(),
            GeoDataType::LargePolygon(_) => self.as_large_polygon().unsigned_area(),
            GeoDataType::MultiPoint(_) => self.as_multi_point().unsigned_area(),
            GeoDataType::LargeMultiPoint(_) => self.as_large_multi_point().unsigned_area(),
            GeoDataType::MultiLineString(_) => self.as_multi_line_string().unsigned_area(),
            GeoDataType::LargeMultiLineString(_) => {
                self.as_large_multi_line_string().unsigned_area()
            }
            GeoDataType::MultiPolygon(_) => self.as_multi_polygon().unsigned_area(),
            GeoDataType::LargeMultiPolygon(_) => self.as_large_multi_polygon().unsigned_area(),
            GeoDataType::Mixed(_) => self.as_mixed().unsigned_area(),
            GeoDataType::LargeMixed(_) => self.as_large_mixed().unsigned_area(),
            GeoDataType::GeometryCollection(_) => self.as_geometry_collection().unsigned_area(),
            GeoDataType::LargeGeometryCollection(_) => {
                self.as_large_geometry_collection().unsigned_area()
            }
            _ => panic!("incorrect type"),
        }
    }
}

impl<G: GeometryArrayTrait> Area for ChunkedGeometryArray<G> {
    type Output = ChunkedArray<Float64Array>;

    fn signed_area(&self) -> Self::Output {
        ChunkedArray::new(
            self.chunks
                .iter()
                .map(|c| c.as_ref().signed_area())
                .collect(),
        )
    }

    fn unsigned_area(&self) -> Self::Output {
        ChunkedArray::new(
            self.chunks
                .iter()
                .map(|c| c.as_ref().unsigned_area())
                .collect(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::polygon::p_array;

    #[test]
    fn area() {
        let arr = p_array();
        let area = arr.unsigned_area();
        assert_eq!(area, Float64Array::new(vec![28., 18.].into(), None));
    }
}
