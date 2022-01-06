use crate::{encoder::compression::*, tags::CompressionMethod};
use std::io::Write;

//
// TiffValue -> Tiffwriter -> Write = Compression -> Write = DirectoryEncoder
//

/// The default algorithm which does not compress at all.
#[derive(Default, Debug, Clone, Copy)]
pub struct Uncompressed;

impl Compression for Uncompressed {
    fn get_algorithm(&self) -> CompressionMethod {
        CompressionMethod::None
    }

    fn write_to<W: Write>(&mut self, writer: &mut W, bytes: &[u8]) -> u64 {
        writer.write(bytes).unwrap_or(0) as u64
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
        Uncompressed::default().write_to(&mut writer, TEST_DATA);
        assert_eq!(TEST_DATA, compressed_data);
    }
}
