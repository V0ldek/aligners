use crate::alignment::Alignment;
use crate::slice::AlignedSlice;

impl<A: Alignment> PartialEq for AlignedSlice<A> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let other_slice: &[u8] = other;
        self.eq(other_slice)
    }
}

impl<A: Alignment> Eq for AlignedSlice<A> {}

impl<A: Alignment> PartialEq<&AlignedSlice<A>> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &&AlignedSlice<A>) -> bool {
        let slice: &[u8] = self;
        let other_slice: &[u8] = other;
        slice.eq(other_slice)
    }
}

impl<A: Alignment> PartialEq<Vec<u8>> for &AlignedSlice<A> {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        other.eq(self)
    }
}

impl<A: Alignment> PartialEq<[u8]> for AlignedSlice<A> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        let slice: &[u8] = self;
        slice.eq(other)
    }
}

impl<A: Alignment> PartialEq<AlignedSlice<A>> for [u8] {
    #[inline]
    fn eq(&self, other: &AlignedSlice<A>) -> bool {
        other.eq(self)
    }
}

impl<A: Alignment, const N: usize> PartialEq<[u8; N]> for AlignedSlice<A> {
    #[inline]
    fn eq(&self, other: &[u8; N]) -> bool {
        let slice: &[u8] = self;
        slice.eq(other)
    }
}

impl<A: Alignment, const N: usize> PartialEq<AlignedSlice<A>> for [u8; N] {
    #[inline]
    fn eq(&self, other: &AlignedSlice<A>) -> bool {
        other.eq(self)
    }
}

impl<A: Alignment> PartialOrd for AlignedSlice<A> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let slice: &[u8] = self;
        let other_slice: &[u8] = other;

        slice.partial_cmp(other_slice)
    }
}

impl<A: Alignment> Ord for AlignedSlice<A> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let slice: &[u8] = self;
        let other_slice: &[u8] = other;

        slice.cmp(other_slice)
    }
}
