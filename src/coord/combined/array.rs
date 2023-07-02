use arrow2::array::Array;

use crate::{Coord, GeometryArrayTrait, InterleavedCoordBuffer, SeparatedCoordBuffer};

#[derive(Debug, Clone)]
pub enum CoordBuffer {
    Interleaved(InterleavedCoordBuffer),
    Separated(SeparatedCoordBuffer),
}

impl CoordBuffer {
    pub fn get_x(&self, i: usize) -> f64 {
        let geo_coord: geo::Coord = self.value(i).into();
        geo_coord.x
    }

    pub fn get_y(&self, i: usize) -> f64 {
        let geo_coord: geo::Coord = self.value(i).into();
        geo_coord.y
    }
}

impl<'a> GeometryArrayTrait<'a> for CoordBuffer {
    type ArrowArray = Box<dyn Array>;
    type Scalar = Coord<'a>;
    type ScalarGeo = geo::Coord;

    fn value(&'a self, i: usize) -> Self::Scalar {
        match self {
            CoordBuffer::Interleaved(c) => Coord::Interleaved(c.value(i)),
            CoordBuffer::Separated(c) => Coord::Separated(c.value(i)),
        }
    }

    fn into_arrow(self) -> Self::ArrowArray {
        match self {
            CoordBuffer::Interleaved(c) => c.into_arrow().boxed(),
            CoordBuffer::Separated(c) => c.into_arrow().boxed(),
        }
    }

    fn len(&self) -> usize {
        match self {
            CoordBuffer::Interleaved(c) => c.len(),
            CoordBuffer::Separated(c) => c.len(),
        }
    }

    fn validity(&self) -> Option<&arrow2::bitmap::Bitmap> {
        panic!("coordinate arrays don't have their own validity arrays")
    }

    fn slice(&self, offset: usize, length: usize) -> Self {
        match self {
            CoordBuffer::Interleaved(c) => CoordBuffer::Interleaved(c.slice(offset, length)),
            CoordBuffer::Separated(c) => CoordBuffer::Separated(c.slice(offset, length)),
        }
    }

    unsafe fn slice_unchecked(&self, offset: usize, length: usize) -> Self {
        match self {
            CoordBuffer::Interleaved(c) => {
                CoordBuffer::Interleaved(c.slice_unchecked(offset, length))
            }
            CoordBuffer::Separated(c) => CoordBuffer::Separated(c.slice_unchecked(offset, length)),
        }
    }

    fn to_boxed(&self) -> Box<Self> {
        match self {
            CoordBuffer::Interleaved(c) => self.to_boxed(),
            CoordBuffer::Separated(c) => self.to_boxed(),
        }
    }
}
