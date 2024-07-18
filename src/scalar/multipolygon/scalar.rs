use crate::algorithm::native::bounding_rect::bounding_rect_multipolygon;
use crate::algorithm::native::eq::multi_polygon_eq;
use crate::array::util::OffsetBufferUtils;
use crate::array::{CoordBuffer, MultiPolygonArray};
use crate::geo_traits::MultiPolygonTrait;
use crate::io::geo::multi_polygon_to_geo;
use crate::scalar::Polygon;
use crate::trait_::{GeometryArraySelfMethods, GeometryScalarTrait};
use arrow_array::OffsetSizeTrait;
use arrow_buffer::OffsetBuffer;
use rstar::{RTreeObject, AABB};
use std::borrow::Cow;

/// An Arrow equivalent of a MultiPolygon
#[derive(Debug, Clone)]
pub struct MultiPolygon<'a, O: OffsetSizeTrait, const D: usize> {
    pub(crate) coords: Cow<'a, CoordBuffer<D>>,

    /// Offsets into the polygon array where each geometry starts
    pub(crate) geom_offsets: Cow<'a, OffsetBuffer<O>>,

    /// Offsets into the ring array where each polygon starts
    pub(crate) polygon_offsets: Cow<'a, OffsetBuffer<O>>,

    /// Offsets into the coordinate array where each ring starts
    pub(crate) ring_offsets: Cow<'a, OffsetBuffer<O>>,

    pub(crate) geom_index: usize,

    start_offset: usize,
}

impl<'a, O: OffsetSizeTrait, const D: usize> MultiPolygon<'a, O, D> {
    pub fn new(
        coords: Cow<'a, CoordBuffer<D>>,
        geom_offsets: Cow<'a, OffsetBuffer<O>>,
        polygon_offsets: Cow<'a, OffsetBuffer<O>>,
        ring_offsets: Cow<'a, OffsetBuffer<O>>,
        geom_index: usize,
    ) -> Self {
        let (start_offset, _) = geom_offsets.start_end(geom_index);
        Self {
            coords,
            geom_offsets,
            polygon_offsets,
            ring_offsets,
            geom_index,
            start_offset,
        }
    }

    pub fn new_borrowed(
        coords: &'a CoordBuffer<D>,
        geom_offsets: &'a OffsetBuffer<O>,
        polygon_offsets: &'a OffsetBuffer<O>,
        ring_offsets: &'a OffsetBuffer<O>,
        geom_index: usize,
    ) -> Self {
        Self::new(
            Cow::Borrowed(coords),
            Cow::Borrowed(geom_offsets),
            Cow::Borrowed(polygon_offsets),
            Cow::Borrowed(ring_offsets),
            geom_index,
        )
    }

    pub fn new_owned(
        coords: CoordBuffer<D>,
        geom_offsets: OffsetBuffer<O>,
        polygon_offsets: OffsetBuffer<O>,
        ring_offsets: OffsetBuffer<O>,
        geom_index: usize,
    ) -> Self {
        Self::new(
            Cow::Owned(coords),
            Cow::Owned(geom_offsets),
            Cow::Owned(polygon_offsets),
            Cow::Owned(ring_offsets),
            geom_index,
        )
    }

    /// Extracts the owned data.
    ///
    /// Clones the data if it is not already owned.
    pub fn into_owned(self) -> Self {
        let arr = MultiPolygonArray::new(
            self.coords.into_owned(),
            self.geom_offsets.into_owned(),
            self.polygon_offsets.into_owned(),
            self.ring_offsets.into_owned(),
            None,
            Default::default(),
        );
        let sliced_arr = arr.owned_slice(self.geom_index, 1);
        Self::new_owned(
            sliced_arr.coords,
            sliced_arr.geom_offsets,
            sliced_arr.polygon_offsets,
            sliced_arr.ring_offsets,
            0,
        )
    }

