use crate::alignment::Alignment;
use crate::bytes::AlignedBytes;
use crate::iterators::AlignedBlockIterator;
use std::borrow::{Borrow, BorrowMut};
use std::mem;
use std::ops::{Deref, DerefMut};

mod cmp;
#[doc(inline)]
#[allow(unreachable_pub)] // False positive, this is reachable and required.
pub use cmp::*;

/// Slice of bytes aligned to a boundary represented by `A`.
///
/// # Guarantees
///
/// It is guaranteed that the bytes allocated in this structure are aligned
/// to an [`A::size()`](`Alignment::size`) byte boundary.
///
/// # Safety
///
/// Because the used `repr` is [`transparent`](https://doc.rust-lang.org/reference/type-layout.html#the-transparent-representation),
/// it is possible to directly [`std::mem::transmute`] a [`[u8]`] into an [`AlignedSlice<A>`] (and vice-versa).
/// This is only safe if the original slice is already aligned to [`A::size()`](`Alignment::size`).
/// Using unaligned bytes in a place that requires alignment is usually undefined behaviour.
#[repr(transparent)]
pub struct AlignedSlice<A: Alignment> {
    phantom: std::marker::PhantomData<A>,
    bytes: [u8],
}

impl<A: Alignment> AlignedSlice<A> {
    /// Returns the slice offset by `count` aligned blocks.
    /// This is equivalent to skipping `count * A::size()` bytes.
    ///
    /// # Panics
    /// If there are less than `count` blocks until end of the slice.
    #[must_use]
    #[inline]
    pub fn offset(&self, count: isize) -> &Self {
        let offset_in_bytes = A::size() * (count as usize);

        if self.bytes.len() < offset_in_bytes {
            panic!(
                "offset {count} out of range for AlignedSlice of {} aligned blocks",
                self.bytes.len() / A::size()
            )
        }

        // SAFETY:
        // - repr(transparent) + the offset_in_bytes is guaranteed to retain alignment,
        // since it is calculated above as a multiple of A::size() and the slice was aligned at the beginning.
        unsafe { std::mem::transmute(&self[offset_in_bytes..]) }
    }

    /// Return the size of the alignment in bytes.
    ///
    /// ## Note
    /// This does not reflect the actual maximal alignment,
    /// only the guarantee provided by `A`, which may be lower than
    /// the actual alignment.
    #[must_use]
    #[inline(always)]
    pub fn alignment_size(&self) -> usize {
        A::size()
    }

    /// Return an iterator over consecutive aligned blocks of the slice.
    #[must_use]
    #[inline]
    pub fn iter_blocks(&self) -> AlignedBlockIterator<A> {
        AlignedBlockIterator::new(self)
    }

    /// Relax the alignment to a smaller one.
    ///
    /// # Panics
    /// If `B::size()` > `A::size()`.
    #[must_use]
    #[inline]
    pub fn relax_alignment<B: Alignment>(&self) -> &AlignedSlice<B> {
        if A::size() < B::size() {
            panic!("target alignment is larger than source alignment, the 'relax_alignment' conversion is not valid")
        }

        // SAFETY:
        // Since all alignments are multiples of two, A::size() >= B::size() => A::size() % B::size() == 0.
        // The precedent condition is asserted above.
        unsafe { mem::transmute(self) }
    }
}

impl<A: Alignment> AsRef<AlignedSlice<A>> for AlignedBytes<A> {
    #[inline(always)]
    fn as_ref(&self) -> &AlignedSlice<A> {
        self
    }
}

impl<A: Alignment> AsMut<AlignedSlice<A>> for AlignedBytes<A> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut AlignedSlice<A> {
        self
    }
}

impl<A: Alignment> AsRef<[u8]> for AlignedSlice<A> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl<A: Alignment> AsMut<[u8]> for AlignedSlice<A> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [u8] {
        self
    }
}

impl<A: Alignment> Borrow<AlignedSlice<A>> for AlignedBytes<A> {
    #[inline(always)]
    fn borrow(&self) -> &AlignedSlice<A> {
        self
    }
}

impl<A: Alignment> BorrowMut<AlignedSlice<A>> for AlignedBytes<A> {
    #[inline(always)]
    fn borrow_mut(&mut self) -> &mut AlignedSlice<A> {
        self
    }
}

impl<A: Alignment> Clone for AlignedBytes<A> {
    #[inline]
    fn clone(&self) -> AlignedBytes<A> {
        let slice: &AlignedSlice<A> = self;
        slice.into()
    }

