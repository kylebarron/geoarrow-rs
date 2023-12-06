use crate::array::linestring::LineStringCapacity;
use crate::array::mixed::array::GeometryType;
use crate::array::multilinestring::MultiLineStringCapacity;
use crate::array::multipoint::MultiPointCapacity;
use crate::array::multipolygon::MultiPolygonCapacity;
use crate::array::polygon::PolygonCapacity;
use crate::array::{
    CoordType, LineStringBuilder, MixedGeometryArray, MultiLineStringBuilder, MultiPointBuilder,
    MultiPolygonBuilder, PointBuilder, PolygonBuilder, WKBArray,
};
use crate::error::{GeoArrowError, Result};
use crate::geo_traits::*;
use crate::io::wkb::reader::geometry::WKBGeometry;
use crate::scalar::WKB;
use crate::trait_::IntoArrow;
use crate::GeometryArrayTrait;
use arrow_array::{OffsetSizeTrait, UnionArray};

/// The Arrow equivalent to a `Vec<Option<Geometry>>` with the caveat that these geometries must be
/// a _primitive_ geometry type. That means this does not support Geometry::GeometryCollection.
///
/// # Invariants
///
/// - All arrays must have the same dimension
/// - All arrays must have the same coordinate layout (interleaved or separated)
#[derive(Debug)]
pub struct MixedGeometryBuilder<O: OffsetSizeTrait> {
    // Invariant: every item in `types` is `> 0 && < fields.len()`
    // - 0: PointArray
    // - 1: LineStringArray
    // - 2: PolygonArray
    // - 3: MultiPointArray
    // - 4: MultiLineStringArray
    // - 5: MultiPolygonArray
    types: Vec<i8>,

    /// Note that we include an ordering so that exporting this array to Arrow is O(1). If we used
    /// another ordering like always Point, LineString, etc. then we'd either have to always export
    /// all arrays (including some zero-length arrays) or have to reorder the `types` buffer when
    /// exporting.
    // ordering: Vec<>,
    points: PointBuilder,
    line_strings: LineStringBuilder<O>,
    polygons: PolygonBuilder<O>,
    multi_points: MultiPointBuilder<O>,
    multi_line_strings: MultiLineStringBuilder<O>,
    multi_polygons: MultiPolygonBuilder<O>,

    // The offset of the _next_ geometry to be pushed into these arrays
    // This is necessary to maintain so that we can efficiently update `offsets` below
    point_counter: i32,
    line_string_counter: i32,
    polygon_counter: i32,
    multi_point_counter: i32,
    multi_line_string_counter: i32,
    multi_polygon_counter: i32,

    // Invariant: `offsets.len() == types.len()`
    offsets: Vec<i32>,
}

