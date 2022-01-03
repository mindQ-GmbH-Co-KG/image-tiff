use std::{convert::TryInto, io::prelude::*};

use crate::{
    encoder::{compression::Compressor, ColorType, DirectoryEncoder, TiffKind, TiffValue},
    error::TiffResult,
    tags::CompressionMethod,
    TiffError,
};

/// Compressor that uses the Packbits[^note] algorithm to compress bytes.
///
/// [^note]: PackBits is often ineffective on continuous tone images,
///          including many grayscale images. In such cases, it is better
///          to leave the image uncompressed.
#[derive(Debug, Clone, Copy, Default)]
pub struct PackbitsCompressor;

impl Compressor for PackbitsCompressor {
    const COMPRESSION_METHOD: CompressionMethod = CompressionMethod::PackBits;

    fn write_to<'a, T: ColorType, K: TiffKind, W: 'a + Write + Seek>(
        &mut self,
        encoder: &mut DirectoryEncoder<'a, W, K>,
        value: &[T::Inner],
    ) -> TiffResult<(K::OffsetType, K::OffsetType)>
    where
        [T::Inner]: TiffValue,
    {
        let bytes = value.data();
        let mut bytes_written = 0usize;
        let mut offset: Option<u64> = None;

        // Port from https://github.com/skirridsystems/packbits
        const MIN_REPT: u8 = 3; // Minimum run to compress between differ blocks
        const MAX_BYTES: u8 = 128; // Maximum number of bytes that can be encoded in a header byte

        // Encoding for header byte based on number of bytes represented.
        fn encode_diff(n: u8) -> u8 {
            n - 1
        }
        fn encode_rept(n: u8) -> u8 {
            let var = 256 - (n - 1) as u16;
            var as u8
        }

        let mut src_index: usize = 0; // Index of the current byte
        let mut src_count = bytes.len();

        let mut in_run = false;
        let mut run_index = 0u8; // Distance into pending bytes that a run starts

        let mut bytes_pending = 0u8; // Bytes looked at but not yet output
        let mut pending_index = 0usize; // Index of the first pending byte

        let mut curr_byte: u8; // Byte currently being considered
        let mut last_byte: u8; // Previous byte

        // Need at least one byte to compress
        if src_count == 0 {
            return Err(TiffError::CompressionError);
        }

        // Prime compressor with first character.
        last_byte = bytes[src_index];
        src_index += 1;
        bytes_pending += 1;

        while src_count - 1 != 0 {
            src_count -= 1;
            curr_byte = bytes[src_index];
            src_index += 1;
            bytes_pending += 1;

            if in_run {
                if (curr_byte != last_byte) || (bytes_pending > MAX_BYTES) {
                    offset.get_or_insert(encoder.write_data(encode_rept(bytes_pending - 1))?);
                    encoder.write_data(last_byte)?;
                    bytes_written += 2;

                    bytes_pending = 1;
                    pending_index = src_index - 1;
                    run_index = 0;
                    in_run = false;
                }
            } else {
                if bytes_pending > MAX_BYTES {
                    // We have as much differing data as we can output in one chunk.
                    // Output MAX_BYTES leaving one byte.
                    offset.get_or_insert(encoder.write_data(encode_diff(MAX_BYTES))?);
                    encoder
                        .write_data(&bytes[pending_index..pending_index + MAX_BYTES as usize])?;
                    bytes_written += 1 + MAX_BYTES as usize;

                    pending_index += MAX_BYTES as usize;
                    bytes_pending -= MAX_BYTES;
                    run_index = bytes_pending - 1; // A run could start here
                } else if curr_byte == last_byte {
                    if (bytes_pending - run_index >= MIN_REPT) || (run_index == 0) {
                        // This is a worthwhile run
                        if run_index != 0 {
                            // Flush differing data out of input buffer
                            offset.get_or_insert(encoder.write_data(encode_diff(run_index))?);
                            encoder.write_data(
                                &bytes[pending_index..pending_index + run_index as usize],
                            )?;
                            bytes_written += 1 + run_index as usize;
                        }
                        bytes_pending -= run_index; // Length of run
                        in_run = true;
                    }
                } else {
                    run_index = bytes_pending - 1; // A run could start here
                }
            }
            last_byte = curr_byte;
        }

        // Output the remainder
        if in_run {
            bytes_written += 2;
            offset.get_or_insert(encoder.write_data(encode_rept(bytes_pending))?);
            encoder.write_data(last_byte)?;
        } else {
            bytes_written += 1 + bytes_pending as usize;
            offset.get_or_insert(encoder.write_data(encode_diff(bytes_pending))?);
            encoder.write_data(&bytes[pending_index..pending_index + bytes_pending as usize])?;
        }

        if offset.is_none() {
            // should never be reached
            return Err(TiffError::CompressionError);
        }

        Ok((
            K::convert_offset(offset.unwrap())?,
            bytes_written.try_into()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::compression::tests::{compress, TEST_DATA};

    #[test]
    fn test_packbits_single() {
        // compress single byte
        const UNCOMPRESSED_DATA: [u8; 1] = [0x3F];
        const EXPECTED_COMPRESSED_DATA: [u8; 2] = [0x00, 0x3F];

        let compressed_data = compress(&UNCOMPRESSED_DATA, PackbitsCompressor::default());
        assert_eq!(compressed_data, EXPECTED_COMPRESSED_DATA);
    }

    #[test]
    fn test_packbits_rept() {
        // compress buffer with repetitive sequence
        const UNCOMPRESSED_DATA: &'static [u8] =
            b"This strrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrring hangs.";
        const EXPECTED_COMPRESSED_DATA: &'static [u8] = b"\x06This st\xD1r\x09ing hangs.";

        let compressed_data = compress(UNCOMPRESSED_DATA, PackbitsCompressor::default());
        assert_eq!(compressed_data, EXPECTED_COMPRESSED_DATA);
    }

    #[test]
    fn test_packbits_large_rept_nonrept() {
        // compress buffer with large repetitive and non-repetitive sequence
        let mut data = b"This st".to_vec();
        for _i in 0..158 {
            data.push(b'r');
        }
        data.extend_from_slice(b"ing hangs.");
        for i in 0..158 {
            data.push(i);
        }

        const EXPECTED_COMPRESSED_DATA: [u8; 182] = [
            0x06, 0x54, 0x68, 0x69, 0x73, 0x20, 0x73, 0x74, 0x81, 0x72, 0xE3, 0x72, 0x7F, 0x69,
            0x6E, 0x67, 0x20, 0x68, 0x61, 0x6E, 0x67, 0x73, 0x2E, 0x00, 0x01, 0x02, 0x03, 0x04,
            0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12,
            0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20,
            0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E,
            0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C,
            0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A,
            0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58,
            0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66,
            0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x27, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F, 0x80, 0x81,
            0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F,
            0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D,
        ];

        let compressed_data = compress(data.as_slice(), PackbitsCompressor::default());
        assert_eq!(compressed_data, EXPECTED_COMPRESSED_DATA);
    }

    #[test]
    fn test_packbits_string() {
        // compress teststring
        const EXPECTED_COMPRESSED_DATA: &'static [u8] =
            b"\x3CThis is a string for checking various compression algorithms.";

        let compressed_data = compress(TEST_DATA, PackbitsCompressor::default());
        assert_eq!(compressed_data, EXPECTED_COMPRESSED_DATA);
    }
}
