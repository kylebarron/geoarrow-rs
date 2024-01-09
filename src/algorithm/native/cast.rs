//! Note: In the future, it may be possible to optimize some of these casts, e.g. from Point to
//! MultiPoint by only constructing a new offsets array, but you have to check that the coordinate
//! type is not casted!
//!
//! todo: have a set of "fast cast" functions, where you first try to fast cast and fall back to
//! slower copies if necessary. Can check that the coord type of the input and output is the same.

use std::sync::Arc;

use arrow_array::OffsetSizeTrait;

use crate::array::util::OffsetBufferUtils;
use crate::array::*;
use crate::chunked_array::{ChunkedGeometryArray, ChunkedGeometryArrayTrait};
use crate::datatypes::GeoDataType;
use crate::error::{GeoArrowError, Result};
use crate::geo_traits::{MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait};
use crate::GeometryArrayTrait;

/// CastOptions provides a way to override the default cast behaviors
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CastOptions {
    /// how to handle cast failures, either return NULL (safe=true) or return ERR (safe=false)
    pub safe: bool,
}

impl Default for CastOptions {
    fn default() -> Self {
        Self { safe: true }
    }
}

/// This cast only covers
#[allow(dead_code)]
pub fn can_cast_types(from_type: &GeoDataType, to_type: &GeoDataType) -> bool {
    if from_type == to_type {
        return true;
    }

    use GeoDataType::*;
    match (from_type, to_type) {
        (Point(_), Point(_) | MultiPoint(_)) => true,
        (LineString(_), LineString(_) | MultiLineString(_)) => true,
        (Polygon(_), Polygon(_) | MultiPolygon(_)) => true,
        (MultiPoint(_), MultiPoint(_)) => true,
        (MultiLineString(_), MultiLineString(_)) => true,
        (MultiPolygon(_), MultiPolygon(_)) => true,
        _ => todo!(),
    }
}

pub trait Cast {
    type Output;

    fn cast(&self, to_type: &GeoDataType) -> Self::Output;
}

