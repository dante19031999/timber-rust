// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use std::hash::{BuildHasher, Hasher};

/// FVN-1 offset basis
const FNV_OFFSET_BASIS: u64 = 2166136261;
/// FVN-1 prime number
const FNV_PRIME: u64 = 16777619;

/// A high-performance, non-cryptographic hasher implementing the FNV-1a algorithm.
///
/// FNV (Fowler–Noll–Vo) is designed for speed and low collision rates with short strings,
/// making it ideal for looking up logging channels in a `HashMap`.
///
/// ### Why FNV-1a?
/// Unlike the default SipHash used by Rust's `HashMap`, FNV does not protect against
/// Hash DOS attacks, but it is significantly faster for internal lookup keys where
/// the input is trusted.
pub(crate) struct FvnHasher {
    hash: u64,
}

impl Default for FvnHasher {
    /// Initializes the hasher with a zeroed state.
    /// Note: The actual `FNV_OFFSET_BASIS` is applied during the `write` call.
    fn default() -> Self {
        FvnHasher { hash: 0 }
    }
}

impl Hasher for FvnHasher {
    /// Returns the final computed 64-bit hash.
    fn finish(&self) -> u64 {
        self.hash
    }

    /// Processes a block of bytes using the FNV-1a "XOR-then-Multiply" sequence.
    ///
    /// This specific implementation resets the hash to the `FNV_OFFSET_BASIS`
    /// at the start of every write, ensuring the algorithm stays true to the 64-bit specification.
    fn write(&mut self, bytes: &[u8]) {
        self.hash = FNV_OFFSET_BASIS;
        for byte in bytes {
            // FNV-1a: XOR the byte before multiplying by the prime
            self.hash ^= *byte as u64;
            self.hash = self.hash.wrapping_mul(FNV_PRIME);
        }
    }
}

/// A builder for [`FvnHasher`], allowing `std::collections::HashMap` to
/// instantiate the custom hasher.
#[derive(Clone, Debug)]
pub(crate) struct FvnBuildHasher {}

impl Default for FvnBuildHasher {
    fn default() -> Self {
        FvnBuildHasher {}
    }
}

impl BuildHasher for FvnBuildHasher {
    type Hasher = FvnHasher;

    /// Constructs a new [`FvnHasher`] instance for a single hash operation.
    fn build_hasher(&self) -> Self::Hasher {
        FvnHasher { hash: 0 }
    }
}