impl<'a, O: OffsetSizeTrait> MixedGeometryBuilder<O> {
    /// Creates a new empty [`MixedGeometryBuilder`].
    pub fn new() -> Self {
        Self {
            types: vec![],
            points: PointBuilder::new(),
            line_strings: LineStringBuilder::new(),
            polygons: PolygonBuilder::new(),
            multi_points: MultiPointBuilder::new(),
            multi_line_strings: MultiLineStringBuilder::new(),
            multi_polygons: MultiPolygonBuilder::new(),
            point_counter: 0,
            line_string_counter: 0,
            polygon_counter: 0,
            multi_point_counter: 0,
            multi_line_string_counter: 0,
            multi_polygon_counter: 0,
            offsets: vec![],
        }
    }

    /// Creates a new [`MixedGeometryBuilder`] with given capacity and no validity.
    pub fn with_capacity(capacity: MixedCapacity) -> Self {
        Self::with_capacity_and_options(capacity, Default::default())
    }

    pub fn with_capacity_and_options(capacity: MixedCapacity, coord_type: CoordType) -> Self {
        Self {
            types: vec![],
            points: PointBuilder::with_capacity_and_options(capacity.point, coord_type),
            line_strings: LineStringBuilder::with_capacity_and_options(
                capacity.line_string,
                coord_type,
            ),
            polygons: PolygonBuilder::with_capacity_and_options(capacity.polygon, coord_type),
            multi_points: MultiPointBuilder::with_capacity_and_options(
                capacity.multi_point,
                coord_type,
            ),
            multi_line_strings: MultiLineStringBuilder::with_capacity_and_options(
                capacity.multi_line_string,
                coord_type,
            ),
            multi_polygons: MultiPolygonBuilder::with_capacity_and_options(
                capacity.multi_polygon,
                coord_type,
            ),
            point_counter: 0,
            line_string_counter: 0,
            polygon_counter: 0,
            multi_point_counter: 0,
            multi_line_string_counter: 0,
            multi_polygon_counter: 0,
            offsets: vec![],
        }
    }

    pub fn with_capacity_from_iter(
        geoms: impl Iterator<Item = Option<&'a (impl GeometryTrait + 'a)>>,
    ) -> Self {
        Self::with_capacity_and_options_from_iter(geoms, Default::default())
    }

    pub fn with_capacity_and_options_from_iter(
        geoms: impl Iterator<Item = Option<&'a (impl GeometryTrait + 'a)>>,
        coord_type: CoordType,
    ) -> Self {
        let counter = MixedCapacity::from_geometries(geoms);
        Self::with_capacity_and_options(counter, coord_type)
    }

    pub fn reserve(&mut self, capacity: MixedCapacity) {
        let total_num_geoms = capacity.total_num_geoms();
        self.types.reserve(total_num_geoms);
        self.offsets.reserve(total_num_geoms);
        self.points.reserve(capacity.point);
        self.line_strings.reserve(capacity.line_string);
        self.polygons.reserve(capacity.polygon);
        self.multi_points.reserve(capacity.multi_point);
        self.multi_line_strings.reserve(capacity.multi_line_string);
        self.multi_polygons.reserve(capacity.multi_polygon);
    }

    pub fn reserve_exact(&mut self, capacity: MixedCapacity) {
        let total_num_geoms = capacity.total_num_geoms();
        self.types.reserve_exact(total_num_geoms);
        self.offsets.reserve_exact(total_num_geoms);
        self.points.reserve_exact(capacity.point);
        self.line_strings.reserve_exact(capacity.line_string);
        self.polygons.reserve_exact(capacity.polygon);
        self.multi_points.reserve_exact(capacity.multi_point);
        self.multi_line_strings
            .reserve_exact(capacity.multi_line_string);
        self.multi_polygons.reserve_exact(capacity.multi_polygon);
    }

    pub fn reserve_from_iter(
        &mut self,
        geoms: impl Iterator<Item = Option<&'a (impl GeometryTrait + 'a)>>,
    ) {
        let counter = MixedCapacity::from_geometries(geoms);
        self.reserve(counter)
    }

    pub fn reserve_exact_from_iter(
        &mut self,
        geoms: impl Iterator<Item = Option<&'a (impl GeometryTrait + 'a)>>,
    ) {
        let counter = MixedCapacity::from_geometries(geoms);
        self.reserve_exact(counter)
    }

    // /// The canonical method to create a [`MixedGeometryBuilder`] out of its internal
    // /// components.
    // ///
    // /// # Implementation
    // ///
    // /// This function is `O(1)`.
    // ///
    // /// # Errors
    // ///
    // pub fn try_new(
    //     coords: CoordBufferBuilder,
    //     geom_offsets: BufferBuilder<O>,
    //     ring_offsets: BufferBuilder<O>,
    //     validity: Option<MutableBitmap>,
    // ) -> Result<Self> {
    //     check(
    //         &coords.clone().into(),
    //         &geom_offsets.clone().into(),
    //         &ring_offsets.clone().into(),
    //         validity.as_ref().map(|x| x.len()),
    //     )?;
    //     Ok(Self {
    //         coords,
    //         geom_offsets,
    //         ring_offsets,
    //         validity,
    //     })
    // }

    /// Add a new Point to the end of this array, storing it in the PointBuilder child array.
    #[inline]
    pub fn push_point(&mut self, value: Option<&impl PointTrait<T = f64>>) {
        self.offsets.push(self.point_counter);
        self.point_counter += 1;

        self.types.push(GeometryType::Point.default_ordering());
        self.points.push_point(value)
    }

    /// Add a new Point to the end of this array, storing it in the MultiPointBuilder child
    /// array.
    #[inline]
    pub fn push_point_as_multi_point(
        &mut self,
        value: Option<&impl PointTrait<T = f64>>,
    ) -> Result<()> {
        self.offsets.push(self.multi_point_counter);
        self.multi_point_counter += 1;

        self.types.push(GeometryType::MultiPoint.default_ordering());
        self.multi_points.push_point(value)
    }

    /// Add a new LineString to the end of this array, storing it in the LineStringBuilder
    /// child array.
    ///
    /// # Errors
    ///
    /// This function errors iff the new last item is larger than what O supports.
    pub fn push_line_string(
        &mut self,
        value: Option<&impl LineStringTrait<T = f64>>,
    ) -> Result<()> {
        self.offsets.push(self.line_string_counter);
        self.line_string_counter += 1;

        self.types.push(GeometryType::LineString.default_ordering());
        self.line_strings.push_line_string(value)
    }

    /// Add a new LineString to the end of this array, storing it in the
    /// MultiLineStringBuilder child array.
    ///
    /// # Errors
    ///
    /// This function errors iff the new last item is larger than what O supports.
    pub fn push_line_string_as_multi_line_string(
        &mut self,
        value: Option<&impl LineStringTrait<T = f64>>,
    ) -> Result<()> {
        self.offsets.push(self.multi_line_string_counter);
        self.multi_line_string_counter += 1;

        self.types
            .push(GeometryType::MultiLineString.default_ordering());
        self.multi_line_strings.push_line_string(value)
    }

    /// Add a new Polygon to the end of this array, storing it in the PolygonBuilder
    /// child array.
    ///
    /// # Errors
    ///
    /// This function errors iff the new last item is larger than what O supports.
    pub fn push_polygon(&mut self, value: Option<&impl PolygonTrait<T = f64>>) -> Result<()> {
        self.offsets.push(self.polygon_counter);
        self.polygon_counter += 1;

        self.types.push(GeometryType::Polygon.default_ordering());
        self.polygons.push_polygon(value)
    }

    /// Add a new Polygon to the end of this array, storing it in the MultiPolygonBuilder
    /// child array.
    ///
    /// # Errors
    ///
    /// This function errors iff the new last item is larger than what O supports.
    pub fn push_polygon_as_multi_polygon(
        &mut self,
        value: Option<&impl PolygonTrait<T = f64>>,
    ) -> Result<()> {
        self.offsets.push(self.multi_polygon_counter);
        self.multi_polygon_counter += 1;

        self.types
            .push(GeometryType::MultiPolygon.default_ordering());
        self.multi_polygons.push_polygon(value)
    }

    /// Add a new MultiPoint to the end of this array.
    ///
    /// # Errors
    ///
    /// This function errors iff the new last item is larger than what O supports.
    pub fn push_multi_point(
        &mut self,
        value: Option<&impl MultiPointTrait<T = f64>>,
    ) -> Result<()> {
        self.offsets.push(self.multi_point_counter);
        self.multi_point_counter += 1;

        self.types.push(GeometryType::MultiPoint.default_ordering());
        self.multi_points.push_multi_point(value)
    }

    /// Add a new MultiLineString to the end of this array.
    ///
    /// # Errors
    ///
    /// This function errors iff the new last item is larger than what O supports.
    pub fn push_multi_line_string(
        &mut self,
        value: Option<&impl MultiLineStringTrait<T = f64>>,
    ) -> Result<()> {
        self.offsets.push(self.multi_line_string_counter);
        self.multi_line_string_counter += 1;

        self.types
            .push(GeometryType::MultiLineString.default_ordering());
        self.multi_line_strings.push_multi_line_string(value)
    }

    /// Add a new MultiPolygon to the end of this array.
    ///
    /// # Errors
    ///
    /// This function errors iff the new last item is larger than what O supports.
    pub fn push_multi_polygon(
        &mut self,
        value: Option<&impl MultiPolygonTrait<T = f64>>,
    ) -> Result<()> {
        self.offsets.push(self.multi_polygon_counter);
        self.multi_polygon_counter += 1;

        self.types
            .push(GeometryType::MultiPolygon.default_ordering());
        self.multi_polygons.push_multi_polygon(value)
    }

    pub fn push_geometry(&mut self, value: Option<&'a impl GeometryTrait<T = f64>>) -> Result<()> {
        if let Some(geom) = value {
            match geom.as_type() {
                crate::geo_traits::GeometryType::Point(g) => self.push_point(Some(g)),
                crate::geo_traits::GeometryType::LineString(g) => self.push_line_string(Some(g))?,
                crate::geo_traits::GeometryType::Polygon(g) => self.push_polygon(Some(g))?,
                crate::geo_traits::GeometryType::MultiPoint(p) => self.push_multi_point(Some(p))?,
                crate::geo_traits::GeometryType::MultiLineString(p) => {
                    self.push_multi_line_string(Some(p))?
                }
                crate::geo_traits::GeometryType::MultiPolygon(p) => {
                    self.push_multi_polygon(Some(p))?
                }
                crate::geo_traits::GeometryType::GeometryCollection(_) => {
                    panic!("nested geometry collections not supported")
                }
                _ => todo!(),
            };
        } else {
            todo!("push null geometry")
        }
        Ok(())
    }

    pub fn extend_from_iter(
        &mut self,
        geoms: impl Iterator<Item = Option<&'a (impl GeometryTrait<T = f64> + 'a)>>,
    ) {
        geoms
            .into_iter()
            .try_for_each(|maybe_geom| self.push_geometry(maybe_geom))
            .unwrap();
    }

    pub fn from_geometries(
        geoms: &[impl GeometryTrait<T = f64>],
        coord_type: Option<CoordType>,
    ) -> Self {
        let mut array = Self::with_capacity_and_options_from_iter(
            geoms.iter().map(Some),
            coord_type.unwrap_or_default(),
        );
        array.extend_from_iter(geoms.iter().map(Some));
        array
    }

    pub fn from_nullable_geometries(
        geoms: &[Option<impl GeometryTrait<T = f64>>],
        coord_type: Option<CoordType>,
    ) -> Self {
        let mut array = Self::with_capacity_and_options_from_iter(
            geoms.iter().map(|x| x.as_ref()),
            coord_type.unwrap_or_default(),
        );
        array.extend_from_iter(geoms.iter().map(|x| x.as_ref()));
        array
    }

    pub fn from_wkb<W: OffsetSizeTrait>(
        wkb_objects: &[Option<WKB<'_, W>>],
        coord_type: Option<CoordType>,
    ) -> Result<Self> {
        let wkb_objects2: Vec<Option<WKBGeometry>> = wkb_objects
            .iter()
            .map(|maybe_wkb| maybe_wkb.as_ref().map(|wkb| wkb.to_wkb_object()))
            .collect();
        Ok(Self::from_nullable_geometries(&wkb_objects2, coord_type))
    }

    pub fn finish(self) -> MixedGeometryArray<O> {
        self.into()
    }
}