fn cast_point_array(
    array: &PointArray,
    to_type: &GeoDataType,
) -> Result<Arc<dyn GeometryArrayTrait>> {
    use GeoDataType::*;
    match to_type {
        Point(ct) => {
            let mut builder = PointBuilder::with_capacity_and_options(array.buffer_lengths(), *ct);
            array.iter().for_each(|x| builder.push_point(x.as_ref()));
            Ok(Arc::new(builder.finish()))
        }
        MultiPoint(ct) => {
            let capacity = MultiPointCapacity::new(array.buffer_lengths(), array.buffer_lengths());
            let mut builder = MultiPointBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_point(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMultiPoint(ct) => {
            let capacity = MultiPointCapacity::new(array.buffer_lengths(), array.buffer_lengths());
            let mut builder = MultiPointBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_point(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        Mixed(ct) => {
            let capacity = MixedCapacity {
                point: array.buffer_lengths(),
                ..Default::default()
            };
            let mut builder = MixedGeometryBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array.iter().for_each(|x| builder.push_point(x.as_ref()));
            Ok(Arc::new(builder.finish()))
        }
        LargeMixed(ct) => {
            let capacity = MixedCapacity {
                point: array.buffer_lengths(),
                ..Default::default()
            };
            let mut builder = MixedGeometryBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array.iter().for_each(|x| builder.push_point(x.as_ref()));
            Ok(Arc::new(builder.finish()))
        }
        GeometryCollection(ct) => {
            let mixed_capacity = MixedCapacity {
                point: array.buffer_lengths(),
                ..Default::default()
            };
            let capacity = GeometryCollectionCapacity::new(mixed_capacity, array.buffer_lengths());
            let mut builder =
                GeometryCollectionBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_point(x.as_ref(), false))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeGeometryCollection(ct) => {
            let mixed_capacity = MixedCapacity {
                point: array.buffer_lengths(),
                ..Default::default()
            };
            let capacity = GeometryCollectionCapacity::new(mixed_capacity, array.buffer_lengths());
            let mut builder =
                GeometryCollectionBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_point(x.as_ref(), false))?;
            Ok(Arc::new(builder.finish()))
        }

        _ => Err(GeoArrowError::General("invalid cast".to_string())),
    }
}

fn cast_line_string_array<O: OffsetSizeTrait>(
    array: &LineStringArray<O>,
    to_type: &GeoDataType,
) -> Result<Arc<dyn GeometryArrayTrait>> {
    use GeoDataType::*;
    match to_type {
        LineString(ct) => {
            let mut builder =
                LineStringBuilder::<i32>::with_capacity_and_options(array.buffer_lengths(), *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_line_string(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeLineString(ct) => {
            let mut builder =
                LineStringBuilder::<i64>::with_capacity_and_options(array.buffer_lengths(), *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_line_string(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        MultiLineString(ct) => {
            let mut capacity = MultiLineStringCapacity::new_empty();
            capacity += array.buffer_lengths();
            let mut builder =
                MultiLineStringBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_line_string(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMultiLineString(ct) => {
            let mut capacity = MultiLineStringCapacity::new_empty();
            capacity += array.buffer_lengths();
            let mut builder =
                MultiLineStringBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_line_string(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        Mixed(ct) => {
            let capacity = MixedCapacity {
                line_string: array.buffer_lengths(),
                ..Default::default()
            };
            let mut builder = MixedGeometryBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_line_string(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMixed(ct) => {
            let capacity = MixedCapacity {
                line_string: array.buffer_lengths(),
                ..Default::default()
            };
            let mut builder = MixedGeometryBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_line_string(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        GeometryCollection(ct) => {
            let mixed_capacity = MixedCapacity {
                line_string: array.buffer_lengths(),
                ..Default::default()
            };
            let capacity = GeometryCollectionCapacity::new(mixed_capacity, array.len());
            let mut builder =
                GeometryCollectionBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_line_string(x.as_ref(), false))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeGeometryCollection(ct) => {
            let mixed_capacity = MixedCapacity {
                line_string: array.buffer_lengths(),
                ..Default::default()
            };
            let capacity = GeometryCollectionCapacity::new(mixed_capacity, array.len());
            let mut builder =
                GeometryCollectionBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_line_string(x.as_ref(), false))?;
            Ok(Arc::new(builder.finish()))
        }

        _ => Err(GeoArrowError::General("invalid cast".to_string())),
    }
}

fn cast_polygon_array<O: OffsetSizeTrait>(
    array: &PolygonArray<O>,
    to_type: &GeoDataType,
) -> Result<Arc<dyn GeometryArrayTrait>> {
    use GeoDataType::*;
    match to_type {
        Polygon(ct) => {
            let mut builder =
                PolygonBuilder::<i32>::with_capacity_and_options(array.buffer_lengths(), *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_polygon(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargePolygon(ct) => {
            let mut builder =
                PolygonBuilder::<i64>::with_capacity_and_options(array.buffer_lengths(), *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_polygon(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        MultiPolygon(ct) => {
            let mut capacity = MultiPolygonCapacity::new_empty();
            capacity += array.buffer_lengths();
            let mut builder = MultiPolygonBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_polygon(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMultiPolygon(ct) => {
            let mut capacity = MultiPolygonCapacity::new_empty();
            capacity += array.buffer_lengths();
            let mut builder = MultiPolygonBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_polygon(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        Mixed(ct) => {
            let capacity = MixedCapacity {
                polygon: array.buffer_lengths(),
                ..Default::default()
            };
            let mut builder = MixedGeometryBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_polygon(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMixed(ct) => {
            let capacity = MixedCapacity {
                polygon: array.buffer_lengths(),
                ..Default::default()
            };
            let mut builder = MixedGeometryBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_polygon(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        GeometryCollection(ct) => {
            let mixed_capacity = MixedCapacity {
                polygon: array.buffer_lengths(),
                ..Default::default()
            };
            let capacity = GeometryCollectionCapacity::new(mixed_capacity, array.len());
            let mut builder =
                GeometryCollectionBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_polygon(x.as_ref(), false))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeGeometryCollection(ct) => {
            let mixed_capacity = MixedCapacity {
                polygon: array.buffer_lengths(),
                ..Default::default()
            };
            let capacity = GeometryCollectionCapacity::new(mixed_capacity, array.len());
            let mut builder =
                GeometryCollectionBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_polygon(x.as_ref(), false))?;
            Ok(Arc::new(builder.finish()))
        }

        _ => Err(GeoArrowError::General("invalid cast".to_string())),
    }
}

fn cast_multi_point_array<O: OffsetSizeTrait>(
    array: &MultiPointArray<O>,
    to_type: &GeoDataType,
) -> Result<Arc<dyn GeometryArrayTrait>> {
    use GeoDataType::*;
    match to_type {
        Point(ct) => {
            if array.geom_offsets.last().to_usize().unwrap() != array.len() {
                return Err(GeoArrowError::General("Unable to cast".to_string()));
            }

            let mut builder = PointBuilder::with_capacity_and_options(array.len(), *ct);
            array
                .iter()
                .for_each(|x| builder.push_point(x.map(|mp| mp.point(0).unwrap()).as_ref()));
            Ok(Arc::new(builder.finish()))
        }
        MultiPoint(ct) => {
            let capacity = array.buffer_lengths();
            let mut builder = MultiPointBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_point(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMultiPoint(ct) => {
            let capacity = array.buffer_lengths();
            let mut builder = MultiPointBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_point(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        Mixed(ct) => {
            let capacity = MixedCapacity {
                multi_point: array.buffer_lengths(),
                ..Default::default()
            };
            let mut builder = MixedGeometryBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_point(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMixed(ct) => {
            let capacity = MixedCapacity {
                multi_point: array.buffer_lengths(),
                ..Default::default()
            };
            let mut builder = MixedGeometryBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_point(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        GeometryCollection(ct) => {
            let mixed_capacity = MixedCapacity {
                multi_point: array.buffer_lengths(),
                ..Default::default()
            };
            let capacity = GeometryCollectionCapacity::new(mixed_capacity, array.len());
            let mut builder =
                GeometryCollectionBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_point(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeGeometryCollection(ct) => {
            let mixed_capacity = MixedCapacity {
                multi_point: array.buffer_lengths(),
                ..Default::default()
            };
            let capacity = GeometryCollectionCapacity::new(mixed_capacity, array.len());
            let mut builder =
                GeometryCollectionBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_point(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }

        _ => Err(GeoArrowError::General("invalid cast".to_string())),
    }
}

fn cast_multi_line_string_array<O: OffsetSizeTrait>(
    array: &MultiLineStringArray<O>,
    to_type: &GeoDataType,
) -> Result<Arc<dyn GeometryArrayTrait>> {
    use GeoDataType::*;
    match to_type {
        LineString(ct) => {
            if array.geom_offsets.last().to_usize().unwrap() != array.len() {
                return Err(GeoArrowError::General("Unable to cast".to_string()));
            }

            let existing_capacity = array.buffer_lengths();
            let capacity = LineStringCapacity {
                coord_capacity: existing_capacity.coord_capacity,
                geom_capacity: existing_capacity.ring_capacity,
            };
            let mut builder = LineStringBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array.iter().try_for_each(|x| {
                builder.push_line_string(x.map(|mp| mp.line(0).unwrap()).as_ref())
            })?;
            Ok(Arc::new(builder.finish()))
        }
        LargeLineString(ct) => {
            if array.geom_offsets.last().to_usize().unwrap() != array.len() {
                return Err(GeoArrowError::General("Unable to cast".to_string()));
            }

            let existing_capacity = array.buffer_lengths();
            let capacity = LineStringCapacity {
                coord_capacity: existing_capacity.coord_capacity,
                geom_capacity: existing_capacity.ring_capacity,
            };
            let mut builder = LineStringBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array.iter().try_for_each(|x| {
                builder.push_line_string(x.map(|mp| mp.line(0).unwrap()).as_ref())
            })?;
            Ok(Arc::new(builder.finish()))
        }
        Mixed(ct) => {
            let capacity = MixedCapacity {
                multi_line_string: array.buffer_lengths(),
                ..Default::default()
            };
            let mut builder = MixedGeometryBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_line_string(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMixed(ct) => {
            let capacity = MixedCapacity {
                multi_line_string: array.buffer_lengths(),
                ..Default::default()
            };
            let mut builder = MixedGeometryBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_line_string(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        GeometryCollection(ct) => {
            let mixed_capacity = MixedCapacity {
                multi_line_string: array.buffer_lengths(),
                ..Default::default()
            };
            let capacity = GeometryCollectionCapacity::new(mixed_capacity, array.len());
            let mut builder =
                GeometryCollectionBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_line_string(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeGeometryCollection(ct) => {
            let mixed_capacity = MixedCapacity {
                multi_line_string: array.buffer_lengths(),
                ..Default::default()
            };
            let capacity = GeometryCollectionCapacity::new(mixed_capacity, array.len());
            let mut builder =
                GeometryCollectionBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_line_string(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }

        _ => Err(GeoArrowError::General("invalid cast".to_string())),
    }
}

fn cast_multi_polygon_array<O: OffsetSizeTrait>(
    array: &MultiPolygonArray<O>,
    to_type: &GeoDataType,
) -> Result<Arc<dyn GeometryArrayTrait>> {
    use GeoDataType::*;
    match to_type {
        Polygon(ct) => {
            if array.geom_offsets.last().to_usize().unwrap() != array.len() {
                return Err(GeoArrowError::General("Unable to cast".to_string()));
            }

            let existing_capacity = array.buffer_lengths();
            let capacity = PolygonCapacity {
                coord_capacity: existing_capacity.coord_capacity,
                ring_capacity: existing_capacity.ring_capacity,
                geom_capacity: existing_capacity.polygon_capacity,
            };
            let mut builder = PolygonBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array.iter().try_for_each(|x| {
                builder.push_polygon(x.map(|mp| mp.polygon(0).unwrap()).as_ref())
            })?;
            Ok(Arc::new(builder.finish()))
        }
        LargePolygon(ct) => {
            if array.geom_offsets.last().to_usize().unwrap() != array.len() {
                return Err(GeoArrowError::General("Unable to cast".to_string()));
            }

            let existing_capacity = array.buffer_lengths();
            let capacity = PolygonCapacity {
                coord_capacity: existing_capacity.coord_capacity,
                ring_capacity: existing_capacity.ring_capacity,
                geom_capacity: existing_capacity.polygon_capacity,
            };
            let mut builder = PolygonBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array.iter().try_for_each(|x| {
                builder.push_polygon(x.map(|mp| mp.polygon(0).unwrap()).as_ref())
            })?;
            Ok(Arc::new(builder.finish()))
        }
        Mixed(ct) => {
            let capacity = MixedCapacity {
                multi_polygon: array.buffer_lengths(),
                ..Default::default()
            };
            let mut builder = MixedGeometryBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_polygon(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMixed(ct) => {
            let capacity = MixedCapacity {
                multi_polygon: array.buffer_lengths(),
                ..Default::default()
            };
            let mut builder = MixedGeometryBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_polygon(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        GeometryCollection(ct) => {
            let mixed_capacity = MixedCapacity {
                multi_polygon: array.buffer_lengths(),
                ..Default::default()
            };
            let capacity = GeometryCollectionCapacity::new(mixed_capacity, array.len());
            let mut builder =
                GeometryCollectionBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_polygon(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeGeometryCollection(ct) => {
            let mixed_capacity = MixedCapacity {
                multi_polygon: array.buffer_lengths(),
                ..Default::default()
            };
            let capacity = GeometryCollectionCapacity::new(mixed_capacity, array.len());
            let mut builder =
                GeometryCollectionBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_multi_polygon(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }

        _ => Err(GeoArrowError::General("invalid cast".to_string())),
    }
}

/// TODO: in the future, do more validation before trying to fill all geometries
fn cast_mixed_array<O: OffsetSizeTrait>(
    array: &MixedGeometryArray<O>,
    to_type: &GeoDataType,
) -> Result<Arc<dyn GeometryArrayTrait>> {
    use GeoDataType::*;
    match to_type {
        Point(ct) => {
            if array.has_line_strings()
                | array.has_polygons()
                | array.has_multi_line_strings()
                | array.has_multi_polygons()
            {
                return Err(GeoArrowError::General("".to_string()));
            }

            let mut builder = PointBuilder::with_capacity_and_options(array.len(), *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LineString(ct) => {
            if array.has_points()
                | array.has_polygons()
                | array.has_multi_points()
                | array.has_multi_polygons()
            {
                return Err(GeoArrowError::General("".to_string()));
            }

            let mut capacity = array
                .line_strings
                .as_ref()
                .map(|x| x.buffer_lengths())
                .unwrap_or_default();
            if let Some(multi_line_strings) = &array.multi_line_strings {
                if multi_line_strings.geom_offsets.last().to_usize().unwrap()
                    != multi_line_strings.len()
                {
                    return Err(GeoArrowError::General("Unable to cast".to_string()));
                }
                let buffer_lengths = multi_line_strings.buffer_lengths();
                capacity.coord_capacity += buffer_lengths.coord_capacity;
                capacity.geom_capacity += buffer_lengths.ring_capacity;
            }

            let mut builder = LineStringBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeLineString(ct) => {
            if array.has_points()
                | array.has_polygons()
                | array.has_multi_points()
                | array.has_multi_polygons()
            {
                return Err(GeoArrowError::General("".to_string()));
            }

            let mut capacity = array
                .line_strings
                .as_ref()
                .map(|x| x.buffer_lengths())
                .unwrap_or_default();
            if let Some(multi_line_strings) = &array.multi_line_strings {
                if multi_line_strings.geom_offsets.last().to_usize().unwrap()
                    != multi_line_strings.len()
                {
                    return Err(GeoArrowError::General("Unable to cast".to_string()));
                }
                let buffer_lengths = multi_line_strings.buffer_lengths();
                capacity.coord_capacity += buffer_lengths.coord_capacity;
                capacity.geom_capacity += buffer_lengths.ring_capacity;
            }

            let mut builder = LineStringBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        Polygon(ct) => {
            if array.has_points()
                | array.has_line_strings()
                | array.has_multi_points()
                | array.has_multi_line_strings()
            {
                return Err(GeoArrowError::General("".to_string()));
            }

            let mut capacity = array
                .polygons
                .as_ref()
                .map(|x| x.buffer_lengths())
                .unwrap_or_default();
            if let Some(multi_polygons) = &array.multi_polygons {
                if multi_polygons.geom_offsets.last().to_usize().unwrap() != multi_polygons.len() {
                    return Err(GeoArrowError::General("Unable to cast".to_string()));
                }
                let buffer_lengths = multi_polygons.buffer_lengths();
                capacity.coord_capacity += buffer_lengths.coord_capacity;
                capacity.ring_capacity += buffer_lengths.ring_capacity;
                capacity.geom_capacity += buffer_lengths.polygon_capacity;
            }

            let mut builder = PolygonBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargePolygon(ct) => {
            if array.has_points()
                | array.has_line_strings()
                | array.has_multi_points()
                | array.has_multi_line_strings()
            {
                return Err(GeoArrowError::General("".to_string()));
            }

            let mut capacity = array
                .polygons
                .as_ref()
                .map(|x| x.buffer_lengths())
                .unwrap_or_default();
            if let Some(multi_polygons) = &array.multi_polygons {
                if multi_polygons.geom_offsets.last().to_usize().unwrap() != multi_polygons.len() {
                    return Err(GeoArrowError::General("Unable to cast".to_string()));
                }
                let buffer_lengths = multi_polygons.buffer_lengths();
                capacity.coord_capacity += buffer_lengths.coord_capacity;
                capacity.ring_capacity += buffer_lengths.ring_capacity;
                capacity.geom_capacity += buffer_lengths.polygon_capacity;
            }

            let mut builder = PolygonBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        MultiPoint(ct) => {
            if array.has_line_strings()
                | array.has_polygons()
                | array.has_multi_line_strings()
                | array.has_multi_polygons()
            {
                return Err(GeoArrowError::General("".to_string()));
            }

            let mut capacity = array
                .multi_points
                .as_ref()
                .map(|x| x.buffer_lengths())
                .unwrap_or_default();
            if let Some(points) = &array.points {
                // Hack: move to newtype
                capacity.coord_capacity += points.buffer_lengths();
                capacity.geom_capacity += points.buffer_lengths();
            }

            let mut builder = MultiPointBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMultiPoint(ct) => {
            if array.has_line_strings()
                | array.has_polygons()
                | array.has_multi_line_strings()
                | array.has_multi_polygons()
            {
                return Err(GeoArrowError::General("".to_string()));
            }

            let mut capacity = array
                .multi_points
                .as_ref()
                .map(|x| x.buffer_lengths())
                .unwrap_or_default();
            if let Some(points) = &array.points {
                // Hack: move to newtype
                capacity.coord_capacity += points.buffer_lengths();
                capacity.geom_capacity += points.buffer_lengths();
            }

            let mut builder = MultiPointBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        MultiLineString(ct) => {
            if array.has_points()
                | array.has_polygons()
                | array.has_multi_points()
                | array.has_multi_polygons()
            {
                return Err(GeoArrowError::General("".to_string()));
            }

            let mut capacity = array
                .multi_line_strings
                .as_ref()
                .map(|x| x.buffer_lengths())
                .unwrap_or_default();
            if let Some(line_strings) = &array.line_strings {
                capacity += line_strings.buffer_lengths();
            }

            let mut builder =
                MultiLineStringBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMultiLineString(ct) => {
            if array.has_points()
                | array.has_polygons()
                | array.has_multi_points()
                | array.has_multi_polygons()
            {
                return Err(GeoArrowError::General("".to_string()));
            }

            let mut capacity = array
                .multi_line_strings
                .as_ref()
                .map(|x| x.buffer_lengths())
                .unwrap_or_default();
            if let Some(line_strings) = &array.line_strings {
                capacity += line_strings.buffer_lengths();
            }

            let mut builder =
                MultiLineStringBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        MultiPolygon(ct) => {
            if array.has_points()
                | array.has_line_strings()
                | array.has_multi_points()
                | array.has_multi_line_strings()
            {
                return Err(GeoArrowError::General("".to_string()));
            }

            let mut capacity = array
                .multi_polygons
                .as_ref()
                .map(|x| x.buffer_lengths())
                .unwrap_or_default();
            if let Some(polygons) = &array.polygons {
                capacity += polygons.buffer_lengths();
            }

            let mut builder = MultiPolygonBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMultiPolygon(ct) => {
            if array.has_points()
                | array.has_line_strings()
                | array.has_multi_points()
                | array.has_multi_line_strings()
            {
                return Err(GeoArrowError::General("".to_string()));
            }

            let mut capacity = array
                .multi_polygons
                .as_ref()
                .map(|x| x.buffer_lengths())
                .unwrap_or_default();
            if let Some(polygons) = &array.polygons {
                capacity += polygons.buffer_lengths();
            }

            let mut builder = MultiPolygonBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        Mixed(ct) => {
            let capacity = array.buffer_lengths();
            let mut builder = MixedGeometryBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeMixed(ct) => {
            let capacity = array.buffer_lengths();
            let mut builder = MixedGeometryBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref()))?;
            Ok(Arc::new(builder.finish()))
        }
        GeometryCollection(ct) => {
            let capacity = GeometryCollectionCapacity::new(array.buffer_lengths(), array.len());
            let mut builder =
                GeometryCollectionBuilder::<i32>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref(), false))?;
            Ok(Arc::new(builder.finish()))
        }
        LargeGeometryCollection(ct) => {
            let capacity = GeometryCollectionCapacity::new(array.buffer_lengths(), array.len());
            let mut builder =
                GeometryCollectionBuilder::<i64>::with_capacity_and_options(capacity, *ct);
            array
                .iter()
                .try_for_each(|x| builder.push_geometry(x.as_ref(), false))?;
            Ok(Arc::new(builder.finish()))
        }

        _ => Err(GeoArrowError::General("invalid cast".to_string())),
    }
}

impl Cast for &dyn GeometryArrayTrait {
    type Output = Result<Arc<dyn GeometryArrayTrait>>;

    fn cast(&self, to_type: &GeoDataType) -> Self::Output {
        // TODO: not working :/
        // if self.data_type() == to_type {
        //     return Ok(Arc::new(self.to_owned()));
        // }

        use GeoDataType::*;
        match self.data_type() {
            Point(_) => cast_point_array(self.as_ref().as_point(), to_type),
            LineString(_) => cast_line_string_array(self.as_ref().as_line_string(), to_type),
            LargeLineString(_) => {
                cast_line_string_array(self.as_ref().as_large_line_string(), to_type)
            }
            Polygon(_) => cast_polygon_array(self.as_ref().as_polygon(), to_type),
            LargePolygon(_) => cast_polygon_array(self.as_ref().as_large_polygon(), to_type),
            MultiPoint(_) => cast_multi_point_array(self.as_ref().as_multi_point(), to_type),
            LargeMultiPoint(_) => {
                cast_multi_point_array(self.as_ref().as_large_multi_point(), to_type)
            }
            MultiLineString(_) => {
                cast_multi_line_string_array(self.as_ref().as_multi_line_string(), to_type)
            }
            LargeMultiLineString(_) => {
                cast_multi_line_string_array(self.as_ref().as_large_multi_line_string(), to_type)
            }
            MultiPolygon(_) => cast_multi_polygon_array(self.as_ref().as_multi_polygon(), to_type),
            LargeMultiPolygon(_) => {
                cast_multi_polygon_array(self.as_ref().as_large_multi_polygon(), to_type)
            }
            Mixed(_) => cast_mixed_array(self.as_ref().as_mixed(), to_type),
            LargeMixed(_) => cast_mixed_array(self.as_ref().as_large_mixed(), to_type),

            _ => todo!(),
        }
    }
}

impl Cast for &dyn ChunkedGeometryArrayTrait {
    type Output = Result<Arc<dyn ChunkedGeometryArrayTrait>>;

    fn cast(&self, to_type: &GeoDataType) -> Self::Output {
        macro_rules! impl_cast {
            ($method:ident) => {
                Arc::new(ChunkedGeometryArray::new(
                    self.geometry_chunks()
                        .iter()
                        .map(|chunk| Ok(chunk.as_ref().cast(to_type)?.as_ref().$method().clone()))
                        .collect::<Result<Vec<_>>>()?,
                ))
            };
        }

        use GeoDataType::*;
        let result: Arc<dyn ChunkedGeometryArrayTrait> = match to_type {
            Point(_) => impl_cast!(as_point),
            LineString(_) => impl_cast!(as_line_string),
            LargeLineString(_) => impl_cast!(as_large_line_string),
            Polygon(_) => impl_cast!(as_polygon),
            LargePolygon(_) => impl_cast!(as_large_polygon),
            MultiPoint(_) => impl_cast!(as_multi_point),
            LargeMultiPoint(_) => impl_cast!(as_large_multi_point),
            MultiLineString(_) => impl_cast!(as_multi_line_string),
            LargeMultiLineString(_) => impl_cast!(as_large_multi_line_string),
            MultiPolygon(_) => impl_cast!(as_polygon),
            LargeMultiPolygon(_) => impl_cast!(as_large_polygon),
            Mixed(_) => impl_cast!(as_mixed),
            LargeMixed(_) => impl_cast!(as_large_mixed),
            GeometryCollection(_) => impl_cast!(as_geometry_collection),
            LargeGeometryCollection(_) => impl_cast!(as_large_geometry_collection),
            WKB => impl_cast!(as_wkb),
            LargeWKB => impl_cast!(as_large_wkb),
            Rect => impl_cast!(as_rect),
        };
        Ok(result)
    }
}
