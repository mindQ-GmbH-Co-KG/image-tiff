use crate::tags::CompressionMethod;
use std::io::{self, Write};

mod deflate;
mod lzw;
mod packbits;
mod uncompressed;
pub use self::deflate::Deflate;
pub use self::deflate::DeflateLevel;
pub use self::lzw::Lzw;
pub use self::packbits::Packbits;
pub use self::uncompressed::Uncompressed;

/// An algorithm used for compression
pub trait CompressionAlgorithm {
    fn write_to<W: Write>(&mut self, writer: &mut W, bytes: &[u8]) -> Result<u64, io::Error>;
}

/// An algorithm used for compression with associated enums and optional configurations.
pub trait Compression: CompressionAlgorithm {
    /// The corresponding tag to the algorithm.
    const COMPRESSION_METHOD: CompressionMethod;
    fn get_algorithm(&self) -> Compressor;
}

/// An enum to store each compression algorithm.
pub enum Compressor {
    Uncompressed(Uncompressed),
    Lzw(Lzw),
    Deflate(Deflate),
    Packbits(Packbits),
}

impl Default for Compressor {
    fn default() -> Self {
        Compressor::Uncompressed(Uncompressed::default())
    }
}

impl CompressionAlgorithm for Compressor {
    fn write_to<W: Write>(&mut self, writer: &mut W, bytes: &[u8]) -> Result<u64, io::Error> {
        match self {
            Compressor::Uncompressed(algorithm) => algorithm.write_to(writer, bytes),
            Compressor::Lzw(algorithm) => algorithm.write_to(writer, bytes),
            Compressor::Deflate(algorithm) => algorithm.write_to(writer, bytes),
            Compressor::Packbits(algorithm) => algorithm.write_to(writer, bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    pub const TEST_DATA: &'static [u8] =
        b"This is a string for checking various compression algorithms.";
}
