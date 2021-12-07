use std::io::Write;

use crate::{bytecast, tags::Type, TiffError, TiffFormatError, TiffResult};

use super::writer::TiffWriter;

/// Trait for types that can be encoded in a tiff file
pub trait TiffValue {
    const BYTE_LEN: u8;
    const FIELD_TYPE: Type;
    fn count(&self) -> usize;
    fn bytes(&self) -> usize {
        self.count() * usize::from(Self::BYTE_LEN)
    }
    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()>;
    fn serialize(&self) -> Vec<u8>;
}

impl TiffValue for [u8] {
    const BYTE_LEN: u8 = 1;
    const FIELD_TYPE: Type = Type::BYTE;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_bytes(self)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let buf: &[u8] = self;
        buf.iter().cloned().collect()
    }
}

impl TiffValue for [i8] {
    const BYTE_LEN: u8 = 1;
    const FIELD_TYPE: Type = Type::SBYTE;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        let slice = bytecast::i8_as_ne_bytes(self);
        writer.write_bytes(slice)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let buf: &[u8] = bytecast::i8_as_ne_bytes(self);
        buf.iter().cloned().collect()
    }
}

impl TiffValue for [u16] {
    const BYTE_LEN: u8 = 2;
    const FIELD_TYPE: Type = Type::SHORT;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        let slice = bytecast::u16_as_ne_bytes(self);
        writer.write_bytes(slice)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let buf: &[u8] = bytecast::u16_as_ne_bytes(self);
        buf.iter().cloned().collect()
    }
}

impl TiffValue for [i16] {
    const BYTE_LEN: u8 = 2;
    const FIELD_TYPE: Type = Type::SSHORT;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        let slice = bytecast::i16_as_ne_bytes(self);
        writer.write_bytes(slice)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let buf: &[u8] = bytecast::i16_as_ne_bytes(self);
        
        buf.iter().cloned().collect()
    }
}

impl TiffValue for [u32] {
    const BYTE_LEN: u8 = 4;
    const FIELD_TYPE: Type = Type::LONG;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        let slice = bytecast::u32_as_ne_bytes(self);
        writer.write_bytes(slice)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let buf: &[u8] = bytecast::u32_as_ne_bytes(self);
        buf.iter().cloned().collect()
    }
}

impl TiffValue for [i32] {
    const BYTE_LEN: u8 = 4;
    const FIELD_TYPE: Type = Type::SLONG;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        let slice = bytecast::i32_as_ne_bytes(self);
        writer.write_bytes(slice)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let buf: &[u8] = bytecast::i32_as_ne_bytes(self);
        buf.iter().cloned().collect()
    }
}

impl TiffValue for [u64] {
    const BYTE_LEN: u8 = 8;
    const FIELD_TYPE: Type = Type::LONG8;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        let slice = bytecast::u64_as_ne_bytes(self);
        writer.write_bytes(slice)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let buf: &[u8] = bytecast::u64_as_ne_bytes(self);
        buf.iter().cloned().collect()
    }
}

impl TiffValue for [i64] {
    const BYTE_LEN: u8 = 8;
    const FIELD_TYPE: Type = Type::SLONG8;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        let slice = bytecast::i64_as_ne_bytes(self);
        writer.write_bytes(slice)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let buf: &[u8] = bytecast::i64_as_ne_bytes(self);
        buf.iter().cloned().collect()
    }
}

impl TiffValue for [f32] {
    const BYTE_LEN: u8 = 4;
    const FIELD_TYPE: Type = Type::FLOAT;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        // We write using nativeedian so this sould be safe
        let slice = bytecast::f32_as_ne_bytes(self);
        writer.write_bytes(slice)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let buf: &[u8] = bytecast::f32_as_ne_bytes(self);
        buf.iter().cloned().collect()
    }
}

impl TiffValue for [f64] {
    const BYTE_LEN: u8 = 8;
    const FIELD_TYPE: Type = Type::DOUBLE;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        // We write using nativeedian so this sould be safe
        let slice = bytecast::f64_as_ne_bytes(self);
        writer.write_bytes(slice)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let buf: &[u8] = bytecast::f64_as_ne_bytes(self);
        buf.iter().cloned().collect()
    }
}

impl TiffValue for [Ifd] {
    const BYTE_LEN: u8 = 4;
    const FIELD_TYPE: Type = Type::IFD;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        for x in self {
            x.write(writer)?;
        }
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        for x in self {
            let mut bytes = x.serialize();
            buf.append(&mut bytes);
        }
        buf
    }
}

impl TiffValue for [Ifd8] {
    const BYTE_LEN: u8 = 8;
    const FIELD_TYPE: Type = Type::IFD8;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        for x in self {
            x.write(writer)?;
        }
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        for x in self {
            let mut bytes = x.serialize();
            buf.append(&mut bytes);
        }
        buf
    }
}

impl TiffValue for [Rational] {
    const BYTE_LEN: u8 = 8;
    const FIELD_TYPE: Type = Type::RATIONAL;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        for x in self {
            x.write(writer)?;
        }
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        for x in self {
            let mut bytes = x.serialize();
            buf.append(&mut bytes);
        }
        buf
    }
}

impl TiffValue for [SRational] {
    const BYTE_LEN: u8 = 8;
    const FIELD_TYPE: Type = Type::SRATIONAL;

    fn count(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        for x in self {
            x.write(writer)?;
        }
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        for x in self {
            let mut bytes = x.serialize();
            buf.append(&mut bytes);
        }
        buf
    }
}

