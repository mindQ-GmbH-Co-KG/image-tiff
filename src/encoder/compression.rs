use crate::{error::TiffResult, tags::CompressionMethod, TiffError::CompressionError};
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
        const MIN_REPT: u16 = 3; // Minimum run to compress between differ blocks
        const MAX_REPT: u16 = 128; // Maximum run of repeated byte
        const MAX_DIFF: u16 = 128; // Maximum run of differing bytes

        // Encoding for header byte based on number of bytes represented.
        fn encode_diff(n: u16) -> u8 {
            (n - 1) as u8
        }
        fn encode_rept(n: u16) -> u8 {
            // like wrapping i8 and cast to u8
            let var = 256 - (n - 1);
            var as u8
        }

        let mut dest = Vec::<u8>::new();
        let mut src_itr: usize = 0; // Index of the current byte
        let mut src_count = bytes.len();

        let mut in_run = false;
        let mut run_start: u16 = 0; // Distance into pending bytes that a run starts

        let mut bytes_pending: u16 = 0; // Bytes looked at but not yet output
        let mut pending_itr: usize = 0; // Index of the first pending byte

        let mut curr_byte: u8; // Byte currently being considered
        let mut last_byte: u8; // Previous byte

        // Need at least one byte to compress
        if src_count == 0 {
            return Err(CompressionError {});
        }

        // Prime compressor with first character.
        last_byte = bytes[src_itr];
        src_itr += 1;
        bytes_pending += 1;

        while src_count - 1 != 0 {
            src_count -= 1;
            curr_byte = bytes[src_itr];
            src_itr += 1;
            bytes_pending += 1;

            if in_run {
                if (curr_byte != last_byte) || (bytes_pending > MAX_REPT) {
                    dest.push(encode_rept(bytes_pending - 1));
                    dest.push(last_byte);

                    bytes_pending = 1;
                    pending_itr = src_itr - 1;
                    run_start = 0;
                    in_run = false;
                }
            } else {
                if bytes_pending > MAX_DIFF {
                    // We have as much differing data as we can output in one chunk.
                    // Output MAX_DIFF leaving one byte.
                    dest.push(encode_diff(MAX_DIFF));
                    dest.extend_from_slice(&bytes[pending_itr..pending_itr + MAX_DIFF as usize]);

                    pending_itr += MAX_DIFF as usize;
                    bytes_pending -= MAX_DIFF;
                    run_start = bytes_pending - 1; // A run could start here
                } else if curr_byte == last_byte {
                    if (bytes_pending - run_start >= MIN_REPT) || (run_start == 0) {
                        // This is a worthwhile run
                        if run_start != 0 {
                            // Flush differing data out of input buffer
                            dest.push(encode_diff(run_start));
                            dest.extend_from_slice(
                                &bytes[pending_itr..pending_itr + run_start as usize],
                            );
                        }
                        bytes_pending -= run_start; // Length of run
                        in_run = true;
                    }
                } else {
                    run_start = bytes_pending - 1; // A run could start here
                }
            }
            last_byte = curr_byte;
        }

        // Output the remainder
        if in_run {
            dest.push(encode_rept(bytes_pending));
            dest.push(last_byte);
        } else {
            dest.push(encode_diff(bytes_pending));
            dest.extend_from_slice(&bytes[pending_itr..pending_itr + bytes_pending as usize]);
        }

        Ok(dest)
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

        // compress buffer with repetitive sequences
        {
            let data =
                b"This strrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrring hangs.".to_vec();
            let compressed_data = compressor.compress(data).unwrap();
            let expected = b"\x06This st\xD1r\x09ing hangs.".to_vec();
            assert_eq!(compressed_data, expected);
        }

        // compress teststring
        {
            let compressed_data = compressor.compress(get_test_data()).unwrap();
            let expected =
                b"+This is a string for checking various compre\xFFs\x0Eion algorithms.".to_vec();
            assert_eq!(compressed_data, expected);
        }
    }
}