impl<O: OffsetSizeTrait> Default for MixedGeometryBuilder<O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O: OffsetSizeTrait> IntoArrow for MixedGeometryBuilder<O> {
    type ArrowArray = UnionArray;

    fn into_arrow(self) -> Self::ArrowArray {
        todo!()
    }
}

impl<O: OffsetSizeTrait> From<MixedGeometryBuilder<O>> for MixedGeometryArray<O> {
    fn from(other: MixedGeometryBuilder<O>) -> Self {
        Self::new(
            other.types.into(),
            other.offsets.into(),
            other.points.into(),
            other.line_strings.into(),
            other.polygons.into(),
            other.multi_points.into(),
            other.multi_line_strings.into(),
            other.multi_polygons.into(),
        )
    }
}

// TODO: figure out these trait impl errors
// fn from_geometry_trait_iterator<'a, O: OffsetSizeTrait>(
//     geoms: impl Iterator<Item = impl GeometryTrait<T = f64> + 'a>,
//     prefer_multi: bool
// ) -> MixedGeometryBuilder<O> {
//     let mut array = MixedGeometryBuilder::new();

//     for geom in geoms.into_iter() {
//         match geom.as_type() {
//             GeometryType::Point(point) => {
//                 array.push_valid_point(point);
//                 // if prefer_multi {
//                 //     array.push_point_as_multi_point(Some(point));
//                 // } else {
//                 //     array.push_point(Some(point));
//                 // }
//             }
//             _ => todo!(),
//         };
//         // maybe_geom.
//     }

