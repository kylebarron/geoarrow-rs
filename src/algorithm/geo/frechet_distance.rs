use crate::algorithm::native::{Binary, MapChunks, Unary};
use crate::array::*;
use crate::chunked_array::{ChunkedArray, ChunkedLineStringArray};
use crate::datatypes::GeoDataType;
use crate::error::{GeoArrowError, Result};
use crate::geo_traits::LineStringTrait;
use crate::io::geo::line_string_to_geo;
use crate::trait_::GeometryScalarTrait;
use crate::GeometryArrayTrait;
use arrow_array::{Float64Array, OffsetSizeTrait};
use geo::FrechetDistance as _FrechetDistance;

// ┌────────────────────────────────┐
// │ Implementations for RHS arrays │
// └────────────────────────────────┘

pub trait FrechetDistance<Rhs = Self> {
    type Output;

    fn frechet_distance(&self, rhs: &Rhs) -> Self::Output;
}

impl<O: OffsetSizeTrait> FrechetDistance for LineStringArray<O> {
    type Output = Float64Array;

    fn frechet_distance(&self, rhs: &Self) -> Self::Output {
        self.try_binary_primitive(rhs, |left, right| {
            Ok(left.to_geo().frechet_distance(&right.to_geo()))
        })
        .unwrap()
    }
}

impl<O: OffsetSizeTrait> FrechetDistance for ChunkedLineStringArray<O> {
    type Output = ChunkedArray<Float64Array>;

    fn frechet_distance(&self, rhs: &Self) -> Self::Output {
        ChunkedArray::new(self.binary_map(rhs.chunks(), |(left, right)| {
            FrechetDistance::frechet_distance(left, right)
        }))
    }
}

impl<O: OffsetSizeTrait> FrechetDistance for &dyn GeometryArrayTrait {
    type Output = Result<Float64Array>;

    fn frechet_distance(&self, rhs: &Self) -> Self::Output {
        let result = match (self.data_type(), rhs.data_type()) {
            GeoDataType::LineString(_) => self.as_line_string().frechet_distance(),
            GeoDataType::LargeLineString(_) => self.as_large_line_string().frechet_distance(),
            _ => return Err(GeoArrowError::IncorrectType("".into())),
        };
        Ok(result)
    }
}

// ┌─────────────────────────────────┐
// │ Implementations for RHS scalars │
// └─────────────────────────────────┘

pub trait FrechetDistanceLineString<Rhs> {
    type Output;

    fn frechet_distance(&self, rhs: &Rhs) -> Self::Output;
}

impl<O: OffsetSizeTrait, G: LineStringTrait<T = f64>> FrechetDistanceLineString<G>
    for LineStringArray<O>
{
    type Output = Float64Array;

    fn frechet_distance(&self, rhs: &G) -> Self::Output {
        let rhs = line_string_to_geo(rhs);
        self.try_unary_primitive(|geom| {
            Ok::<_, GeoArrowError>(geom.to_geo().frechet_distance(&rhs))
        })
        .unwrap()
    }
}

impl<O: OffsetSizeTrait, G: LineStringTrait<T = f64> + Sync> FrechetDistanceLineString<G>
    for ChunkedLineStringArray<O>
{
    type Output = ChunkedArray<Float64Array>;

    fn frechet_distance(&self, rhs: &G) -> Self::Output {
        ChunkedArray::new(self.map(|chunk| FrechetDistanceLineString::frechet_distance(chunk, rhs)))
    }
}