impl TiffValue for u8 {
    const BYTE_LEN: u8 = 1;
    const FIELD_TYPE: Type = Type::BYTE;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_u8(*self)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl TiffValue for i8 {
    const BYTE_LEN: u8 = 1;
    const FIELD_TYPE: Type = Type::SBYTE;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_i8(*self)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        (*self).to_ne_bytes().to_vec()
    }
}

impl TiffValue for u16 {
    const BYTE_LEN: u8 = 2;
    const FIELD_TYPE: Type = Type::SHORT;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_u16(*self)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        (*self).to_ne_bytes().to_vec()
    }
}

impl TiffValue for i16 {
    const BYTE_LEN: u8 = 2;
    const FIELD_TYPE: Type = Type::SSHORT;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_i16(*self)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        (*self).to_ne_bytes().to_vec()
    }
}

impl TiffValue for u32 {
    const BYTE_LEN: u8 = 4;
    const FIELD_TYPE: Type = Type::LONG;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_u32(*self)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        (*self).to_ne_bytes().to_vec()
    }
}

impl TiffValue for i32 {
    const BYTE_LEN: u8 = 4;
    const FIELD_TYPE: Type = Type::SLONG;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_i32(*self)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        (*self).to_ne_bytes().to_vec()
    }
}

impl TiffValue for u64 {
    const BYTE_LEN: u8 = 8;
    const FIELD_TYPE: Type = Type::LONG8;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_u64(*self)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        (*self).to_ne_bytes().to_vec()
    }
}

impl TiffValue for i64 {
    const BYTE_LEN: u8 = 8;
    const FIELD_TYPE: Type = Type::SLONG8;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_i64(*self)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        (*self).to_ne_bytes().to_vec()
    }
}

impl TiffValue for f32 {
    const BYTE_LEN: u8 = 4;
    const FIELD_TYPE: Type = Type::FLOAT;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_f32(*self)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        (*self).to_ne_bytes().to_vec()
    }
}

impl TiffValue for f64 {
    const BYTE_LEN: u8 = 8;
    const FIELD_TYPE: Type = Type::DOUBLE;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_f64(*self)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        (*self).to_ne_bytes().to_vec()
    }
}

impl TiffValue for Ifd {
    const BYTE_LEN: u8 = 4;
    const FIELD_TYPE: Type = Type::IFD;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_u32(self.0)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let dword: [u8; 4] = self.0.to_ne_bytes();
        dword.to_vec()
    }
}

impl TiffValue for Ifd8 {
    const BYTE_LEN: u8 = 8;
    const FIELD_TYPE: Type = Type::IFD8;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_u64(self.0)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let qword: [u8; 8] = self.0.to_ne_bytes();
        qword.to_vec()
    }
}

impl TiffValue for Rational {
    const BYTE_LEN: u8 = 8;
    const FIELD_TYPE: Type = Type::RATIONAL;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_u32(self.n)?;
        writer.write_u32(self.d)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let first_dword: [u8; 4] = self.n.to_ne_bytes();
        let second_dword: [u8; 4] = self.d.to_ne_bytes();
        [first_dword, second_dword].concat()
    }
}

impl TiffValue for SRational {
    const BYTE_LEN: u8 = 8;
    const FIELD_TYPE: Type = Type::SRATIONAL;

    fn count(&self) -> usize {
        1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        writer.write_i32(self.n)?;
        writer.write_i32(self.d)?;
        Ok(())
    }

    fn serialize(&self) -> Vec<u8> {
        let first_dword: [u8; 4] = self.n.to_ne_bytes();
        let second_dword: [u8; 4] = self.d.to_ne_bytes();
        [first_dword, second_dword].concat()
    }
}

impl TiffValue for str {
    const BYTE_LEN: u8 = 1;
    const FIELD_TYPE: Type = Type::ASCII;

    fn count(&self) -> usize {
        self.len() + 1
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        if self.is_ascii() && !self.bytes().any(|b| b == 0) {
            writer.write_bytes(self.as_bytes())?;
            writer.write_u8(0)?;
            Ok(())
        } else {
            Err(TiffError::FormatError(TiffFormatError::InvalidTag))
        }
    }

    fn serialize(&self) -> Vec<u8> {
        if self.is_ascii() && !self.bytes().any(|b| b == 0) {
            let bytes: &[u8] = self.as_bytes();
            [bytes, &[0]].concat()
        } else {
            vec![]
        }
    }
}

impl<'a, T: TiffValue + ?Sized> TiffValue for &'a T {
    const BYTE_LEN: u8 = T::BYTE_LEN;
    const FIELD_TYPE: Type = T::FIELD_TYPE;

    fn count(&self) -> usize {
        (*self).count()
    }

    fn write<W: Write>(&self, writer: &mut TiffWriter<W>) -> TiffResult<()> {
        (*self).write(writer)
    }

    fn serialize(&self) -> Vec<u8> {
        (*self).serialize()
    }
}

/// Type to represent tiff values of type `IFD`
#[derive(Clone)]
pub struct Ifd(pub u32);

/// Type to represent tiff values of type `IFD8`
#[derive(Clone)]
pub struct Ifd8(pub u64);

/// Type to represent tiff values of type `RATIONAL`
#[derive(Clone)]
pub struct Rational {
    pub n: u32,
    pub d: u32,
}

/// Type to represent tiff values of type `SRATIONAL`
#[derive(Clone)]
pub struct SRational {
    pub n: i32,
    pub d: i32,
}
