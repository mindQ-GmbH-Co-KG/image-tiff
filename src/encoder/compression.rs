use crate::{error::TiffResult, tags::CompressionMethod};

extern crate flate2;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::prelude::*;

extern crate weezl;
use weezl::encode::Encoder as LZWEncoder;

/// Indicates whether a compression method is supported for encoding image data.
pub fn supported(compression: CompressionMethod) -> bool {
    match compression {
        CompressionMethod::None => true,
        CompressionMethod::LZW => true,
        CompressionMethod::Deflate => true,
        _ => false,
    }
}

pub trait Compressor {
    fn compress(&self, bytes: Vec<u8>) -> TiffResult<Vec<u8>>;
}

pub struct NoneCompressor {}

impl Compressor for NoneCompressor {
    fn compress(&self, bytes: Vec<u8>) -> TiffResult<Vec<u8>> {
        Ok(bytes)
    }
}

pub struct DeflateCompressor {}

impl Compressor for DeflateCompressor {
    fn compress(&self, bytes: Vec<u8>) -> TiffResult<Vec<u8>> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes)?;
        Ok(encoder.finish()?)
    }
}

pub struct LZWCompressor {}

impl Compressor for LZWCompressor {
    fn compress(&self, bytes: Vec<u8>) -> TiffResult<Vec<u8>> {
        let mut encoder = LZWEncoder::with_tiff_size_switch(weezl::BitOrder::Msb, 8);
        Ok(encoder.encode(&bytes)?)
    }
}
