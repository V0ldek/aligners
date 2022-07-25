use crate::alignment::Alignment;
use cfg_if::cfg_if;
use std::ptr::NonNull;

mod cmp;
mod multiple;

#[doc(inline)]
#[allow(unreachable_pub)] // False positive, this is reachable and required.
pub use cmp::*;

cfg_if! {
    if #[cfg(feature = "simd")] {
        mod simd;

        #[doc(inline)]
        #[allow(unreachable_pub)] // False positive, this is reachable and required.
        pub use simd::*;
    }
}

/// Bytes aligned to a boundary represented by `A`.
///
/// This type owns the bytes. They are allocated when the struct is created and deallocated
/// on drop.
///
/// # Guarantees
///
/// It is guaranteed that the bytes allocated in this structure are aligned
/// to an [`A::size()`](`Alignment::size`) byte boundary. Therefore the integer representation
/// of the pointer obtained by the [`as_ptr`](`std::slice::[]::as_ptr`) (or
/// [`as_mut_ptr`](`std::slice::[]::as_mut_ptr`)) will be divisible by
/// [`A::size()`](`Alignment::size`).
pub struct AlignedBytes<A: Alignment> {
    bytes_ptr: std::ptr::NonNull<u8>,
    size: usize,
    phantom: std::marker::PhantomData<A>,
}

impl<A: Alignment> AlignedBytes<A> {
    fn get_layout(size: usize) -> std::alloc::Layout {
        std::alloc::Layout::from_size_align(size, A::size()).unwrap()
    }

    /// Create new, possibly uninitialized, block of bytes of given length.
    ///
    /// # Safety
    /// The memory used by the bytes might not be initialized, which makes reading
    /// from them undefined behaviour (yes, [even for `u8` reading uninitialized bytes is UB](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#initialization-invariant)).
    /// To use the bytes you must first initialize them manually.
    ///
    /// If you want zeroed bytes, use [`AlignedBytes::new_zeroed`] instead.
    /// If you want to initialize the bytes with custom logic, use [`AlignedBytes::new_initialize`] instead.
    /// If you want to align existing bytes, use the [`From`] trait implementations.
    #[inline]
    #[must_use]
    pub unsafe fn new(size: usize) -> Self {
        Self::new_impl(size)
    }

    // Extracted so that this fn isn't all in an `unsafe` context by default.
    fn new_impl(size: usize) -> Self {
        if size > (isize::MAX as usize) {
            panic!("cannot allocate more than `isize::MAX` bytes, attempted to allocate {size}");
        }

        let layout = Self::get_layout(size);

        // SAFETY:
        // Layout is guaranteed to be of non-zero size at this point.
        let raw_ptr = unsafe { std::alloc::alloc(layout) };
        let ptr = std::ptr::NonNull::new(raw_ptr).unwrap();

        Self {
            bytes_ptr: ptr,
            size,
            phantom: std::marker::PhantomData {},
        }
    }

    /// Create new block of bytes of given length and initialize each byte to a function
    /// of its index.
    ///
    /// # Examples
    /// ```rust
    /// # use aligners::{Aligned, AlignedBytes, alignment::{self, Alignment}};
    /// let aligned = AlignedBytes::<alignment::Page>::new_initialize(8, |i| { (i % 2) as u8 });
    /// let ptr = aligned.as_ptr();
    ///
    /// assert_eq!(ptr as usize % alignment::Page::size(), 0);
    /// assert_eq!(aligned, [0, 1, 0, 1, 0, 1, 0, 1]);
    /// ```
    #[inline]
    pub fn new_initialize<F>(size: usize, mut f: F) -> Self
    where
        F: FnMut(usize) -> u8,
    {
        // SAFETY:
        // All bytes are initialized right after.
        let mut block = unsafe { Self::new(size) };

        for i in 0..block.size {
            block[i] = f(i);
        }

        block
    }

    /// Create new block of bytes of given length and initialize
    /// to all-zeroes.
    /// # Panics
    /// If allocating memory fails, i.e. internal call to [`std::alloc::alloc_zeroed`] panics.
    #[must_use]
    #[inline]
    pub fn new_zeroed(size: usize) -> Self {
        if size == 0 {
            return Self::default();
        }

        let layout = Self::get_layout(size);

        // SAFETY:
        // Layout is guaranteed to be of non-zero size at this point.
        let raw_ptr = unsafe { std::alloc::alloc_zeroed(layout) };
        let ptr = std::ptr::NonNull::new(raw_ptr).unwrap();

        Self {
            bytes_ptr: ptr,
            size,
            phantom: std::marker::PhantomData {},
        }
    }

