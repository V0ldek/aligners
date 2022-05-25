#[cfg(feature = "simd")]
mod tests {
    use anyhow::{bail, Result};

    const EXPECTED_SWITCH_ENV_NAME: &str = "ALIGNERS_TEST_SIMD_EXPECTED_SIZE";

    #[test]
    #[ignore]
    pub fn simd_alignment_test() {
        let verify = verify_simd_block_size();
        verify.unwrap();
    }

    pub fn verify_simd_block_size() -> Result<()> {
        use aligners::alignment::{Alignment, SimdBlock};
        let expected_size: usize = std::env::var(EXPECTED_SWITCH_ENV_NAME)?.parse()?;
        let actual_size = SimdBlock::size();

        if expected_size != actual_size {
            bail!(
                "Expected SIMD block size was '{}', but actual is '{}'",
                expected_size,
                actual_size
            );
        }

        Ok(())
    }
}
