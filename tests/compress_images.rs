extern crate tiff;

use std::io::{Cursor, Seek, SeekFrom};
use tiff::decoder::{Decoder, DecodingResult};
use tiff::encoder::compression::*;
use tiff::encoder::{colortype, TiffEncoder};

fn encode_decode_with_compression<C: Compressor + Clone>(compression: C) {
    let mut img_file = Cursor::new(Vec::new());

    let data0_dims: (u32, u32) = (1, 7);
    let data1_dims: (u32, u32) = (21, 10);
    let mut data0: Vec<u16> = Vec::with_capacity((data0_dims.0 * data0_dims.1) as usize * 3);
    let mut data1: Vec<u8> = Vec::with_capacity((data1_dims.0 * data1_dims.1) as usize * 3);

    // create test data
    {
        for x in 0..data0_dims.0 {
            for y in 0..data0_dims.1 {
                let val = (x + y) % std::u16::MAX as u32;
                data0.push(val as u16);
                data0.push(val as u16);
                data0.push(val as u16);
            }
        }

        for x in 0..data1_dims.0 {
            for y in 0..data1_dims.1 {
                let mut val = (x + y) % std::u8::MAX as u32;
                // to have repeating values to compress
                if x < data1_dims.0 / 2 {
                    val = (data1_dims.0 / 2) % std::u16::MAX as u32;
                }
                data1.push(val as u8);
            }
        }
    }

    // encode tiff with compression
    {
        // first create a multipage image with 2 images
        let mut encoder = TiffEncoder::new(&mut img_file).unwrap();

        // write first colored image (100x70 16-bit)
        let image0 = encoder
            .new_image_with_compression::<colortype::RGB16, C>(
                data0_dims.0,
                data0_dims.1,
                compression.clone(),
            )
            .unwrap();
        image0.write_data(&data0[..]).unwrap();

        // write second grayscale image (210x100 8-bit)
        let image1 = encoder
            .new_image_with_compression::<colortype::Gray8, C>(
                data1_dims.0,
                data1_dims.1,
                compression,
            )
            .unwrap();
        image1.write_data(&data1[..]).unwrap();
    }
    img_file.seek(SeekFrom::Start(0)).unwrap();

    // decode tiff
    {
        let mut decoder = Decoder::new(&mut img_file).unwrap();

        if let DecodingResult::U16(image0) = decoder.read_image().unwrap() {
            assert_eq!(data0, image0);
        } else {
            panic!("Wrong data type");
        }

        decoder.next_image().unwrap();
        if let DecodingResult::U8(image1) = decoder.read_image().unwrap() {
            assert_eq!(data1, image1);
        } else {
            panic!("Wrong data type");
        }
    }
}

#[test]
fn encode_decode_without_compression() {
    let compressor = NoneCompressor;
    encode_decode_with_compression(compressor);
}

#[test]
fn encode_decode_with_lzw() {
    let compressor = LZWCompressor::default();
    encode_decode_with_compression(compressor);
}

#[test]
fn encode_decode_with_deflate() {
    let compressor = DeflateCompressor::default();
    encode_decode_with_compression(compressor);
}

#[test]
fn encode_decode_with_packbits() {
    let compressor = PackbitsCompressor::default();
    encode_decode_with_compression(compressor);
}