    /// Create a new block of bytes by copying the given bytes
    /// and padding them with zeroes, so that the total size is
    /// divisible by the alignment size.
    ///
    /// This is primarily useful to guarantee that [`AlignedBlockIterator`](crate::iterators::AlignedBlockIterator)
    /// returns full blocks of size exactly equal to the alignment,
    /// as otherwise the final block can be potentially smaller.
    #[must_use]
    #[inline]
    pub fn new_padded(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Self::default();
        }

        let size = bytes.len();
        let padding = if size % A::size() == 0 {
            0
        } else {
            A::size() - size % A::size()
        };
        let padded_size = size + padding;

        let mut aligned = Self::new_zeroed(padded_size);
        aligned[..size].copy_from_slice(bytes);

        aligned
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

    /// Return the length of the byte array.
    #[must_use]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.size
    }

    /// Return whether the length of this byte array is zero.
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Get the pointer to the beginning of the aligned bytes array.
    #[must_use]
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.bytes_ptr.as_ptr()
    }

    /// Get a `mut` pointer to the beginning of the aligned bytes array.
    #[must_use]
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.bytes_ptr.as_ptr()
    }
}

impl<A: Alignment> Drop for AlignedBytes<A> {
    #[inline]
    fn drop(&mut self) {
        use std::alloc::dealloc;

        if self.size == 0 {
            return;
        }

        let layout = Self::get_layout(self.size);

        // SAFETY:
        // `ptr` is allocated in `new_internal` and
        // layout is constructed using the same function and will be the same.
        // This relies on `A::size()` being constant and self.size not being mutated ever.
        unsafe { dealloc(self.bytes_ptr.as_ptr(), layout) }
    }
}

impl<T: AsRef<[u8]>, A: Alignment> From<T> for AlignedBytes<A> {
    #[inline]
    fn from(s: T) -> Self {
        let slice = s.as_ref();
        let bytes;

        // SAFETY:
        // Uninitialized `new` is safe since we immediately initialize the bytes with `s`, and `copy` is safe because:
        // - src is valid for reading `slice.len()` bytes.
        // - dst is valid for writing `slice.len()` bytes, since `Self::new` allocates that much
        //   bytes, but aligned.
        // - Both pointers are properly aligned, since proper alignment for `u8` is 1.
        unsafe {
            bytes = Self::new(slice.len());
            std::ptr::copy(slice.as_ptr(), bytes.bytes_ptr.as_ptr(), slice.len())
        };

        bytes
    }
}

impl<A: Alignment> std::fmt::Debug for AlignedBytes<A> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let deref = &**self;
        std::fmt::Debug::fmt(deref, f)
    }
}

impl<A: Alignment> Default for AlignedBytes<A> {
    #[inline]
    fn default() -> Self {
        // SAFETY:
        // A zero-sized allocation can be represented by any pointer that is
        // 1. non-null
        // 2. properly aligned
        // The simplest value that satisfies this is just the alignment value reinterpreted as a pointer.
        // This is the strategy used by the standard library for zero-sized allocations
        // (like a Box::new(()), or of any other ZST), usually employed by calling NonNull::dangling().
        // This is the same implementation (https://doc.rust-lang.org/src/core/ptr/non_null.rs.html#88),
        // but for `A::size()` alignment.
        // The only requirement of new_unchecked is the pointer being not-null, and A::size() must be > 0.
        let bytes_ptr = unsafe {
            // Use strict pointer functions if enabled.
            // See https://github.com/V0ldek/aligners/issues/34
            #[cfg(miri)]
            let raw_ptr = std::ptr::invalid_mut(A::size());
            #[cfg(not(miri))]
            let raw_ptr = A::size() as *mut u8;

            NonNull::new_unchecked(raw_ptr)
        };
        Self {
            bytes_ptr,
            size: 0,
            phantom: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test::assert_aligned;
    use crate::{alignment, AlignedBytes};

    #[test]
    fn empty_bytes_are_aligned() {
        let empty: AlignedBytes<alignment::Eight> = Default::default();
        assert_aligned(empty.as_ptr(), 8);
    }
    #[test]
    fn new_initialize_is_aligned() {
        let bytes: AlignedBytes<alignment::TwoTo<15>> = AlignedBytes::new_initialize(2137, |_| 1);
        assert_aligned(bytes.as_ptr(), 2usize.pow(15));
    }

    #[test]
    fn alignment_size_equal_to_alignment_type() {
        let bytes: AlignedBytes<alignment::TwoTo<7>> = AlignedBytes::new_zeroed(1024);

        assert_eq!(128, bytes.alignment_size());
    }
}