//     array
// }

#[derive(Debug, Clone, Copy)]
pub struct MixedCapacity {
    /// Simple: just the total number of points, nulls included
    point: usize,
    line_string: LineStringCapacity,
    polygon: PolygonCapacity,
    multi_point: MultiPointCapacity,
    multi_line_string: MultiLineStringCapacity,
    multi_polygon: MultiPolygonCapacity,
}

impl MixedCapacity {
    pub fn new_empty() -> Self {
        Self {
            point: 0,
            line_string: LineStringCapacity::new_empty(),
            polygon: PolygonCapacity::new_empty(),
            multi_point: MultiPointCapacity::new_empty(),
            multi_line_string: MultiLineStringCapacity::new_empty(),
            multi_polygon: MultiPolygonCapacity::new_empty(),
        }
    }

    pub fn total_num_geoms(&self) -> usize {
        let mut total = 0;
        total += self.point;
        total += self.line_string.geom_capacity();
        total += self.polygon.geom_capacity();
        total += self.multi_point.geom_capacity();
        total += self.multi_line_string.geom_capacity();
        total += self.multi_polygon.geom_capacity();
        total
    }

    pub fn point_capacity(&self) -> usize {
        self.point
    }

    pub fn line_string_capacity(&self) -> LineStringCapacity {
        self.line_string
    }

