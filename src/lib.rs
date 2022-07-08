#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(
    explicit_outlives_requirements,
    unreachable_pub,
    semicolon_in_expressions_from_macros,
    unused_import_braces,
    single_use_lifetimes,
    unused_lifetimes
)]
#![warn(
    clippy::undocumented_unsafe_blocks,
    clippy::cargo_common_metadata,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::ptr_as_ptr,
    clippy::cloned_instead_of_copied,
    clippy::unreadable_literal,
    clippy::must_use_candidate,
    clippy::missing_inline_in_public_items
)]
// feature(doc_cfg) is nightly (https://doc.rust-lang.org/unstable-book/language-features/doc-cfg.html)
// Since we don't want the entire crate to be nightly, this is enabled only when building documentation.
#![cfg_attr(docsrs, feature(doc_cfg))]
// Same goes for the revolutionary "link to pointers documentation" feature.
// (https://github.com/rust-lang/rust/issues/80896).
#![cfg_attr(docsrs, feature(intra_doc_pointers))]
#![cfg_attr(miri, feature(strict_provenance))]

//! Structures providing guarantees on byte sequence alignment.
//!
//! For some crucial data it might be beneficial to align them to page boundaries
//! for better cache performance. This crate uses the [`page_size`](https://crates.io/crates/page_size)
//! crate to get the page size.
//!
//! # Examples
//!
//! ```
//! # use aligners::{alignment::{self, Alignment}};
//! assert_eq!(page_size::get(), alignment::Page::size());
//! ```
//! ```
//! # use aligners::{Aligned, AlignedBytes, alignment::{self, Alignment}};
//! let possibly_unaligned = [1, 2, 3];
//! let aligned = AlignedBytes::<alignment::Page>::from(possibly_unaligned);
//! let ptr = aligned.as_ptr();
//!
//! assert_eq!(ptr.align_offset(page_size::get()), 0);
//! assert_eq!(aligned, possibly_unaligned);
//! ```
//!
//! To create a new aligned block of bytes it's easiest to use [`new_zeroed`](`AlignedBytes::new_zeroed`).
//!
//! ```
//! # use aligners::{Aligned, AlignedBytes, alignment::{self, Alignment}};
//! let aligned = AlignedBytes::<alignment::Page>::new_zeroed(1024);
//! let ptr = aligned.as_ptr();
//!
//! assert_eq!(ptr.align_offset(page_size::get()), 0);
//! assert!(aligned.iter().all(|&x| x == 0));
//! ```
//!
//! You can also use [`new`](`AlignedBytes::new`) to possibly skip initialization.
//! This is `unsafe`, since the underlying memory might be uninitialized, but may be useful
//! if you immediately want to initialize the memory afterwards.
//!
//! ```
//! # use aligners::{Aligned, AlignedBytes, alignment::{self, Alignment}};
//! let mut aligned = unsafe { AlignedBytes::<alignment::Page>::new(1024) };
//! let ptr = aligned.as_ptr();
//!
//! assert_eq!(ptr.align_offset(page_size::get()), 0);
//!
//! // We cannot assert anything else, `aligned` can contain arbitrary bytes.
//! // To be able to read anything, we must first initialize.
//!
//! for i in 0..1024 {
//!     aligned[i] = 1;
//! }
//!
//! let ones = std::iter::repeat(1).take(1024).collect::<Vec<u8>>();
//! assert_eq!(ones, aligned);
//!
//! ```
//!
//! If you want a safe way to initialize the bytes, there is [`new_initialize`](`AlignedBytes::new_initialize`)
//! that initializes all bytes with a function of their index.
//!
//! ```
//! # use aligners::{Aligned, AlignedBytes, alignment::{self, Alignment}};
//! let aligned = AlignedBytes::<alignment::Page>::new_initialize(8, |i| { i as u8 });
//! let ptr = aligned.as_ptr();
//!
//! assert_eq!(ptr.align_offset(page_size::get()), 0);
//! assert_eq!(aligned, [0, 1, 2, 3, 4, 5, 6, 7]);
//! ```
//!
//! ## SIMD
//!
//! Loading block-aligned bytes into SIMD is generally preferred over unaligned.
//! The SIMD alignment constructs are enabled with the `simd` default feature.
//!
#![cfg_attr(not(feature = "simd"), doc = "```ignore")]
#![cfg_attr(feature = "simd", doc = "```")]
//! # use aligners::{Aligned, AlignedBytes, alignment::{self, Alignment}};
//! let possibly_unaligned = [1, 2, 3];
//! let aligned = AlignedBytes::<alignment::SimdBlock>::from(possibly_unaligned);
//! let ptr = aligned.as_ptr();
//!
//! assert_eq!(ptr.align_offset(alignment::SimdBlock::size()), 0);
//! assert_eq!(aligned, possibly_unaligned);
//! ```
//!
//! ## Note on alignment checking
//!
//! Checking alignment is hard.
#![cfg_attr(docsrs, doc = "[`pointer::align_offset`]")]
#![cfg_attr(not(docsrs), doc = "`pointer::align_offset`")]
//! can return [`usize::MAX`]
//! without any reason and it explicitly says that it should not be relied on for correctness.
//!
//! The above examples ignore that for assertion purposes, because there is no better way.
//! In reality, the check in
#![cfg_attr(docsrs, doc = "[`align_offset`](pointer::align_offset)")]
#![cfg_attr(not(docsrs), doc = "`align_offset`")]
//! just checks the remainder
//! of the pointer's integer representation, plus some additional bounds checking, so it is
//! fine for testing purposes on most platforms.
//!
//! This is an additional benefit of using `aligners`, as it is a strong guarantee on alignment.
//!
//! If you disagree with this assessment, feel free to [contribute to this StackOverflow question](https://stackoverflow.com/questions/71972143/assert-that-a-pointer-is-aligned-to-some-value).
//!

pub mod alignment;
mod bytes;
mod iterators;
mod slice;

#[cfg(test)]
pub(crate) mod test;

pub use bytes::*;
pub use iterators::*;
pub use slice::*;

/// Common trait for [`AlignedBytes`] for all different alignments.
pub trait Aligned {
    /// Return the size of the alignment in bytes.
    fn alignment_size() -> usize;

    /// Return the slice of the bytes offset by `count` alignment units.
    fn offset(&self, count: isize) -> &Self;
}

// TODO: Implement indexing?
// TODO: Implement IntoIterator for AlignedBytes and an Iterator for AlignedSlice that iterates over aligned blocks.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_page_aligned_when_created_from_unaligned_slice() {
        let alignment_size = page_size::get();
        let slice: &[u8] = &std::iter::repeat(42)
            .take(alignment_size)
            .collect::<Vec<_>>();
        let misalignment = slice.as_ptr() as usize % alignment_size;
        let source = if misalignment > 0 { slice } else { &slice[1..] };
        let bytes = AlignedBytes::<alignment::Page>::from(source);

        assert_eq!(bytes.as_ptr() as usize % alignment_size, 0);
    }

    #[test]
    fn contains_same_bytes_when_page_aligned_from_slice() {
        let slice = (0..=47).collect::<Vec<u8>>();
        let bytes = AlignedBytes::<alignment::Page>::from(&slice);

        assert_eq!(bytes, slice);
    }

    #[test]
    fn creates_empty_bytes_when_given_zero_length_for_page() {
        let bytes = AlignedBytes::<alignment::Page>::new_zeroed(0);

        assert_eq!(bytes.len(), 0);
    }
}
