use std::sync::Arc;

use crate::array::metadata::ArrayMetadata;
use crate::array::mixed::array::GeometryType;
use crate::array::mixed::MixedCapacity;
use crate::array::{
    CoordType, LineStringBuilder, MixedGeometryArray, MultiLineStringBuilder, MultiPointBuilder,
    MultiPolygonBuilder, PointBuilder, PolygonBuilder, WKBArray,
};
use crate::error::{GeoArrowError, Result};
use crate::io::wkb::reader::WKBGeometry;
use crate::scalar::WKB;
use crate::trait_::{ArrayAccessor, GeometryArrayBuilder, IntoArrow};
use crate::{ArrayBase, NativeArray};
use arrow_array::{OffsetSizeTrait, UnionArray};
use geo_traits::*;

pub(crate) const DEFAULT_PREFER_MULTI: bool = false;

/// The GeoArrow equivalent to a `Vec<Option<Geometry>>`: a mutable collection of Geometries.
///
/// This currently has the caveat that these geometries must be a _primitive_ geometry type. This
/// does not currently support nested GeometryCollection objects.
///
/// Converting an [`MixedGeometryBuilder`] into a [`MixedGeometryArray`] is `O(1)`.
///
/// # Invariants
///
/// - All arrays must have the same dimension
/// - All arrays must have the same coordinate layout (interleaved or separated)
#[derive(Debug)]
pub struct MixedGeometryBuilder<const D: usize> {
    metadata: Arc<ArrayMetadata>,

    // Invariant: every item in `types` is `> 0 && < fields.len()`
    types: Vec<i8>,

    pub(crate) points: PointBuilder<D>,
    pub(crate) line_strings: LineStringBuilder<D>,
    pub(crate) polygons: PolygonBuilder<D>,
    pub(crate) multi_points: MultiPointBuilder<D>,
    pub(crate) multi_line_strings: MultiLineStringBuilder<D>,
    pub(crate) multi_polygons: MultiPolygonBuilder<D>,

    // Invariant: `offsets.len() == types.len()`
    offsets: Vec<i32>,

    /// Whether to prefer multi or single arrays for new geometries.
    ///
    /// E.g. if this is `true` and a Point geometry is added, it will be added to the
    /// MultiPointBuilder. If this is `false`, the Point geometry will be added to the
    /// PointBuilder.
    ///
    /// The idea is that always adding multi-geometries will make it easier to downcast later.
    pub(crate) prefer_multi: bool,
}

