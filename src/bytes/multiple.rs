use crate::alignment::{self, Alignment};
use crate::iterators::AlignedBlock;
use crate::slice::AlignedSlice;
use crate::AlignedBytes;
use std::mem;

impl<A: Alignment> AlignedBlock<alignment::Twice<A>> {
    /// Split the block into two blocks aligned to [`alignment::SimdBlock`].
    #[must_use]
    #[inline]
    pub fn halves(&self) -> (&AlignedBlock<A>, &AlignedBlock<A>) {
        let slice: &AlignedSlice<alignment::Twice<A>> = self;
        let empty_aligned = AlignedBytes::<A>::default();

        let (slice1, slice2) = if slice.len() <= A::size() {
            (slice as &[u8], &empty_aligned as &[u8])
        } else {
            (&slice[..A::size()], &slice[A::size()..])
        };

        // SAFETY:
        // AlignedBlock is a repr(transparent) over AlignedSlice, which is repr(transparent) over [u8].
        // Both transmutes are safe. The alignment guarantee is obviously upheld, since slice is aligned
        // to twice `A` and the bytes are contiguous.
        unsafe {
            let block1 = mem::transmute(slice1);
            let block2 = mem::transmute(slice2);

            (block1, block2)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        alignment::{Twice, TwoTo},
        test::assert_aligned,
        AlignedBlockIterator, AlignedBytes,
    };

    #[test]
    fn halves() {
        let bytes: AlignedBytes<Twice<TwoTo<2>>> = AlignedBytes::from([1, 2, 3, 4, 5, 6, 7, 8]);
        let expected = [([1, 2], [3, 4]), ([5, 6], [7, 8])];
        let iter: AlignedBlockIterator<Twice<TwoTo<1>>> = bytes.relax_alignment().iter_blocks();

        for (block, ex) in iter.zip(expected) {
            let (block1, block2) = block.halves();
            let slice1: &[u8] = block1;
            let slice2: &[u8] = block2;

            assert_eq!(slice1, ex.0);
            assert_eq!(slice2, ex.1);
            assert_aligned(block1.as_ptr(), 2);
            assert_aligned(block2.as_ptr(), 2);
        }
    }

    #[test]
    fn halves_not_full() {
        let bytes: AlignedBytes<Twice<TwoTo<2>>> = AlignedBytes::from([1, 2, 3, 4, 5, 6]);
        let expected: [(&[u8], &[u8]); 2] = [(&[1, 2], &[3, 4]), (&[5, 6], &[] as &[u8])];
        let iter: AlignedBlockIterator<Twice<TwoTo<1>>> = bytes.relax_alignment().iter_blocks();

        for (block, ex) in iter.zip(expected) {
            let (block1, block2) = block.halves();
            let slice1: &[u8] = block1;
            let slice2: &[u8] = block2;

            assert_eq!(slice1, ex.0);
            assert_eq!(slice2, ex.1);
            assert_aligned(block1.as_ptr(), 2);
            assert_aligned(block2.as_ptr(), 2);
        }
    }
}