    pub fn into_owned_inner(
        self,
    ) -> (
        CoordBuffer<D>,
        OffsetBuffer<O>,
        OffsetBuffer<O>,
        OffsetBuffer<O>,
        usize,
    ) {
        let owned = self.into_owned();
        (
            owned.coords.into_owned(),
            owned.geom_offsets.into_owned(),
            owned.polygon_offsets.into_owned(),
            owned.ring_offsets.into_owned(),
            owned.geom_index,
        )
    }
}

impl<'a, O: OffsetSizeTrait> GeometryScalarTrait for MultiPolygon<'a, O, 2> {
    type ScalarGeo = geo::MultiPolygon;

    fn to_geo(&self) -> Self::ScalarGeo {
        self.into()
    }

    fn to_geo_geometry(&self) -> geo::Geometry {
        geo::Geometry::MultiPolygon(self.to_geo())
    }

    #[cfg(feature = "geos")]
    fn to_geos(&self) -> std::result::Result<geos::Geometry, geos::Error> {
        self.try_into()
    }
}

impl<'a, O: OffsetSizeTrait> MultiPolygonTrait for MultiPolygon<'a, O, 2> {
    type T = f64;
    type ItemType<'b> = Polygon<'a, O> where Self: 'b;

    fn num_polygons(&self) -> usize {
        let (start, end) = self.geom_offsets.start_end(self.geom_index);
        end - start
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        Polygon::new(
            self.coords.clone(),
            self.polygon_offsets.clone(),
            self.ring_offsets.clone(),
            self.start_offset + i,
        )
    }
}

impl<'a, O: OffsetSizeTrait> MultiPolygonTrait for &'a MultiPolygon<'a, O, 2> {
    type T = f64;
    type ItemType<'b> = Polygon<'a, O> where Self: 'b;

    fn num_polygons(&self) -> usize {
        let (start, end) = self.geom_offsets.start_end(self.geom_index);
        end - start
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        Polygon::new(
            self.coords.clone(),
            self.polygon_offsets.clone(),
            self.ring_offsets.clone(),
            self.start_offset + i,
        )
    }
}

impl<O: OffsetSizeTrait> From<MultiPolygon<'_, O, 2>> for geo::MultiPolygon {
    fn from(value: MultiPolygon<'_, O, 2>) -> Self {
        (&value).into()
    }
}

impl<O: OffsetSizeTrait> From<&MultiPolygon<'_, O, 2>> for geo::MultiPolygon {
    fn from(value: &MultiPolygon<'_, O, 2>) -> Self {
        multi_polygon_to_geo(value)
    }
}

impl<O: OffsetSizeTrait> From<MultiPolygon<'_, O, 2>> for geo::Geometry {
    fn from(value: MultiPolygon<'_, O, 2>) -> Self {
        geo::Geometry::MultiPolygon(value.into())
    }
}

impl<O: OffsetSizeTrait> RTreeObject for MultiPolygon<'_, O, 2> {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let (lower, upper) = bounding_rect_multipolygon(self);
        AABB::from_corners(lower, upper)
    }
}

impl<O: OffsetSizeTrait, G: MultiPolygonTrait<T = f64>> PartialEq<G> for MultiPolygon<'_, O, 2> {
    fn eq(&self, other: &G) -> bool {
        multi_polygon_eq(self, other)
    }
}

#[cfg(test)]
mod test {
    use crate::array::MultiPolygonArray;
    use crate::test::multipolygon::{mp0, mp1};
    use crate::trait_::GeometryArrayAccessor;

    /// Test Eq where the current index is true but another index is false
    #[test]
    fn test_eq_other_index_false() {
        let arr1: MultiPolygonArray<i32, 2> = vec![mp0(), mp1()].as_slice().into();
        let arr2: MultiPolygonArray<i32, 2> = vec![mp0(), mp0()].as_slice().into();

        assert_eq!(arr1.value(0), arr2.value(0));
        assert_ne!(arr1.value(1), arr2.value(1));
    }
}
