use super::Alignment;

/// Alignment to twice the alignment of `A`.
///
/// This size is always equal to twice the size of `A`.
///
/// # Examples
/// ```rust
/// use aligners::alignment::{self, Alignment};
///
/// assert_eq!(alignment::Twice::<alignment::TwoTo::<2>>::size(), 2 * alignment::TwoTo::<2>::size());
/// ```
#[derive(Debug)]
pub struct Twice<A: Alignment> {
    a: std::marker::PhantomData<A>,
}

// SAFETY:
// Safe as long as the impl for `SimdBlock` is safe, since we multiply by 2.
unsafe impl<A: Alignment> Alignment for Twice<A> {
    #[inline]
    fn size() -> usize {
        A::size() * 2
    }
}
