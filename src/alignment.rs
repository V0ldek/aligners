//! Types of possible alignment type arguments for [`AlignedBytes`](`super::AlignedBytes`).
use cfg_if::cfg_if;

/// Trait for all alignment types that provides its size.
///
/// # Safety
/// The `size` returned must satisfy the following conditions:
/// - it is constant between calls, i.e. two calls to `size` for the same alignment *MUST* return the same value;
/// - the value returned is a power of two.
///
/// Violating any of these constraints will cause undefined behaviour when the alignment is used
/// for [`AlignedBytes`](`super::AlignedBytes`).
pub unsafe trait Alignment {
    /// Size of the alignment.
    fn size() -> usize;
}

/// Alignment to $2^N$. All acceptable alignments can be derived
/// from this alignment, for example 64-byte alignment is simply [`TwoTo<6>`].
///
/// # Examples
/// ```rust
/// use aligners::alignment::{self, Alignment};
///
/// assert_eq!(64, alignment::TwoTo::<6>::size());
/// ```
#[derive(Debug)]
pub enum TwoTo<const N: u32> {}

/// Alignment to 1 byte, so no special alignment &ndash; every slice is always one-byte-aligned.
///
/// # Examples
/// ```rust
/// use aligners::alignment::{self, Alignment};
///
/// assert_eq!(1, alignment::One::size());
/// ```
pub type One = TwoTo<0>;

/// Alignment to 2 bytes, same as [`u16`]/[`i16`].
///
/// # Examples
/// ```rust
/// use aligners::alignment::{self, Alignment};
///
/// assert_eq!(2, alignment::Two::size());
/// ```
pub type Two = TwoTo<1>;

/// Alignment to 4 bytes, same as [`u32`]/[`i32`].
///
/// # Examples
/// ```rust
/// use aligners::alignment::{self, Alignment};
///
/// assert_eq!(4, alignment::Four::size());
/// ```
pub type Four = TwoTo<2>;

/// Alignment to 8 bytes, same as [`u64`]/[`i64`].
///
/// # Examples
/// ```rust
/// use aligners::alignment::{self, Alignment};
///
/// assert_eq!(8, alignment::Eight::size());
/// ```
pub type Eight = TwoTo<3>;

// SAFETY:
// 2^N is a power of two (duh).
unsafe impl<const N: u32> Alignment for TwoTo<N> {
    #[inline(always)]
    fn size() -> usize {
        2usize.pow(N)
    }
}

cfg_if! {
    if #[cfg(doc)] {
        #[cfg_attr(docsrs, doc(cfg(feature = "simd")))]
        mod simd;

        #[cfg_attr(docsrs, doc(cfg(feature = "simd")))]
        #[doc(inline)]
        pub use simd::*;
    }
    else if #[cfg(feature = "simd")] {
        mod simd;
        pub use simd::*;
    }
}

mod page;
pub use page::*;
mod multiple;
pub use multiple::*;
