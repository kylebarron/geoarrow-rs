use super::line_string::LineStringTrait;
use geo::{CoordNum, LineString, Polygon};
use std::iter::Cloned;
use std::slice::Iter;

pub trait PolygonTrait {
    type T: CoordNum;
    type ItemType<'a>: 'a + LineStringTrait<T = Self::T>
    where
        Self: 'a;
    type Iter<'a>: ExactSizeIterator<Item = Self::ItemType<'a>>
    where
        Self: 'a;

    /// The exterior ring of the polygon
    fn exterior(&self) -> Option<Self::ItemType<'_>>;

    /// An iterator of the interior rings of this Polygon
    fn interiors(&self) -> Self::Iter<'_>;

    /// The number of interior rings in this Polygon
    fn num_interiors(&self) -> usize;

    /// Access to a specified interior ring in this Polygon
    /// Will return None if the provided index is out of bounds
    fn interior(&self, i: usize) -> Option<Self::ItemType<'_>>;
}

impl<'a, T: CoordNum + 'a> PolygonTrait for Polygon<T> {
    type T = T;
    type ItemType = LineString<Self::T>;
    type Iter = Cloned<Iter<'a, Self::ItemType<'a>>>;

    fn exterior(&self) -> Option<Self::ItemType<'_>> {
        // geo-types doesn't really have a way to describe an empty polygon
        Some(Polygon::exterior(self).clone())
    }

    fn interiors(&'a self) -> Self::Iter<'_> {
        Polygon::interiors(self).iter().cloned()
    }

    fn num_interiors(&self) -> usize {
        Polygon::interiors(self).len()
    }

    fn interior(&self, i: usize) -> Option<Self::ItemType<'_>> {
        Polygon::interiors(self).get(i).cloned()
    }
}

impl<'a, T: CoordNum + 'a> PolygonTrait for &Polygon<T> {
    type T = T;
    type ItemType = LineString<Self::T>;
    type Iter = Cloned<Iter<'a, Self::ItemType<'a>>>;

    fn exterior(&self) -> Option<Self::ItemType<'_>> {
        Some(Polygon::exterior(self).clone())
    }

    fn interiors(&'a self) -> Self::Iter<'_> {
        Polygon::interiors(self).iter().cloned()
    }

    fn num_interiors(&self) -> usize {
        Polygon::interiors(self).len()
    }

    fn interior(&self, i: usize) -> Option<Self::ItemType<'_>> {
        Polygon::interiors(self).get(i).cloned()
    }
}
