use crate::{error::TiffResult, tags::CompressionMethod};
use std::io::prelude::*;

extern crate weezl;
use weezl::encode::Encoder as LZWEncoder;

extern crate flate2;
use flate2::{write::ZlibEncoder, Compression};

/// Indicates whether a compression method is supported for encoding image data.
pub fn supported(compression: CompressionMethod) -> bool {
    match compression {
        CompressionMethod::None => true,
        CompressionMethod::LZW => true,
        CompressionMethod::Deflate => true,
        CompressionMethod::PackBits => true,
        _ => false,
    }
}

/// Trait for objects that can compress bytes.
pub trait Compressor {
    fn compress(&self, bytes: Vec<u8>) -> TiffResult<Vec<u8>>;
}

/// Compressor that does not compress any bytes.
pub struct NoneCompressor;

impl Compressor for NoneCompressor {
    fn compress(&self, bytes: Vec<u8>) -> TiffResult<Vec<u8>> {
        Ok(bytes)
    }
}

/// Compressor that uses the LZW algorithm to compress bytes.
pub struct LZWCompressor;

impl Compressor for LZWCompressor {
    fn compress(&self, bytes: Vec<u8>) -> TiffResult<Vec<u8>> {
        let mut encoder = LZWEncoder::with_tiff_size_switch(weezl::BitOrder::Msb, 8);
        Ok(encoder.encode(&bytes)?)
    }
}

/// Compressor that uses the Deflate algorithm to compress bytes.
/// (The compression level "default" is used)
pub struct DeflateCompressor;

impl Compressor for DeflateCompressor {
    fn compress(&self, bytes: Vec<u8>) -> TiffResult<Vec<u8>> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes)?;
        Ok(encoder.finish()?)
    }
}

/// Compressor that uses the Packbits[^note] algorithm to compress bytes.
///
/// [^note]: PackBits is often ineffective on continuous tone images,
///          including many grayscale images. In such cases, it is better
///          to leave the image uncompressed.
pub struct PackbitsCompressor;