impl<'a, const D: usize> MixedGeometryBuilder<D> {
    /// Creates a new empty [`MixedGeometryBuilder`].
    pub fn new() -> Self {
        Self::new_with_options(Default::default(), Default::default(), DEFAULT_PREFER_MULTI)
    }

    pub fn new_with_options(
        coord_type: CoordType,
        metadata: Arc<ArrayMetadata>,
        prefer_multi: bool,
    ) -> Self {
        Self::with_capacity_and_options(Default::default(), coord_type, metadata, prefer_multi)
    }

    /// Creates a new [`MixedGeometryBuilder`] with given capacity and no validity.
    pub fn with_capacity(capacity: MixedCapacity) -> Self {
        Self::with_capacity_and_options(
            capacity,
            Default::default(),
            Default::default(),
            DEFAULT_PREFER_MULTI,
        )
    }

    pub fn with_capacity_and_options(
        capacity: MixedCapacity,
        coord_type: CoordType,
        metadata: Arc<ArrayMetadata>,
        prefer_multi: bool,
    ) -> Self {
        // Don't store array metadata on child arrays
        Self {
            metadata,
            types: vec![],
            points: PointBuilder::with_capacity_and_options(
                capacity.point,
                coord_type,
                Default::default(),
            ),
            line_strings: LineStringBuilder::with_capacity_and_options(
                capacity.line_string,
                coord_type,
                Default::default(),
            ),
            polygons: PolygonBuilder::with_capacity_and_options(
                capacity.polygon,
                coord_type,
                Default::default(),
            ),
            multi_points: MultiPointBuilder::with_capacity_and_options(
                capacity.multi_point,
                coord_type,
                Default::default(),
            ),
            multi_line_strings: MultiLineStringBuilder::with_capacity_and_options(
                capacity.multi_line_string,
                coord_type,
                Default::default(),
            ),
            multi_polygons: MultiPolygonBuilder::with_capacity_and_options(
                capacity.multi_polygon,
                coord_type,
                Default::default(),
            ),
            offsets: vec![],
            prefer_multi,
        }
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

    pub fn finish(self) -> MixedGeometryArray<D> {
        self.into()
    }

    pub fn with_capacity_from_iter(
        geoms: impl Iterator<Item = Option<&'a (impl GeometryTrait + 'a)>>,
    ) -> Result<Self> {
        Self::with_capacity_and_options_from_iter(
            geoms,
            Default::default(),
            Default::default(),
            DEFAULT_PREFER_MULTI,
        )
    }

    pub fn with_capacity_and_options_from_iter(
        geoms: impl Iterator<Item = Option<&'a (impl GeometryTrait + 'a)>>,
        coord_type: CoordType,
        metadata: Arc<ArrayMetadata>,
        prefer_multi: bool,
    ) -> Result<Self> {
        let counter = MixedCapacity::from_geometries(geoms)?;
        Ok(Self::with_capacity_and_options(
            counter,
            coord_type,
            metadata,
            prefer_multi,
        ))
    }

    pub fn reserve_from_iter(
        &mut self,
        geoms: impl Iterator<Item = Option<&'a (impl GeometryTrait + 'a)>>,
    ) -> Result<()> {
        let counter = MixedCapacity::from_geometries(geoms)?;
        self.reserve(counter);
        Ok(())
    }

    pub fn reserve_exact_from_iter(
        &mut self,
        geoms: impl Iterator<Item = Option<&'a (impl GeometryTrait + 'a)>>,
    ) -> Result<()> {
        let counter = MixedCapacity::from_geometries(geoms)?;
        self.reserve_exact(counter);
        Ok(())
    }

    /// Add a new Point to the end of this array.
    ///
    /// If `self.prefer_multi` is `true`, it will be stored in the `MultiPointBuilder` child
    /// array. Otherwise, it will be stored in the `PointBuilder` child array.
    #[inline]
    pub fn push_point(&mut self, value: Option<&impl PointTrait<T = f64>>) -> Result<()> {
        if self.prefer_multi {
            self.add_multi_point_type();
            self.multi_points.push_point(value)
        } else {
            self.add_point_type();
            self.points.push_point(value);
            Ok(())
        }
    }

    #[inline]
    pub(crate) fn add_point_type(&mut self) {
        self.offsets.push(self.points.len().try_into().unwrap());
        self.types.push(GeometryType::Point.default_ordering());
    }

    /// Add a new LineString to the end of this array.
    ///
    /// If `self.prefer_multi` is `true`, it will be stored in the `MultiLineStringBuilder` child
    /// array. Otherwise, it will be stored in the `LineStringBuilder` child array.
    ///
    /// # Errors
    ///
    /// This function errors iff the new last item is larger than what O supports.
    #[inline]
    pub fn push_line_string(
        &mut self,
        value: Option<&impl LineStringTrait<T = f64>>,
    ) -> Result<()> {
        if self.prefer_multi {
            self.add_multi_line_string_type();
            self.multi_line_strings.push_line_string(value)
        } else {
            self.add_line_string_type();
            self.line_strings.push_line_string(value)
        }
    }

    #[inline]
    pub(crate) fn add_line_string_type(&mut self) {
        self.offsets
            .push(self.line_strings.len().try_into().unwrap());
        self.types.push(GeometryType::LineString.default_ordering());
    }

    /// Add a new Polygon to the end of this array.
    ///
    /// If `self.prefer_multi` is `true`, it will be stored in the `MultiPolygonBuilder` child
    /// array. Otherwise, it will be stored in the `PolygonBuilder` child array.
    ///
    /// # Errors
    ///
    /// This function errors iff the new last item is larger than what O supports.
    #[inline]
    pub fn push_polygon(&mut self, value: Option<&impl PolygonTrait<T = f64>>) -> Result<()> {
        if self.prefer_multi {
            self.add_multi_polygon_type();
            self.multi_polygons.push_polygon(value)
        } else {
            self.add_polygon_type();
            self.polygons.push_polygon(value)
        }
    }

    #[inline]
    pub(crate) fn add_polygon_type(&mut self) {
        self.offsets.push(self.polygons.len().try_into().unwrap());
        self.types.push(GeometryType::Polygon.default_ordering());
    }

    /// Add a new MultiPoint to the end of this array.
    ///
    /// # Errors
    ///
    /// This function errors iff the new last item is larger than what O supports.
    #[inline]
    pub fn push_multi_point(
        &mut self,
        value: Option<&impl MultiPointTrait<T = f64>>,
    ) -> Result<()> {
        self.add_multi_point_type();
        self.multi_points.push_multi_point(value)
    }

    #[inline]
    pub(crate) fn add_multi_point_type(&mut self) {
        self.offsets
            .push(self.multi_points.len().try_into().unwrap());
        self.types.push(GeometryType::MultiPoint.default_ordering());
    }

    /// Add a new MultiLineString to the end of this array.
    ///
    /// # Errors
    ///
    /// This function errors iff the new last item is larger than what O supports.
    #[inline]
    pub fn push_multi_line_string(
        &mut self,
        value: Option<&impl MultiLineStringTrait<T = f64>>,
    ) -> Result<()> {
        self.add_multi_line_string_type();
        self.multi_line_strings.push_multi_line_string(value)
    }

    #[inline]
    pub(crate) fn add_multi_line_string_type(&mut self) {
        self.offsets
            .push(self.multi_line_strings.len().try_into().unwrap());
        self.types
            .push(GeometryType::MultiLineString.default_ordering());
    }

    /// Add a new MultiPolygon to the end of this array.
    ///
    /// # Errors
    ///
    /// This function errors iff the new last item is larger than what O supports.
    #[inline]
    pub fn push_multi_polygon(
        &mut self,
        value: Option<&impl MultiPolygonTrait<T = f64>>,
    ) -> Result<()> {
        self.add_multi_polygon_type();
        self.multi_polygons.push_multi_polygon(value)
    }

    #[inline]
    pub(crate) fn add_multi_polygon_type(&mut self) {
        self.offsets
            .push(self.multi_polygons.len().try_into().unwrap());
        self.types
            .push(GeometryType::MultiPolygon.default_ordering());
    }

    #[inline]
    pub fn push_geometry(&mut self, value: Option<&'a impl GeometryTrait<T = f64>>) -> Result<()> {
        use geo_traits::GeometryType::*;

        if let Some(geom) = value {
            match geom.as_type() {
                Point(g) => {
                    self.push_point(Some(g))?;
                }
                LineString(g) => {
                    self.push_line_string(Some(g))?;
                }
                Polygon(g) => {
                    self.push_polygon(Some(g))?;
                }
                MultiPoint(p) => self.push_multi_point(Some(p))?,
                MultiLineString(p) => self.push_multi_line_string(Some(p))?,
                MultiPolygon(p) => self.push_multi_polygon(Some(p))?,
                GeometryCollection(gc) => {
                    if gc.num_geometries() == 1 {
                        self.push_geometry(Some(&gc.geometry(0).unwrap()))?
                    } else {
                        return Err(GeoArrowError::General(
                            "nested geometry collections not supported".to_string(),
                        ));
                    }
                }
                Rect(_) | Triangle(_) | Line(_) => todo!(),
            };
        } else {
            self.push_null();
        }
        Ok(())
    }

    #[inline]
    pub fn push_null(&mut self) {
        todo!("push null geometry")
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

    /// Create this builder from a slice of Geometries.
    pub fn from_geometries(
        geoms: &[impl GeometryTrait<T = f64>],
        coord_type: Option<CoordType>,
        metadata: Arc<ArrayMetadata>,
        prefer_multi: bool,
    ) -> Result<Self> {
        let mut array = Self::with_capacity_and_options_from_iter(
            geoms.iter().map(Some),
            coord_type.unwrap_or_default(),
            metadata,
            prefer_multi,
        )?;
        array.extend_from_iter(geoms.iter().map(Some));
        Ok(array)
    }

    /// Create this builder from a slice of nullable Geometries.
    pub fn from_nullable_geometries(
        geoms: &[Option<impl GeometryTrait<T = f64>>],
        coord_type: Option<CoordType>,
        metadata: Arc<ArrayMetadata>,
        prefer_multi: bool,
    ) -> Result<Self> {
        let mut array = Self::with_capacity_and_options_from_iter(
            geoms.iter().map(|x| x.as_ref()),
            coord_type.unwrap_or_default(),
            metadata,
            prefer_multi,
        )?;
        array.extend_from_iter(geoms.iter().map(|x| x.as_ref()));
        Ok(array)
    }

    pub(crate) fn from_wkb<W: OffsetSizeTrait>(
        wkb_objects: &[Option<WKB<'_, W>>],
        coord_type: Option<CoordType>,
        metadata: Arc<ArrayMetadata>,
        prefer_multi: bool,
    ) -> Result<Self> {
        let wkb_objects2: Vec<Option<WKBGeometry>> = wkb_objects
            .iter()
            .map(|maybe_wkb| maybe_wkb.as_ref().map(|wkb| wkb.to_wkb_object()))
            .collect();
        Self::from_nullable_geometries(&wkb_objects2, coord_type, metadata, prefer_multi)
    }
}

impl<const D: usize> Default for MixedGeometryBuilder<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const D: usize> IntoArrow for MixedGeometryBuilder<D> {
    type ArrowArray = UnionArray;

    fn into_arrow(self) -> Self::ArrowArray {
        todo!()
    }
}

impl<const D: usize> From<MixedGeometryBuilder<D>> for MixedGeometryArray<D> {
    fn from(other: MixedGeometryBuilder<D>) -> Self {
        Self::new(
            other.types.into(),
            other.offsets.into(),
            other.points.into(),
            other.line_strings.into(),
            other.polygons.into(),
            other.multi_points.into(),
            other.multi_line_strings.into(),
            other.multi_polygons.into(),
            other.metadata,
        )
    }
}

impl<G: GeometryTrait<T = f64>, const D: usize> TryFrom<&[G]> for MixedGeometryBuilder<D> {
    type Error = GeoArrowError;

    fn try_from(geoms: &[G]) -> Result<Self> {
        Self::from_geometries(geoms, Default::default(), Default::default(), true)
    }
}

impl<G: GeometryTrait<T = f64>, const D: usize> TryFrom<Vec<Option<G>>>
    for MixedGeometryBuilder<D>
{
    type Error = GeoArrowError;

    fn try_from(geoms: Vec<Option<G>>) -> Result<Self> {
        Self::from_nullable_geometries(&geoms, Default::default(), Default::default(), true)
    }
}

impl<O: OffsetSizeTrait, const D: usize> TryFrom<WKBArray<O>> for MixedGeometryBuilder<D> {
    type Error = GeoArrowError;

    fn try_from(value: WKBArray<O>) -> std::result::Result<Self, Self::Error> {
        assert_eq!(
            value.nulls().map_or(0, |validity| validity.null_count()),
            0,
            "Parsing a WKBArray with null elements not supported",
        );

        let metadata = value.metadata.clone();
        let wkb_objects: Vec<Option<WKB<'_, O>>> = value.iter().collect();
        Self::from_wkb(&wkb_objects, Default::default(), metadata, true)
    }
}

impl<const D: usize> GeometryArrayBuilder for MixedGeometryBuilder<D> {
    fn len(&self) -> usize {
        self.types.len()
    }

    fn nulls(&self) -> &arrow_buffer::NullBufferBuilder {
        // Take this method off trait
        todo!()
    }

    fn new() -> Self {
        Self::new()
    }

    fn into_array_ref(self) -> Arc<dyn arrow_array::Array> {
        Arc::new(self.into_arrow())
    }

    fn with_geom_capacity_and_options(
        _geom_capacity: usize,
        coord_type: CoordType,
        metadata: Arc<ArrayMetadata>,
    ) -> Self {
        // We don't know where to allocate the capacity
        Self::with_capacity_and_options(
            Default::default(),
            coord_type,
            metadata,
            DEFAULT_PREFER_MULTI,
        )
    }

    fn push_geometry(&mut self, value: Option<&impl GeometryTrait<T = f64>>) -> Result<()> {
        self.push_geometry(value)
    }

    fn finish(self) -> std::sync::Arc<dyn NativeArray> {
        Arc::new(self.finish())
    }

    fn coord_type(&self) -> CoordType {
        self.points.coord_type()
    }

    fn set_metadata(&mut self, metadata: Arc<ArrayMetadata>) {
        self.metadata = metadata;
    }

    fn metadata(&self) -> Arc<ArrayMetadata> {
        self.metadata.clone()
    }
}