    #[inline]
    fn clone_from(&mut self, other: &AlignedBytes<A>) {
        let source: &AlignedSlice<A> = other;
        let target: &mut AlignedSlice<A> = self;

        target.clone_from_slice(source);
    }
}

impl<A: Alignment> Deref for AlignedBytes<A> {
    type Target = AlignedSlice<A>;

    #[inline]
    fn deref(&self) -> &AlignedSlice<A> {
        // SAFETY:
        // - the `data` pointer is a `NonNull` pointer to a single allocated object of size exactly `self.size`
        //   and is properly aligned since proper alignment for `u8` is 1;
        unsafe {
            let slice = std::slice::from_raw_parts(self.as_ptr(), self.len());
            std::mem::transmute(slice)
        }
    }
}

impl<A: Alignment> DerefMut for AlignedBytes<A> {
    #[inline]
    fn deref_mut<'a>(&'a mut self) -> &'a mut AlignedSlice<A> {
        // SAFETY:
        // 1. All the conditions for from_raw_parts_mut:
        //   > `data` must be valid for reads for `len * mem::size_of::<T>()` many bytes, and it must be properly aligned.
        //   - `T` is `u8` and we allocated `len` bytes in AlignedBytes' ctors. Proper alignment for `u8` is 1, trivially satisfied.
        //   > `data` must point to `len` consecutive properly initialized values of type `T`.
        //   - This is upheld by AlignedBytes' constructors.
        //   > The memory referenced by the returned slice must not be accessed through any other pointer
        //   > (not derived from the return value) for the duration of lifetime `'a`. Both read and write accesses are forbidden.
        //   - This follows from the explicit lifetimes given. To call deref_mut we mutably borrow the AlignedBytes for 'a,
        //     and return a mutable borrow of a slice valid for 'a. Because of borrow rules, this can be the only valid mutable
        //     reference to the underlying bytes.
        //   > The total size len * mem::size_of::<T>() of the slice must be no larger than isize::MAX. See the safety documentation of pointer::offset.
        //   - This is asserted in AlignedBytes' ctor.
        // 2. transmute is safe because of AlignedSlice's repr(transparent).
        unsafe {
            let slice: &'a mut [u8] = std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len());
            std::mem::transmute(slice)
        }
    }
}

impl<A: Alignment> Deref for AlignedSlice<A> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        // SAFETY:
        // Using AlignedSlice's repr(transparent).
        unsafe { std::mem::transmute(self) }
    }
}

impl<A: Alignment> DerefMut for AlignedSlice<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        // SAFETY:
        // Using AlignedSlice's repr(transparent).
        unsafe { std::mem::transmute(self) }
    }
}

impl<A: Alignment> std::fmt::Debug for AlignedSlice<A> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let deref: &[u8] = self;
        std::fmt::Debug::fmt(deref, f)
    }
}

impl<A: Alignment> Default for &AlignedSlice<A> {
    #[inline]
    fn default() -> Self {
        let default_bytes: AlignedBytes<A> = Default::default();
        // SAFETY:
        // Using AlignedSlice's repr(transparent).
        unsafe {
            let slice = std::slice::from_raw_parts(default_bytes.as_ptr(), 0);
            std::mem::transmute(slice)
        }
    }
}

impl<A: Alignment> Default for &mut AlignedSlice<A> {
    #[inline]
    fn default() -> Self {
        let mut default_bytes: AlignedBytes<A> = Default::default();
        // SAFETY:
        // Using AlignedSlice's repr(transparent).
        unsafe {
            let slice = std::slice::from_raw_parts_mut(default_bytes.as_mut_ptr(), 0);
            std::mem::transmute(slice)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test::assert_aligned;
    use crate::{alignment, AlignedBytes, AlignedSlice};

    #[test]
    fn empty_slice_is_aligned() {
        let empty: &AlignedSlice<alignment::Eight> = Default::default();
        assert_aligned(empty.as_ptr(), 8);
    }

    #[test]
    fn empty_mut_slice_is_aligned() {
        let empty: &mut AlignedSlice<alignment::Eight> = Default::default();
        assert_aligned(empty.as_ptr(), 8);
    }

    #[test]
    fn alignment_size_equal_to_alignment_type() {
        let bytes: AlignedBytes<alignment::TwoTo<7>> = AlignedBytes::new_zeroed(1024);
        let slice: &AlignedSlice<alignment::TwoTo<7>> = &bytes;

        assert_eq!(128, slice.alignment_size());
    }
}
