#!/bin/python3
import os


class TestCase:
    def __init__(self, feature, target_triple, toolchain, size):
        self.feature = feature
        self.target_triple = target_triple
        self.toolchain = toolchain
        self.size = size


def system_succeed(cmd):
    print(f"Executing {cmd}")
    ret = os.system(cmd)
    if ret != 0:
        print(f"Command {cmd} failed with exit code {ret}")
        exit(ret)


x86 = "x86_64-unknown-linux-gnu"
matrix = [
    TestCase("avx", x86, "stable", 32),
    TestCase("avx2", x86, "stable", 32),
    TestCase("sse", x86, "stable", 16),
    TestCase("sse2", x86, "stable", 16),
    TestCase("sse3", x86, "stable", 16),
    TestCase("sse4.1", x86, "stable", 16),
    TestCase("sse4.2", x86, "stable", 16),
    TestCase("ssse3", x86, "stable", 16),
    TestCase("avx512f", x86, "nightly", 64),
]

for test_case in matrix:
    print(f"TEST CASE: target_feature {test_case.feature} \
should cause SimdBlock to have size {test_case.size} \
when compiled with {test_case.toolchain} \
on {test_case.target_triple}")
    rustflags = f"-C target-feature=+{test_case.feature}"
    testflags = str(test_case.size)
    rustup = f"rustup override set {test_case.toolchain}"
    build = f"cargo build --target {test_case.target_triple}"
    test = f"cargo test simd_alignment_test --target {test_case.target_triple} -- --include-ignored"

    system_succeed(rustup)
    system_succeed(f"RUSTFLAGS=\"{rustflags}\" {build}")
    system_succeed(
        f"ALIGNERS_TEST_SIMD_EXPECTED_SIZE={test_case.size} RUSTFLAGS=\"{rustflags}\" {test}")
    print("TEST CASE OK")
