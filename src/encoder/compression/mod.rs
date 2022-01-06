use crate::tags::CompressionMethod;
use std::io::Write;
mod uncompressed;
pub use self::uncompressed::Uncompressed;

/// An algorithm used for compression with associated optional buffers and/or configurations.
pub trait Compression {
    /// The corresponding tag to the algorithm.
    const COMPRESSION_METHOD: CompressionMethod;
    fn write_to<W: Write>(&mut self, writer: &mut W, bytes: &[u8]) -> u64;
}

pub enum CompressionAlgorithm {
    None(Uncompressed),
    Undefined(Uncompressed),
}

pub fn get_compressor(compressionMethod: CompressionMethod) -> CompressionAlgorithm {
    match compressionMethod {
        CompressionMethod::None => CompressionAlgorithm::None,
        _ => CompressionAlgorithm::Undefined,
    }
}
#[cfg(test)]
mod tests {
    pub const TEST_DATA: &'static [u8] =
        b"This is a string for checking various compression algorithms.";
}
