use crate::algorithm::native::eq::geometry_collection_eq;
use crate::array::{GeometryCollectionArray, MixedGeometryArray};
use crate::datatypes::Dimension;
use crate::scalar::{Geometry, GeometryCollection};
use arrow_buffer::OffsetBuffer;
use geo_traits::GeometryCollectionTrait;

#[derive(Clone, Debug)]
pub struct OwnedGeometryCollection {
    array: MixedGeometryArray<D>,

    /// Offsets into the geometry array where each geometry starts
    geom_offsets: OffsetBuffer<i32>,

    geom_index: usize,
}

impl OwnedGeometryCollection<D> {
    pub fn new(
        array: MixedGeometryArray<D>,
        geom_offsets: OffsetBuffer<i32>,
        geom_index: usize,
    ) -> Self {
        Self {
            array,
            geom_offsets,
            geom_index,
        }
    }
}

impl<'a> From<&'a OwnedGeometryCollection<D>> for GeometryCollection<'a> {
    fn from(value: &'a OwnedGeometryCollection<D>) -> Self {
        Self::new(&value.array, &value.geom_offsets, value.geom_index)
    }
}

impl<'a> From<&'a OwnedGeometryCollection<2>> for geo::GeometryCollection {
    fn from(value: &'a OwnedGeometryCollection<2>) -> Self {
        let geom = GeometryCollection::from(value);
        geom.into()
    }
}

impl<'a> From<GeometryCollection<'a>> for OwnedGeometryCollection<D> {
    fn from(value: GeometryCollection<'a>) -> Self {
        let (array, geom_offsets, geom_index) = value.into_inner();
        Self::new(array.clone(), geom_offsets.clone(), geom_index)
    }
}

impl From<OwnedGeometryCollection<D>> for GeometryCollectionArray<D> {
    fn from(value: OwnedGeometryCollection<D>) -> Self {
        Self::new(value.array, value.geom_offsets, None, Default::default())
    }
}

impl GeometryCollectionTrait for OwnedGeometryCollection<D> {
    type T = f64;
    type GeometryType<'b> = Geometry<'b> where Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        match self.coords.dim() {
            Dimension::XY => geo_traits::Dimensions::Xy,
            Dimension::XYZ => geo_traits::Dimensions::Xyz,
        }
    }

    fn num_geometries(&self) -> usize {
        GeometryCollection::from(self).num_geometries()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        GeometryCollection::from(self).geometry_unchecked(i)
    }
}

impl<G: GeometryCollectionTrait<T = f64>> PartialEq<G> for OwnedGeometryCollection<2> {
    fn eq(&self, other: &G) -> bool {
        geometry_collection_eq(self, other)
    }
}
