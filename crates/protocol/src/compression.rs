// SPDX-License-Identifier: AGPL-3.0-or-later
//! Static legacy Huffman compression with bounded, allocation-free operations.
//!
//! Decoding matches upstream's byte-length framing, including any extra symbol
//! decoded from final zero padding. The last code is safely zero-extended instead
//! of reading past the input buffer. Higher-level message EOF opcodes determine
//! logical contents; compression alone cannot establish message integrity.

mod codebook;
mod compression_error;
mod compressor;
mod decompressor;
mod symbol_decoder;

pub use compression_error::CompressionError;
pub use compressor::compress;
pub use decompressor::decompress;
