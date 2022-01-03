use std::{convert::TryInto, io::prelude::*};

use crate::{
    encoder::{compression::Compressor, ColorType, DirectoryEncoder, TiffKind, TiffValue},
    error::TiffResult,
    tags::CompressionMethod,
};

/// Compressor that does not compress any bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NoneCompressor;

impl Compressor for NoneCompressor {
    const COMPRESSION_METHOD: CompressionMethod = CompressionMethod::None;

    fn write_to<'a, T: ColorType, K: TiffKind, W: 'a + Write + Seek>(
        &mut self,
        encoder: &mut DirectoryEncoder<'a, W, K>,
        value: &[T::Inner],
    ) -> TiffResult<(K::OffsetType, K::OffsetType)>
    where
        [T::Inner]: TiffValue,
    {
        let byte_count = value.len().try_into()?;
        let offset = encoder.write_data(value).and_then(K::convert_offset)?;
        Ok((offset, byte_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::compression::tests::{compress, TEST_DATA};

    #[test]
    fn test_no_compression() {
        let compressed_data = compress(TEST_DATA, NoneCompressor::default());
        assert_eq!(compressed_data, TEST_DATA);
    }
}
