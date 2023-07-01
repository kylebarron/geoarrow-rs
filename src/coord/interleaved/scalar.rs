use arrow2::buffer::Buffer;

pub struct InterleavedCoord<'a> {
    coords: &'a Buffer<f64>,
    i: usize,
}

impl From<InterleavedCoord<'_>> for geo::Coord {
    fn from(value: InterleavedCoord) -> Self {
        geo::Coord {
            x: *value.coords.get(value.i * 2).unwrap(),
            y: *value.coords.get(value.i * 2 + 1).unwrap(),
        }
    }
}
