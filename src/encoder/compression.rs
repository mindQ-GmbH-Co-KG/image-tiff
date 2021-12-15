use std::io::prelude::*;

use crate::{error::TiffResult, tags::CompressionMethod};

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
pub struct NoneCompressor {}

impl Compressor for NoneCompressor {
    fn compress(&self, bytes: Vec<u8>) -> TiffResult<Vec<u8>> {
        Ok(bytes)
    }
}

/// Compressor that uses the LZW algorithm to compress bytes.
pub struct LZWCompressor {}

impl Compressor for LZWCompressor {
    fn compress(&self, bytes: Vec<u8>) -> TiffResult<Vec<u8>> {
        let mut encoder = LZWEncoder::with_tiff_size_switch(weezl::BitOrder::Msb, 8);
        Ok(encoder.encode(&bytes)?)
    }
}

/// Compressor that uses the Deflate algorithm to compress bytes.
/// (The compression level "default" is used)
pub struct DeflateCompressor {}

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
pub struct PackbitsCompressor {}

impl Compressor for PackbitsCompressor {
    fn compress(&self, bytes: Vec<u8>) -> TiffResult<Vec<u8>> {
        if bytes.len() == 0 {
            return Ok(Vec::new());
        }

        let mut compressed_data = Vec::<u8>::new();
        let mut buffer = Vec::<u8>::new();
        let mut pos: usize = 0;
        let mut repeat_count: u8 = 0;
        const MAX_LENGTH: usize = 127;
        let mut byte_is_repeating = false;

        fn append_raw(buffer: &mut Vec<u8>, result: &mut Vec<u8>) {
            if buffer.len() == 0 {
                return;
            }
            result.push(buffer.len() as u8 - 1);
            result.append(buffer);
        }

        fn append_rle(result: &mut Vec<u8>, repeat_count: u8) {
            let val: u16 = 256 - (repeat_count - 1) as u16;
            result.push(val as u8);
        }

        while pos < bytes.len() - 1 {
            let current_byte = bytes[pos];

            if bytes[pos] == bytes[pos + 1] {
                if byte_is_repeating == false {
                    // end of RAW bytes
                    append_raw(&mut buffer, &mut compressed_data);
                    byte_is_repeating = true;
                    repeat_count = 1;
                } else if byte_is_repeating == true {
                    if repeat_count as usize == MAX_LENGTH {
                        // restart the encoding
                        append_rle(&mut compressed_data, repeat_count);
                        repeat_count = 0;
                        compressed_data.push(bytes[pos]);
                    }
                    // move to next byte
                    repeat_count += 1;
                }
            } else {
                if byte_is_repeating == true {
                    repeat_count += 1;
                    append_rle(&mut compressed_data, repeat_count);
                    repeat_count = 0;
                    compressed_data.push(bytes[pos]);
                    byte_is_repeating = false;
                } else if byte_is_repeating == false {
                    if buffer.len() == MAX_LENGTH {
                        // restart the encoding
                        append_raw(&mut buffer, &mut compressed_data);
                        ();
                    }
                    buffer.push(current_byte);
                }
            }
            pos += 1;
        }

        if byte_is_repeating == false {
            buffer.push(bytes[pos]);
            append_raw(&mut buffer, &mut compressed_data);
            ();
        } else {
            repeat_count += 1;
            append_rle(&mut compressed_data, repeat_count);
            compressed_data.push(bytes[pos]);
        }

        Ok(compressed_data)
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
