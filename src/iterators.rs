use crate::alignment::Alignment;
use crate::slice::AlignedSlice;
use std::iter::FusedIterator;
use std::mem;
use std::ops::Deref;

/// Thin wrapper that represents an [`AlignedSlice`] of size at most the alignment size.
///
/// # Safety
/// Similarly to [`AlignedSlice`], the used `repr` is [`transparent`](https://doc.rust-lang.org/reference/type-layout.html#the-transparent-representation),
/// and it is possible to directly [`std::mem::transmute`] an [`AlignedSlice<A>`] into an [`AlignedBlock<A>`] (and vice-versa).
/// This is only safe if the size of the the slice is at most [`A::size()`](`Alignment::size`).
#[repr(transparent)]
pub struct AlignedBlock<A: Alignment> {
    slice: AlignedSlice<A>,
}

/// Iterator over [`AlignedBlocks`](`AlignedBlock`) of a given aligned bytes span.
pub struct AlignedBlockIterator<'a, A: Alignment> {
    bytes: &'a AlignedSlice<A>,
}

impl<'a, A: Alignment> AlignedBlockIterator<'a, A> {
    #[must_use]
    pub(crate) fn new(slice: &'a AlignedSlice<A>) -> Self {
        Self { bytes: slice }
    }
}

impl<A: Alignment> Deref for AlignedBlock<A> {
    type Target = AlignedSlice<A>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // repr(transparent) and the requirements for AlignedSlice are
        // a subset of those of AlignedBlock
        unsafe { mem::transmute(self) }
    }
}

impl<A: Alignment> AlignedBlock<A> {
    /// Returns the length of the block. Guaranteed to be at most [`A::size()`](`Alignment::size`).
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    /// Returns whether the block is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }
}

impl<'a, A: Alignment> Iterator for AlignedBlockIterator<'a, A> {
    type Item = &'a AlignedBlock<A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }

        if self.bytes.len() < A::size() {
            // SAFETY:
            // `self.bytes` is aligned to `A` and we checked its size does not exceed `A::size()`.
            let chunk = unsafe { mem::transmute(self.bytes) };
            self.bytes = Default::default();
            return Some(chunk);
        }

        // SAFETY:
        // `self.bytes` is aligned to `A` and we take exactly one block of size `A::size()`.
        let chunk = unsafe { mem::transmute(&self.bytes[..A::size()]) };
        self.bytes = self.bytes.offset(1);

        Some(chunk)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.bytes.len() + A::size() - 1) / A::size();
        (size, Some(size))
    }
}

impl<A: Alignment> ExactSizeIterator for AlignedBlockIterator<'_, A> {}

impl<A: Alignment> FusedIterator for AlignedBlockIterator<'_, A> {}
