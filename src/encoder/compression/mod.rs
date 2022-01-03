use std::io::prelude::*;

use crate::{
    encoder::{ColorType, DirectoryEncoder, TiffKind, TiffValue},
    error::TiffResult,
    tags::CompressionMethod,
};

mod deflate;
mod lzw;
mod packbits;
mod uncompressed;

pub use self::deflate::DeflateCompressor;
pub use self::lzw::LZWCompressor;
pub use self::packbits::PackbitsCompressor;
pub use self::uncompressed::NoneCompressor;

/// Trait for objects that can compress bytes.
pub trait Compressor {
    const COMPRESSION_METHOD: CompressionMethod;

    /// Write the data of a specific color type to the given encoder and return the offset and byte count, respectively.
    fn write_to<'a, T: ColorType, K: TiffKind, W: 'a + Write + Seek>(
        &mut self,
        encoder: &mut DirectoryEncoder<'a, W, K>,
        value: &[T::Inner],
    ) -> TiffResult<(K::OffsetType, K::OffsetType)>
    where
        [T::Inner]: TiffValue;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::colortype;
    use crate::encoder::TiffEncoder;
    use std::io::{Cursor, SeekFrom};

    pub const TEST_DATA: &'static [u8] =
        b"This is a string for checking various compression algorithms.";

    pub fn compress<C: Compressor>(bytes: &[u8], compressor: C) -> Vec<u8> {
        let mut file = Cursor::new(Vec::new());

        // write bytes as single strip to be compressed as one chunk
        let mut encoder = TiffEncoder::new(&mut file).unwrap();
        let img = encoder
            .new_image_with_compression::<colortype::Gray8, C>(bytes.len() as u32, 1, compressor)
            .unwrap();
        img.write_data(&bytes).unwrap();

        // unwrap compressed bytes from enclosing tiff
        let mut data = Vec::<u8>::new();
        file.seek(SeekFrom::Start(0)).unwrap();
        file.read_to_end(&mut data).unwrap();
        data.drain(..8);
        data.drain(data.len() - 178..);
        data
    }
}