    pub fn polygon_capacity(&self) -> PolygonCapacity {
        self.polygon
    }

    pub fn multi_point_capacity(&self) -> MultiPointCapacity {
        self.multi_point
    }

    pub fn multi_line_string_capacity(&self) -> MultiLineStringCapacity {
        self.multi_line_string
    }

    pub fn multi_polygon_capacity(&self) -> MultiPolygonCapacity {
        self.multi_polygon
    }

    pub fn add_point(&mut self) {
        self.point += 1;
    }

    pub fn add_line_string<'a>(&mut self, line_string: Option<&'a (impl LineStringTrait + 'a)>) {
        self.line_string.add_line_string(line_string);
    }

    pub fn add_polygon<'a>(&mut self, polygon: Option<&'a (impl PolygonTrait + 'a)>) {
        self.polygon.add_polygon(polygon);
    }

    pub fn add_multi_point<'a>(&mut self, multi_point: Option<&'a (impl MultiPointTrait + 'a)>) {
        self.multi_point.add_multi_point(multi_point);
    }

    pub fn add_multi_line_string<'a>(
        &mut self,
        multi_line_string: Option<&'a (impl MultiLineStringTrait + 'a)>,
    ) {
        self.multi_line_string
            .add_multi_line_string(multi_line_string);
    }

    pub fn add_multi_polygon<'a>(
        &mut self,
        multi_polygon: Option<&'a (impl MultiPolygonTrait + 'a)>,
    ) {
        self.multi_polygon.add_multi_polygon(multi_polygon);
    }

    pub fn add_geometry<'a>(&mut self, geom: Option<&'a (impl GeometryTrait + 'a)>) {
        // TODO: what to do about null geometries? We don't know which type they have
        if let Some(geom) = geom {
            match geom.as_type() {
                crate::geo_traits::GeometryType::Point(_) => self.add_point(),
                crate::geo_traits::GeometryType::LineString(g) => self.add_line_string(Some(g)),
                crate::geo_traits::GeometryType::Polygon(g) => self.add_polygon(Some(g)),
                crate::geo_traits::GeometryType::MultiPoint(p) => self.add_multi_point(Some(p)),
                crate::geo_traits::GeometryType::MultiLineString(p) => {
                    self.add_multi_line_string(Some(p))
                }
                crate::geo_traits::GeometryType::MultiPolygon(p) => self.add_multi_polygon(Some(p)),
                crate::geo_traits::GeometryType::GeometryCollection(_) => {
                    panic!("nested geometry collections not supported")
                }
                _ => todo!(),
            }
        }
    }

    pub fn from_geometries<'a>(
        geoms: impl Iterator<Item = Option<&'a (impl GeometryTrait + 'a)>>,
    ) -> Self {
        let mut counter = Self::new_empty();
        for maybe_geom in geoms.into_iter() {
            counter.add_geometry(maybe_geom);
        }
        counter
    }
}

