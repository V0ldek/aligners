#[cfg(test)]
pub(crate) fn assert_aligned<T>(ptr: *const T, alignment: usize) {
    cfg_if::cfg_if! {
        if #[cfg(miri)] {
            let as_int = ptr as usize;
            assert_eq!(0, as_int % alignment);
        }
        else {
            assert_eq!(0, ptr.align_offset(alignment));
        }
    }
}