impl Compressor for PackbitsCompressor {
    fn compress(&self, bytes: Vec<u8>) -> TiffResult<Vec<u8>> {
        // Port from https://github.com/skirridsystems/packbits
        const MIN_REPT: u8 = 3; // minimum run to compress between differ blocks
        const MAX_BYTES: u8 = 128; // maximum number of bytes that can be encoded in a header byte

        // Encoding for header byte based on number of bytes represented.
        fn encode_diff(n: u8) -> u8 {
            n - 1
        }
        fn encode_rept(n: u8) -> u8 {
            // like wrapping i8 and cast to u8
            let var = 256 - (n as u16 - 1);
            var as u8
        }

        let mut compressed_bytes = Vec::<u8>::new();
        let mut src_index: usize = 0; // Index of the current byte
        let mut src_count = bytes.len();

        let mut in_run = false;
        let mut run_index: u8 = 0; // Distance into pending bytes that a run starts

        let mut bytes_pending: u8 = 0; // Bytes looked at but not yet output
        let mut pending_index: usize = 0; // Index of the first pending byte

        let mut curr_byte: u8; // Byte currently being considered
        let mut last_byte: u8; // Previous byte

        // Need at least one byte to compress
        if src_count == 0 {
            return Ok(Vec::new());
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
                    compressed_bytes.push(encode_rept(bytes_pending - 1));
                    compressed_bytes.push(last_byte);

                    bytes_pending = 1;
                    pending_index = src_index - 1;
                    run_index = 0;
                    in_run = false;
                }
            } else {
                if bytes_pending > MAX_BYTES {
                    // We have as much differing data as we can output in one chunk.
                    // Output MAX_BYTES leaving one byte.
                    compressed_bytes.push(encode_diff(MAX_BYTES));
                    compressed_bytes.extend_from_slice(
                        &bytes[pending_index..pending_index + MAX_BYTES as usize],
                    );

                    pending_index += MAX_BYTES as usize;
                    bytes_pending -= MAX_BYTES;
                    run_index = bytes_pending - 1; // A run could start here
                } else if curr_byte == last_byte {
                    if (bytes_pending - run_index >= MIN_REPT) || (run_index == 0) {
                        // This is a worthwhile run
                        if run_index != 0 {
                            // Flush differing data out of input buffer
                            compressed_bytes.push(encode_diff(run_index));
                            compressed_bytes.extend_from_slice(
                                &bytes[pending_index..pending_index + run_index as usize],
                            );
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
            compressed_bytes.push(encode_rept(bytes_pending));
            compressed_bytes.push(last_byte);
        } else {
            compressed_bytes.push(encode_diff(bytes_pending));
            compressed_bytes
                .extend_from_slice(&bytes[pending_index..pending_index + bytes_pending as usize]);
        }

        Ok(compressed_bytes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        b"This is a string for checking various compression algorithms.".to_vec()
    }

    #[test]
    fn test_suported_compressionmethods() {
        assert_eq!(supported(CompressionMethod::None), true);
        assert_eq!(supported(CompressionMethod::Huffman), false);
        assert_eq!(supported(CompressionMethod::Fax3), false);
        assert_eq!(supported(CompressionMethod::Fax4), false);
        assert_eq!(supported(CompressionMethod::LZW), true);
        assert_eq!(supported(CompressionMethod::JPEG), false);
        assert_eq!(supported(CompressionMethod::ModernJPEG), false);
        assert_eq!(supported(CompressionMethod::Deflate), true);
        assert_eq!(supported(CompressionMethod::OldDeflate), false);
        assert_eq!(supported(CompressionMethod::PackBits), true);
    }

    #[test]
    fn test_no_compression() {
        let compressor = NoneCompressor {};
        let compressed_data = compressor.compress(get_test_data()).unwrap();
        assert_eq!(compressed_data, get_test_data());
    }

    #[test]
    fn test_deflate() {
        let compressor = DeflateCompressor {};
        let compressed_data = compressor.compress(get_test_data()).unwrap();
        let expected = vec![
            0x78, 0x9C, 0x15, 0xC7, 0xD1, 0x0D, 0x80, 0x20, 0x0C, 0x04, 0xD0, 0x55, 0x6E, 0x02,
            0xA7, 0x71, 0x81, 0xA6, 0x41, 0xDA, 0x28, 0xD4, 0xF4, 0xD0, 0xF9, 0x81, 0xE4, 0xFD,
            0xBC, 0xD3, 0x9C, 0x58, 0x04, 0x1C, 0xE9, 0xBD, 0xE2, 0x8A, 0x84, 0x5A, 0xD1, 0x7B,
            0xE7, 0x97, 0xF4, 0xF8, 0x08, 0x8D, 0xF6, 0x66, 0x21, 0x3D, 0x3A, 0xE4, 0xA9, 0x91,
            0x3E, 0xAC, 0xF1, 0x98, 0xB9, 0x70, 0x17, 0x13,
        ];
        assert_eq!(compressed_data, expected);
    }

    #[test]
    fn test_lzw() {
        let compressor = LZWCompressor {};
        let compressed_data = compressor.compress(get_test_data()).unwrap();
        let expected = vec![
            0x80, 0x15, 0x0D, 0x06, 0x93, 0x98, 0x82, 0x08, 0x20, 0x30, 0x88, 0x0E, 0x67, 0x43,
            0x91, 0xA4, 0xDC, 0x67, 0x10, 0x19, 0x8D, 0xE7, 0x21, 0x01, 0x8C, 0xD0, 0x65, 0x31,
            0x9A, 0xE1, 0xD1, 0x03, 0xB1, 0x86, 0x1A, 0x6F, 0x3A, 0xC1, 0x4C, 0x66, 0xF3, 0x69,
            0xC0, 0xE4, 0x65, 0x39, 0x9C, 0xCD, 0x26, 0xF3, 0x74, 0x20, 0xD8, 0x67, 0x89, 0x9A,
            0x4E, 0x86, 0x83, 0x69, 0xCC, 0x5D, 0x01,
        ];
        assert_eq!(compressed_data, expected);
    }

    #[test]
    fn test_packbits() {
        let compressor = PackbitsCompressor {};

        // compress empty buffer
        {
            let compressed_data = compressor.compress(Vec::<u8>::new()).unwrap();
            let expected = Vec::<u8>::new();
            assert_eq!(compressed_data, expected);
        }

        // compress single byte
        {
            let compressed_data = compressor.compress(vec![0x3F]).unwrap();
            let expected = vec![0x00, 0x3F];
            assert_eq!(compressed_data, expected);
        }

        // compress buffer with repetitive sequence
        {
            let data =
                b"This strrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrring hangs.".to_vec();
            let compressed_data = compressor.compress(data).unwrap();
            let expected = b"\x06This st\xD1r\x09ing hangs.".to_vec();
            assert_eq!(compressed_data, expected);
        }

        // compress buffer with large repetitive and non-repetitive sequence
        {
            let mut data = b"This st".to_vec();
            for _i in 0..158 {
                data.push(b'r');
            }
            data.extend_from_slice(b"ing hangs.");
            for i in 0..158 {
                data.push(i);
            }

            let compressed_data = compressor.compress(data).unwrap();
            let expected = vec![
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
            assert_eq!(compressed_data, expected);
        }

        // compress teststring
        {
            let compressed_data = compressor.compress(get_test_data()).unwrap();
            let expected =
                b"\x3CThis is a string for checking various compression algorithms.".to_vec();
            assert_eq!(compressed_data, expected);
        }
    }
}
