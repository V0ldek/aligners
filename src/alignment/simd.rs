use super::Alignment;
use cfg_if::cfg_if;

/// Alignment to a SIMD block guarantee.
///
/// It is guaranteed that this alignment's [`size`](`Alignment::size`) is a multiplicity
/// of the size of a SIMD register of the target architecture.
///
/// # Alignments
///
/// The alignment size will be the first entry in the below table
/// that is supported by the target CPU, as long as the application
/// is compiled with the appropriate [target feature](https://doc.rust-lang.org/reference/conditional-compilation.html#target_feature).
///
/// | CPU feature     | Alignment (bytes) | Required `target_feature`      |
/// |:----------------|------------------:|-------------------------------:|
/// | AVX512          | 64                | `avx512f`                      |
/// | AVX             | 32                | any of `avx`, `avx2`           |
/// | SSE             | 16                | any of `sse`, `sse2`, `sse3`, <br/> `sse4.1`, `sse4.2`, `ssse3` |
///
/// If the target does not support any of these extensions, the compilation will fail.
/// In that case you need to disable the `simd` feature.
#[derive(Debug)]
#[cfg_attr(docsrs, doc(cfg(feature = "simd")))]
pub enum SimdBlock {}

// SAFETY:
// Always returning a const value that is a power of two.
unsafe impl Alignment for SimdBlock {
    #[inline(always)]
    fn size() -> usize {
        cfg_if! {
            if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
                cfg_if! {
                    if #[cfg(target_feature = "avx512f")] {
                        64
                    }
                    else if #[cfg(target_feature = "avx")] {
                        32
                    }
                    else if #[cfg(target_feature = "sse")] {
                        16
                    }
                }
            } else if #[cfg(doc)] {
                32
            }
            else {
                compile_error!("Target architecture is not supported by SIMD features of this crate. Disable the default `simd` feature.");
                unreachable!();
            }
        }
    }
}
