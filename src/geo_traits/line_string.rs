use super::CoordTrait;
use geo::{Coord, CoordNum, LineString};
use std::iter::Cloned;
use std::slice::Iter;

pub trait LineStringTrait {
    type T: CoordNum;
    type ItemType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;
    type Iter<'a>: ExactSizeIterator<Item = Self::ItemType<'a>>
    where
        Self: 'a;

    /// An iterator over the coords in this LineString
    fn coords(&self) -> Self::Iter<'_>;

    /// The number of coords in this LineString
    fn num_coords(&self) -> usize;

    /// Access to a specified point in this LineString
    /// Will return None if the provided index is out of bounds
    fn coord(&self, i: usize) -> Option<Self::ItemType<'_>>;
}

impl<'a, T: CoordNum> LineStringTrait for LineString<T> {
    type T = T;
    type ItemType = Coord<Self::T>;
    type Iter = Cloned<Iter<'a, Self::ItemType<'a>>>;

    fn coords(&self) -> Self::Iter<'_> {
        // TODO: remove cloned
        self.0.iter().cloned()
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    fn coord(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.0.get(i).cloned()
    }
}

impl<'a, T: CoordNum + 'a> LineStringTrait for &LineString<T> {
    type T = T;
    type ItemType = Coord<Self::T>;
    type Iter = Cloned<Iter<'a, Self::ItemType<'a>>>;

    fn coords(&self) -> Self::Iter<'_> {
        self.0.iter().cloned()
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    fn coord(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.0.get(i).cloned()
    }
}
