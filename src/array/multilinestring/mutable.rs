use crate::array::{
    MultiLineStringArray, MutableCoordBuffer, MutableInterleavedCoordBuffer, MutablePolygonArray,
    WKBArray,
};
use crate::error::GeoArrowError;
use crate::geo_traits::{LineStringTrait, MultiLineStringTrait};
use crate::io::native::wkb::maybe_multi_line_string::WKBMaybeMultiLineString;
use crate::scalar::WKB;
use crate::GeometryArrayTrait;
use arrow2::array::ListArray;
use arrow2::bitmap::{Bitmap, MutableBitmap};
use arrow2::offset::{Offsets, OffsetsBuffer};
use arrow2::types::Offset;

#[derive(Debug, Clone)]
pub struct MutableMultiLineStringArray<O: Offset> {
    coords: MutableCoordBuffer,

    /// Offsets into the ring array where each geometry starts
    geom_offsets: Offsets<O>,

    /// Offsets into the coordinate array where each ring starts
    ring_offsets: Offsets<O>,

    /// Validity is only defined at the geometry level
    validity: Option<MutableBitmap>,
}

pub type MultiLineStringInner<O> = (
    MutableCoordBuffer,
    Offsets<O>,
    Offsets<O>,
    Option<MutableBitmap>,
);

impl<O: Offset> MutableMultiLineStringArray<O> {
    /// Creates a new empty [`MutableMultiLineStringArray`].
    pub fn new() -> Self {
        MutablePolygonArray::new().into()
    }

    /// Creates a new [`MutableMultiLineStringArray`] with a capacity.
    pub fn with_capacities(
        coord_capacity: usize,
        geom_capacity: usize,
        ring_capacity: usize,
    ) -> Self {
        MutablePolygonArray::with_capacities(coord_capacity, geom_capacity, ring_capacity).into()
    }

    /// The canonical method to create a [`MutableMultiLineStringArray`] out of its internal components.
    /// # Implementation
    /// This function is `O(1)`.
    ///
    /// # Errors
    /// This function errors iff:
    /// * The validity is not `None` and its length is different from `values`'s length
    pub fn try_new(
        coords: MutableCoordBuffer,
        geom_offsets: Offsets<O>,
        ring_offsets: Offsets<O>,
        validity: Option<MutableBitmap>,
    ) -> Result<Self, GeoArrowError> {
        MutablePolygonArray::try_new(coords, geom_offsets, ring_offsets, validity)
            .map(|result| result.into())
    }

    /// Extract the low-level APIs from the [`MutableMultiLineStringArray`].
    pub fn into_inner(self) -> MultiLineStringInner<O> {
        (
            self.coords,
            self.geom_offsets,
            self.ring_offsets,
            self.validity,
        )
    }

    pub fn into_arrow(self) -> ListArray<O> {
        let arr: MultiLineStringArray<O> = self.into();
        arr.into_arrow()
    }
}

impl<O: Offset> Default for MutableMultiLineStringArray<O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O: Offset> From<MutableMultiLineStringArray<O>> for MultiLineStringArray<O> {
    fn from(other: MutableMultiLineStringArray<O>) -> Self {
        let validity = other.validity.and_then(|x| {
            let bitmap: Bitmap = x.into();
            if bitmap.unset_bits() == 0 {
                None
            } else {
                Some(bitmap)
            }
        });

        let geom_offsets: OffsetsBuffer<O> = other.geom_offsets.into();
        let ring_offsets: OffsetsBuffer<O> = other.ring_offsets.into();

        Self::new(other.coords.into(), geom_offsets, ring_offsets, validity)
    }
}

fn first_pass<'a, O: Offset>(
    geoms: impl Iterator<Item = Option<impl MultiLineStringTrait<'a> + 'a>>,
    geoms_length: usize,
) -> (Offsets<O>, Offsets<O>, Option<MutableBitmap>) {
    let mut validity = MutableBitmap::with_capacity(geoms_length);

    // Offset into ring indexes for each geometry
    let mut geom_offsets = Offsets::<O>::with_capacity(geoms_length);

    // Offset into coordinates for each ring
    // This capacity will only be enough in the case where each geometry has only a single
    // linestring
    let mut ring_offsets = Offsets::<O>::with_capacity(geoms_length);

    for maybe_multi_line_string in geoms {
        if let Some(multi_line_string) = maybe_multi_line_string {
            validity.push(true);

            // Total number of linestrings in this multilinestring
            let num_line_strings = multi_line_string.num_lines();
            geom_offsets.try_push_usize(num_line_strings).unwrap();

            // Number of coords for each ring
            for line_string_idx in 0..num_line_strings {
                let line_string = multi_line_string.line(line_string_idx).unwrap();
                ring_offsets
                    .try_push_usize(line_string.num_coords())
                    .unwrap();
            }
        } else {
            validity.push(false);
            geom_offsets.try_push_usize(0).unwrap();
        }
    }

    let validity = if validity.unset_bits() == 0 {
        None
    } else {
        Some(validity)
    };

    (geom_offsets, ring_offsets, validity)
}

