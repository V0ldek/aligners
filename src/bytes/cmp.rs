use crate::alignment::Alignment;
use crate::bytes::AlignedBytes;
use crate::slice::AlignedSlice;

impl<A: Alignment> PartialEq for AlignedBytes<A> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let slice: &AlignedSlice<A> = self;
        let other_slice: &AlignedSlice<A> = other;

        slice.eq(other_slice)
    }
}

impl<A: Alignment> Eq for AlignedBytes<A> {}

impl<A: Alignment> PartialEq<AlignedBytes<A>> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &AlignedBytes<A>) -> bool {
        other.eq(self)
    }
}

impl<A: Alignment> PartialEq<Vec<u8>> for AlignedBytes<A> {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        let slice: &AlignedSlice<A> = self;
        let other_slice: &[u8] = other;

        slice.eq(other_slice)
    }
}

impl<A: Alignment> PartialEq<AlignedBytes<A>> for [u8] {
    #[inline]
    fn eq(&self, other: &AlignedBytes<A>) -> bool {
        other.eq(self)
    }
}

impl<A: Alignment> PartialEq<[u8]> for AlignedBytes<A> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        let slice: &AlignedSlice<A> = self;

        slice.eq(other)
    }
}

impl<A: Alignment, const N: usize> PartialEq<AlignedBytes<A>> for [u8; N] {
    #[inline]
    fn eq(&self, other: &AlignedBytes<A>) -> bool {
        other.eq(self)
    }
}

impl<A: Alignment, const N: usize> PartialEq<[u8; N]> for AlignedBytes<A> {
    #[inline]
    fn eq(&self, other: &[u8; N]) -> bool {
        let slice: &AlignedSlice<A> = self;

        slice.eq(other)
    }
}
impl<A: Alignment> PartialOrd for AlignedBytes<A> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let slice: &AlignedSlice<A> = self;
        let other_slice: &AlignedSlice<A> = other;

        slice.partial_cmp(other_slice)
    }
}

impl<A: Alignment> Ord for AlignedBytes<A> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let slice: &AlignedSlice<A> = self;
        let other_slice: &AlignedSlice<A> = other;

        slice.cmp(other_slice)
    }
}

impl<A: Alignment> std::hash::Hash for AlignedBytes<A> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.as_ptr(), state)
    }
}
