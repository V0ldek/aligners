use super::Alignment;

/// Alignment to page boundary.
///
/// Size is the size of a single page in the OS as returned by the
/// [`page_size`] crate.
///
/// # Examples
/// ```rust
/// use page_size;
/// use aligners::alignment::{self, Alignment};
///
/// assert_eq!(page_size::get(), alignment::Page::size());
/// ```
#[derive(Debug)]
pub enum Page {}

// SAFETY:
// We check whether the size is power of two. The [`page_size`] crate caches the result
// of its call, so it will not change, but I prefer not to rely on an external crate not changing
// its implementation for safety.
//
// No sane platform would have a page size that is not a power of two, but better not to take chances.
// This assertion will only be called once anyway.
unsafe impl Alignment for Page {
    #[inline]
    fn size() -> usize {
        use lazy_static::lazy_static;

        lazy_static! {
            static ref PAGE_SIZE: usize = {
                let size = page_size::get();

                if size.next_power_of_two() != size {
                    panic!(
                        "detected page size {size} that is not a power of two, this is unsupported"
                    );
                }

                size
            };
        }

        *PAGE_SIZE
    }
}
