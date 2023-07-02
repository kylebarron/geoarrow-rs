use arrow2::array::{FixedSizeListArray, PrimitiveArray};
use arrow2::buffer::Buffer;
use arrow2::datatypes::{DataType, Field};

use crate::error::GeoArrowError;
use crate::{GeometryArrayTrait, InterleavedCoord};

/// A an array of XY coordinates stored interleaved in a single buffer.
#[derive(Debug, Clone)]
pub struct InterleavedCoordBuffer {
    coords: Buffer<f64>,
}

impl InterleavedCoordBuffer {
    pub fn new(coords: Buffer<f64>) -> Self {
        Self { coords }
    }

    pub fn values_array(&self) -> PrimitiveArray<f64> {
        PrimitiveArray::new(DataType::Float64, self.coords, None)
    }

    pub fn values_field(&self) -> Field {
        Field::new("xy", DataType::Float64, false)
    }
}

impl<'a> GeometryArrayTrait<'a> for InterleavedCoordBuffer {
    type ArrowArray = FixedSizeListArray;
    type Scalar = InterleavedCoord<'a>;
    type ScalarGeo = geo::Coord;

    fn value(&'a self, i: usize) -> Self::Scalar {
        InterleavedCoord {
            coords: &self.coords,
            i,
        }
    }

    fn logical_type(&self) -> DataType {
        DataType::FixedSizeList(Box::new(self.values_field()), 2)
    }

    fn extension_type(&self) -> DataType {
        panic!("Coordinate arrays do not have an extension name.")
    }

    fn into_arrow(self) -> Self::ArrowArray {
        FixedSizeListArray::new(self.logical_type(), self.values_array().boxed(), None)
    }

    fn len(&self) -> usize {
        self.coords.len() / 2
    }

    fn validity(&self) -> Option<&arrow2::bitmap::Bitmap> {
        panic!("coordinate arrays don't have their own validity arrays")
    }

    fn slice(&self, offset: usize, length: usize) -> Self {
        InterleavedCoordBuffer::new(self.coords.slice(offset * 2, length * 2))
    }

    unsafe fn slice_unchecked(&self, offset: usize, length: usize) -> Self {
        let new_coords = unsafe { self.coords.slice_unchecked(offset * 2, length * 2) };
        InterleavedCoordBuffer { coords: new_coords }
    }

    fn to_boxed(&self) -> Box<Self> {
        Box::new(self.clone())
    }
}

impl From<InterleavedCoordBuffer> for FixedSizeListArray {
    fn from(value: InterleavedCoordBuffer) -> Self {
        value.into_arrow()
    }
}

impl TryFrom<&FixedSizeListArray> for InterleavedCoordBuffer {
    type Error = GeoArrowError;

    fn try_from(value: &FixedSizeListArray) -> Result<Self, Self::Error> {
        if value.size() != 2 {
            return Err(GeoArrowError::General(
                "Expected this FixedSizeListArray to have size 2".to_string(),
            ));
        }

        let coord_array_values = value
            .values()
            .as_any()
            .downcast_ref::<PrimitiveArray<f64>>()
            .unwrap();

        Ok(InterleavedCoordBuffer::new(coord_array_values.values().clone()))
    }
}
