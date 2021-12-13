extern crate tiff;

use std::io::{Cursor, Seek, SeekFrom};
use tiff::decoder::{Decoder, DecodingResult};
use tiff::encoder::{colortype, TiffEncoder};
use tiff::tags::CompressionMethod;

fn encode_decode_with_compression(compression: CompressionMethod) {
    let mut img_file = Cursor::new(Vec::new());

    let data0_dims: (u32, u32) = (100, 70);
    let data1_dims: (u32, u32) = (210, 100);
    let mut data0: Vec<u16> = Vec::with_capacity(100 * 70 * 3);
    let mut data1: Vec<u8> = Vec::with_capacity(210 * 100 * 3);

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
                let val = (x + y) % std::u8::MAX as u32;
                data1.push(val as u8);
            }
        }
    }

    // encode tiff with compression
    {
        // first create a multipage image with 2 images
        let mut encoder = TiffEncoder::new(&mut img_file).unwrap();

        // write first colored image (100x70 16-bit)
        let mut image0 = encoder
            .new_image::<colortype::RGB16>(data0_dims.0, data0_dims.1)
            .unwrap();
        image0.compression(compression).unwrap();
        image0.write_data(&data0[..]).unwrap();

        // write second grayscale image (210x100 8-bit)
        let mut image1 = encoder
            .new_image::<colortype::Gray8>(data1_dims.0, data1_dims.1)
            .unwrap();
        image1.compression(compression).unwrap();
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
    encode_decode_with_compression(CompressionMethod::None);
}

#[test]
#[should_panic(expected = "CompressionMethod is not supported.")]
fn encode_decode_with_huffman() {
    encode_decode_with_compression(CompressionMethod::Huffman);
}

#[test]
#[should_panic(expected = "CompressionMethod is not supported.")]
fn encode_decode_with_fax3() {
    encode_decode_with_compression(CompressionMethod::Fax3);
}

#[test]
#[should_panic(expected = "CompressionMethod is not supported.")]
fn encode_decode_with_fax4() {
    encode_decode_with_compression(CompressionMethod::Fax4);
}

#[test]
fn encode_decode_with_lzw() {
    encode_decode_with_compression(CompressionMethod::LZW);
}

#[test]
#[should_panic(expected = "CompressionMethod is not supported.")]
fn encode_decode_with_jpeg() {
    encode_decode_with_compression(CompressionMethod::JPEG);
}

#[test]
#[should_panic(expected = "CompressionMethod is not supported.")]
fn encode_decode_with_modernjpeg() {
    encode_decode_with_compression(CompressionMethod::ModernJPEG);
}

#[test]
fn encode_decode_with_deflate() {
    encode_decode_with_compression(CompressionMethod::Deflate);
}

#[test]
#[should_panic(expected = "CompressionMethod is not supported.")]
fn encode_decode_with_olddeflate() {
    encode_decode_with_compression(CompressionMethod::OldDeflate);
}

#[test]
#[should_panic(expected = "CompressionMethod is not supported.")]
fn encode_decode_with_packbits() {
    encode_decode_with_compression(CompressionMethod::PackBits);
}
