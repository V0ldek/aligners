#[cfg(test)]
mod tests {
    use crate::alignment::{self, Alignment};
    use crate::bytes::AlignedBytes;
    use crate::slice::AlignedSlice;

    #[test]
    fn is_block_aligned_when_created_from_unaligned_slice() {
        let alignment_size = alignment::SimdBlock::size();
        let slice: &[u8] = &std::iter::repeat(42)
            .take(alignment_size)
            .collect::<Vec<_>>();
        let misalignment = slice.as_ptr() as usize % alignment_size;
        let source = if misalignment > 0 { slice } else { &slice[1..] };
        let bytes = AlignedBytes::<alignment::SimdBlock>::from(source);

        assert_eq!(bytes.as_ptr() as usize % alignment_size, 0);
    }

    #[test]
    fn contains_same_bytes_when_block_aligned_from_slice() {
        let slice = (0..=47).collect::<Vec<u8>>();
        let bytes = AlignedBytes::<alignment::SimdBlock>::from(&slice);

        assert_eq!(bytes, slice);
    }

    #[test]
    fn creates_empty_bytes_when_given_zero_length_for_block() {
        let bytes = AlignedBytes::<alignment::SimdBlock>::new_zeroed(0);

        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn block_alignment_from_page_alignment_is_identity() {
        let slice = (0..=47).collect::<Vec<u8>>();
        let page_aligned: &AlignedSlice<alignment::Page> =
            &AlignedBytes::<alignment::Page>::from(&slice);
        let block_aligned: &AlignedSlice<alignment::SimdBlock> = page_aligned.relax_alignment();

        assert_eq!(block_aligned, slice);
    }
}
