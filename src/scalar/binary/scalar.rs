use crate::trait_::GeometryScalarTrait;
use arrow2::array::BinaryArray;
use arrow2::types::Offset;
use geo::BoundingRect;
#[cfg(feature = "geozero")]
use geozero::ToGeo;
use rstar::{RTreeObject, AABB};
use std::borrow::Cow;

/// An Arrow equivalent of a Point
#[derive(Debug, Clone)]
pub struct WKB<'a, O: Offset> {
    pub arr: Cow<'a, BinaryArray<O>>,
    pub geom_index: usize,
}

impl<'a, O: Offset> WKB<'a, O> {
    pub fn new(arr: Cow<'a, BinaryArray<O>>, geom_index: usize) -> Self {
        Self { arr, geom_index }
    }

    pub fn new_borrowed(arr: &'a BinaryArray<O>, geom_index: usize) -> Self {
        Self {
            arr: Cow::Borrowed(arr),
            geom_index,
        }
    }

    pub fn new_owned(arr: BinaryArray<O>, geom_index: usize) -> Self {
        Self {
            arr: Cow::Owned(arr),
            geom_index,
        }
    }
}

impl<'a, O: Offset> GeometryScalarTrait<'a> for WKB<'a, O> {
    type ScalarGeo = geo::Geometry;

    fn to_geo(&self) -> Self::ScalarGeo {
        self.into()
    }
}

impl<'a, O: Offset> AsRef<[u8]> for WKB<'a, O> {
    fn as_ref(&self) -> &[u8] {
        self.arr.value(self.geom_index)
    }
}

#[cfg(feature = "geozero")]
impl<O: Offset> From<WKB<'_, O>> for geo::Geometry {
    fn from(value: WKB<'_, O>) -> Self {
        (&value).into()
    }
}

#[cfg(feature = "geozero")]
impl<O: Offset> From<&WKB<'_, O>> for geo::Geometry {
    fn from(value: &WKB<'_, O>) -> Self {
        let buf = value.arr.value(value.geom_index);
        geozero::wkb::Wkb(buf).to_geo().unwrap()
    }
}

#[cfg(not(feature = "geozero"))]
impl<O: Offset> From<WKB<'_, O>> for geo::Geometry {
    fn from(_value: WKB<'_, O>) -> Self {
        (&_value).into()
    }
}

#[cfg(not(feature = "geozero"))]
impl<O: Offset> From<&WKB<'_, O>> for geo::Geometry {
    fn from(_value: &WKB<'_, O>) -> Self {
        panic!("Activate the 'geozero' feature to convert WKB items to geo::Geometry.")
    }
}

impl<O: Offset> RTreeObject for WKB<'_, O> {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let geom: geo::Geometry = self.into();
        let rect = geom.bounding_rect().unwrap();
        let lower: [f64; 2] = rect.min().into();
        let upper: [f64; 2] = rect.max().into();
        AABB::from_corners(lower, upper)
    }
}

impl<O: Offset> PartialEq for WKB<'_, O> {
    fn eq(&self, other: &Self) -> bool {
        self.arr.value(self.geom_index) == other.arr.value(other.geom_index)
    }
}