fn second_pass<'a, O: Offset>(
    geoms: impl Iterator<Item = Option<impl MultiLineStringTrait<'a, T = f64> + 'a>>,
    geom_offsets: Offsets<O>,
    ring_offsets: Offsets<O>,
    validity: Option<MutableBitmap>,
) -> MutableMultiLineStringArray<O> {
    let mut coord_buffer =
        MutableInterleavedCoordBuffer::with_capacity(ring_offsets.last().to_usize());

    for multi_line_string in geoms.into_iter().flatten() {
        for line_string_idx in 0..multi_line_string.num_lines() {
            let line_string = multi_line_string.line(line_string_idx).unwrap();
            for coord_idx in 0..line_string.num_coords() {
                let coord = line_string.coord(coord_idx).unwrap();
                coord_buffer.push_coord(coord);
            }
        }
    }

    MutableMultiLineStringArray {
        coords: MutableCoordBuffer::Interleaved(coord_buffer),
        geom_offsets,
        ring_offsets,
        validity,
    }
}

impl<O: Offset> From<Vec<geo::MultiLineString>> for MutableMultiLineStringArray<O> {
    fn from(geoms: Vec<geo::MultiLineString>) -> Self {
        let (geom_offsets, ring_offsets, validity) =
            first_pass::<O>(geoms.iter().map(Some), geoms.len());
        second_pass(
            geoms.into_iter().map(Some),
            geom_offsets,
            ring_offsets,
            validity,
        )
    }
}

impl<O: Offset> From<Vec<Option<geo::MultiLineString>>> for MutableMultiLineStringArray<O> {
    fn from(geoms: Vec<Option<geo::MultiLineString>>) -> Self {
        let (geom_offsets, ring_offsets, validity) =
            first_pass::<O>(geoms.iter().map(|x| x.as_ref()), geoms.len());
        second_pass(geoms.into_iter(), geom_offsets, ring_offsets, validity)
    }
}

impl<O: Offset> From<bumpalo::collections::Vec<'_, geo::MultiLineString>>
    for MutableMultiLineStringArray<O>
{
    fn from(geoms: bumpalo::collections::Vec<'_, geo::MultiLineString>) -> Self {
        let (geom_offsets, ring_offsets, validity) =
            first_pass::<O>(geoms.iter().map(Some), geoms.len());
        second_pass(
            geoms.into_iter().map(Some),
            geom_offsets,
            ring_offsets,
            validity,
        )
    }
}

impl<O: Offset> From<bumpalo::collections::Vec<'_, Option<geo::MultiLineString>>>
    for MutableMultiLineStringArray<O>
{
    fn from(geoms: bumpalo::collections::Vec<'_, Option<geo::MultiLineString>>) -> Self {
        let (geom_offsets, ring_offsets, validity) =
            first_pass::<O>(geoms.iter().map(|x| x.as_ref()), geoms.len());
        second_pass(geoms.into_iter(), geom_offsets, ring_offsets, validity)
    }
}

impl<O: Offset> TryFrom<WKBArray<O>> for MutableMultiLineStringArray<O> {
    type Error = GeoArrowError;

    fn try_from(value: WKBArray<O>) -> Result<Self, Self::Error> {
        let wkb_objects: Vec<Option<WKB<'_, O>>> = value.iter().collect();
        let wkb_objects2: Vec<Option<WKBMaybeMultiLineString>> = wkb_objects
            .iter()
            .map(|maybe_wkb| {
                maybe_wkb
                    .as_ref()
                    .map(|wkb| wkb.to_wkb_object().to_maybe_multi_line_string())
            })
            .collect();
        let (geom_offsets, ring_offsets, validity) =
            first_pass::<O>(wkb_objects2.iter().map(|item| item.as_ref()), value.len());
        Ok(second_pass(
            wkb_objects2.iter().map(|item| item.as_ref()),
            geom_offsets,
            ring_offsets,
            validity,
        ))
    }
}

/// Polygon and MultiLineString have the same layout, so enable conversions between the two to
/// change the semantic type
impl<O: Offset> From<MutableMultiLineStringArray<O>> for MutablePolygonArray<O> {
    fn from(value: MutableMultiLineStringArray<O>) -> Self {
        Self::try_new(
            value.coords,
            value.geom_offsets,
            value.ring_offsets,
            value.validity,
        )
        .unwrap()
    }
}
