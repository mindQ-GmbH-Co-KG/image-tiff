use std::{convert::TryInto, io::prelude::*};

use crate::{
    encoder::{compression::Compressor, ColorType, DirectoryEncoder, TiffKind, TiffValue},
    error::TiffResult,
    tags::CompressionMethod,
};

extern crate flate2;
use flate2::{write::ZlibEncoder, Compression};

/// Compressor that uses the Deflate algorithm to compress bytes.
#[derive(Debug, Clone)]
pub struct DeflateCompressor {
    pub level: Compression,
    buffer: Vec<u8>,
}

impl DeflateCompressor {
    /// Lets be greedy and allocate more bytes in advance. We will likely encode longer image strips.
    const DEFAULT_BUFFER_SIZE: usize = 256;

    /// Create a new deflate compr+essor with a specific level of compression.
    pub fn with_level(level: Compression) -> Self {
        Self {
            buffer: Vec::with_capacity(Self::DEFAULT_BUFFER_SIZE),
            level,
        }
    }
}

impl Default for DeflateCompressor {
    fn default() -> Self {
        Self {
            buffer: Vec::with_capacity(Self::DEFAULT_BUFFER_SIZE),
            level: Compression::default(),
        }
    }
}

impl Compressor for DeflateCompressor {
    const COMPRESSION_METHOD: CompressionMethod = CompressionMethod::Deflate;

    fn write_to<'a, T: ColorType, K: TiffKind, W: 'a + Write + Seek>(
        &mut self,
        encoder: &mut DirectoryEncoder<'a, W, K>,
        value: &[T::Inner],
    ) -> TiffResult<(K::OffsetType, K::OffsetType)>
    where
        [T::Inner]: TiffValue,
    {
        let data = value.data();
        {
            let mut encoder = ZlibEncoder::new(&mut self.buffer, self.level);
            encoder.write_all(&data)?;
            encoder.finish()?;
        }

        let compressed_byte_count = self.buffer.len().try_into()?;
        let offset = encoder
            .write_data(self.buffer.as_slice())
            .and_then(K::convert_offset)?;

        // Clear the buffer for the next compression.
        self.buffer.clear();

        Ok((offset, compressed_byte_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::compression::tests::{compress, TEST_DATA};

    #[test]
    fn test_deflate() {
        let compressed_data = compress(TEST_DATA, DeflateCompressor::default());

        const EXPECTED_COMPRESSED_DATA: [u8; 64] = [
            0x78, 0x9C, 0x15, 0xC7, 0xD1, 0x0D, 0x80, 0x20, 0x0C, 0x04, 0xD0, 0x55, 0x6E, 0x02,
            0xA7, 0x71, 0x81, 0xA6, 0x41, 0xDA, 0x28, 0xD4, 0xF4, 0xD0, 0xF9, 0x81, 0xE4, 0xFD,
            0xBC, 0xD3, 0x9C, 0x58, 0x04, 0x1C, 0xE9, 0xBD, 0xE2, 0x8A, 0x84, 0x5A, 0xD1, 0x7B,
            0xE7, 0x97, 0xF4, 0xF8, 0x08, 0x8D, 0xF6, 0x66, 0x21, 0x3D, 0x3A, 0xE4, 0xA9, 0x91,
            0x3E, 0xAC, 0xF1, 0x98, 0xB9, 0x70, 0x17, 0x13,
        ];
        assert_eq!(compressed_data, EXPECTED_COMPRESSED_DATA);
    }
}