fn from_geo_iterator<'a, O: OffsetSizeTrait>(
    geoms: impl Iterator<Item = &'a geo::Geometry>,
    prefer_multi: bool,
) -> Result<MixedGeometryBuilder<O>> {
    let mut array = MixedGeometryBuilder::new();

    for geom in geoms.into_iter() {
        match geom {
            geo::Geometry::Point(point) => {
                if prefer_multi {
                    array.push_point_as_multi_point(Some(point))?;
                } else {
                    array.push_point(Some(point));
                }
            }
            geo::Geometry::LineString(line_string) => {
                if prefer_multi {
                    array.push_line_string_as_multi_line_string(Some(line_string))?;
                } else {
                    array.push_line_string(Some(line_string))?;
                }
            }
            geo::Geometry::Polygon(polygon) => {
                if prefer_multi {
                    array.push_polygon_as_multi_polygon(Some(polygon))?;
                } else {
                    array.push_polygon(Some(polygon))?;
                }
            }
            geo::Geometry::MultiPoint(multi_point) => {
                array.push_multi_point(Some(multi_point))?;
            }
            geo::Geometry::MultiLineString(multi_line_string) => {
                array.push_multi_line_string(Some(multi_line_string))?;
            }
            geo::Geometry::MultiPolygon(multi_polygon) => {
                array.push_multi_polygon(Some(multi_polygon))?;
            }
            _ => todo!(),
        }
    }

    Ok(array)
}

impl<O: OffsetSizeTrait> TryFrom<Vec<geo::Geometry>> for MixedGeometryBuilder<O> {
    type Error = GeoArrowError;

    fn try_from(value: Vec<geo::Geometry>) -> std::result::Result<Self, Self::Error> {
        from_geo_iterator(value.iter(), true)
    }
}

impl<O: OffsetSizeTrait> TryFrom<WKBArray<O>> for MixedGeometryBuilder<O> {
    type Error = GeoArrowError;

    fn try_from(value: WKBArray<O>) -> std::result::Result<Self, Self::Error> {
        assert_eq!(
            value.nulls().map_or(0, |validity| validity.null_count()),
            0,
            "Parsing a WKBArray with null elements not supported",
        );

        let wkb_objects: Vec<Option<WKB<'_, O>>> = value.iter().collect();
        Self::from_wkb(&wkb_objects, Default::default())
    }
}
