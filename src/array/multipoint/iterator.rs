use crate::array::MultiPointArray;
use crate::scalar::MultiPoint;
use crate::GeometryArrayTrait;
use arrow2::bitmap::utils::{BitmapIter, ZipValidity};
use arrow2::trusted_len::TrustedLen;
use arrow2::types::Offset;

/// Iterator of values of a [`MultiPointArray`]
#[derive(Clone, Debug)]
pub struct MultiPointArrayValuesIter<'a, O: Offset> {
    array: &'a MultiPointArray<O>,
    index: usize,
    end: usize,
}

impl<'a, O: Offset> MultiPointArrayValuesIter<'a, O> {
    #[inline]
    pub fn new(array: &'a MultiPointArray<O>) -> Self {
        Self {
            array,
            index: 0,
            end: array.len(),
        }
    }
}

impl<'a, O: Offset> Iterator for MultiPointArrayValuesIter<'a, O> {
    type Item = MultiPoint<'a, O>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.end {
            return None;
        }
        let old = self.index;
        self.index += 1;
        Some(self.array.value(old))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.end - self.index, Some(self.end - self.index))
    }
}

unsafe impl<'a, O: Offset> TrustedLen for MultiPointArrayValuesIter<'a, O> {}

impl<'a, O: Offset> DoubleEndedIterator for MultiPointArrayValuesIter<'a, O> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index == self.end {
            None
        } else {
            self.end -= 1;
            Some(self.array.value(self.end))
        }
    }
}

impl<'a, O: Offset> IntoIterator for &'a MultiPointArray<O> {
    type Item = Option<MultiPoint<'a, O>>;
    type IntoIter =
        ZipValidity<MultiPoint<'a, O>, MultiPointArrayValuesIter<'a, O>, BitmapIter<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, O: Offset> MultiPointArray<O> {
    /// Returns an iterator of `Option<Point>`
    pub fn iter(
        &'a self,
    ) -> ZipValidity<MultiPoint<'a, O>, MultiPointArrayValuesIter<'a, O>, BitmapIter<'a>> {
        ZipValidity::new_with_validity(MultiPointArrayValuesIter::new(self), self.validity())
    }

    /// Returns an iterator of `Point`
    pub fn values_iter(&'a self) -> MultiPointArrayValuesIter<'a, O> {
        MultiPointArrayValuesIter::new(self)
    }
}
