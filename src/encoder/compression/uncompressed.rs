use crate::{encoder::compression::*, tags::CompressionMethod};
use std::io::Write;

//
// TiffValue -> Tiffwriter -> Write = Compression -> Write = DirectoryEncoder
//

/// The default algorithm which does not compress at all.
#[derive(Default, Debug, Clone, Copy)]
pub struct Uncompressed;

impl Compression for Uncompressed {
    /// The corresponding tag to the algorithm.
    const COMPRESSION_METHOD: CompressionMethod = CompressionMethod::None;

    fn get_algorithm(&self) -> Compressor {
        Compressor::Uncompressed(*self)
    }
}

impl CompressionAlgorithm for Uncompressed {
    fn write_to<W: Write>(&mut self, writer: &mut W, bytes: &[u8]) -> Result<u64, io::Error> {
        writer.write(bytes).map(|byte_count| byte_count as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::compression::tests::TEST_DATA;
    use std::io::Cursor;

    #[test]
    fn test_no_compression() {
        let mut compressed_data = Vec::<u8>::new();
        let mut writer = Cursor::new(&mut compressed_data);
        Uncompressed::default()
            .write_to(&mut writer, TEST_DATA)
            .unwrap();
        assert_eq!(TEST_DATA, compressed_data);
    }
}
