//! This module contains a list of font primitives.  Font primitives are data
//! types that are built into the OpenType/TrueType font specification.

use std::marker::PhantomData;
use std::fmt;

use error::{Error, Result};
use decode::{Decode, DecodeRead, DecodeWith, EncodeSize, StaticEncodeSize};
use byteorder::{BigEndian, ByteOrder};

/// A 32-bit signed fixed-point number: 16.16.
#[derive(Debug, Copy, Clone, PartialEq, Eq, From)]
pub struct Fixed(i32);

/// A signed 16-bit qunatity in font design units.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd,
         Ord, From, Add, AddAssign, Mul, MulAssign, Not)]
pub struct FWord(i16);

/// An unisgned 16-bit qunatity in font design units.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd,
         Ord, From, Add, AddAssign, Mul, MulAssign, Not)]
pub struct UFWord(u16);

/// A 16-bit signed fixed-point number: 2.14.
#[derive(Debug, Copy, Clone, PartialEq, Eq, From)]
pub struct F2Dot14(i16);

/// A 24-bit unsigned integer.
#[derive(Debug, Copy, Clone, PartialEq, Eq, From)]
pub struct Uint24(u32);

static_size!(Uint24 = 3);

impl<'fnt> Decode<'fnt> for Uint24 {
    fn decode(buffer: &[u8]) -> Result<Uint24> {
        required_len!(buffer, Uint24::size());
        Ok(Uint24(BigEndian::read_u24(buffer)))
    }
}

/// A 16.16 version containing a major (16-bit) and minor (16-bit).
//  TODO: This should consist of two 16-bit unsigned integers.
pub struct FixedVersion(Fixed);

/// An unsigned 64-bit date and time represented in the number of seconds
/// since midnight, January 1, 1904.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, From)]
pub struct LongDateTime(u64);

// Unfortunately byteorder doesn't provide `read_u8` or `read_i8`
// methods, so we must provide them ourselves.

fn read_u8(buffer: &[u8]) -> u8 {
    buffer[0]
}

fn read_i8(buffer: &[u8]) -> i8 {
    buffer[0] as i8
}

// NB: This should only be used for types for which their stack size
//     agrees with their decoded size.
macro_rules! impl_decode {
    ($($conv:expr => $type:tt),* $(,)*) => (
        $(
            impl StaticEncodeSize for $type {
                fn size() -> usize { ::std::mem::size_of::<$type>() }
            }

            impl<'fnt> Decode<'fnt> for $type {
                fn decode(buffer: &[u8]) -> Result<$type> {
                    required_len!(buffer, Self::size());
                    Ok($type::from($conv(buffer)))
                }
            }
        )*
    );
}

impl_decode!(
    BigEndian::read_i16 => FWord,
    BigEndian::read_u16 => UFWord,
    BigEndian::read_i16 => F2Dot14,
    BigEndian::read_i32 => Fixed,
    BigEndian::read_u64 => LongDateTime,

    read_u8             => u8,
    read_i8             => i8,
    BigEndian::read_u16 => u16,
    BigEndian::read_i16 => i16,
    BigEndian::read_u32 => u32,
    BigEndian::read_i32 => i32,
    BigEndian::read_i64 => i64,
);


impl From<Fixed> for f64 {
    fn from(fixed: Fixed) -> f64 {
        (fixed.0 as f64) / ((1i32 << 16) as f64)
    }
}

impl From<Fixed> for f32 {
    fn from(fixed: Fixed) -> f32 {
        (fixed.0 as f32) / ((1i32 << 16) as f32)
    }
}

impl From<F2Dot14> for f64 {
    fn from(fdot: F2Dot14) -> f64 {
        (fdot.0 as f64) / ((1i32 << 14) as f64)
    }
}

impl From<F2Dot14> for f32 {
    fn from(fdot: F2Dot14) -> f32 {
        (fdot.0 as f32) / ((1i32 << 14) as f32)
    }
}

/// An array of 4 bytes used to identify scripts, language systems, features,
/// baselines, and table names.  The bytes are either in Latin-1 or
/// treated as a 32-bit native endian indentifying integer.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Tag(pub(crate) [u8; 4]);

static_size!(Tag = 4);

impl<'fnt> Decode<'fnt> for Tag {
    fn decode(buffer: &'fnt [u8]) -> Result<Tag> {
        let tag = [
            buffer[0],
            buffer[1],
            buffer[2],
            buffer[3],
        ];

        Ok(Tag(tag))
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ::std::str;
        // Print the ASCII name if the name contains only
        // visible ASCII characters.  Otherwise Hex.
        if self.0.iter().all(|&c| c >= 32 && c <= 128) {
            let s = str::from_utf8(&self.0[..]).unwrap();
            f.debug_tuple("Tag")
                .field(&s)
                .finish()
        } else {
            let n = (self.0[0] as u32) << 24
                | (self.0[1] as u32) << 16
                | (self.0[2] as u32) << 8
                | (self.0[3] as u32);

            write!(f, "Tag(0x{:08X})", n)
        }
    }
}

/// A 16-bit unsigned integer, representing an offset to a table `T`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Offset16<'fnt, T: 'fnt> {
    pub(crate) buffer: &'fnt u8,
    pub(crate) table: PhantomData<T>,
}

/// A 32-bit unsigned integer, representing an offset to a table `T`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Offset32<'fnt, T: 'fnt> {
    pub(crate) buffer: &'fnt u8,
    pub(crate) table: PhantomData<T>,
}

impl<'fnt, T> fmt::Debug for Offset16<'fnt, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Offset16")
    }
}

impl<'fnt, T> fmt::Debug for Offset32<'fnt, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Offset32")
    }
}

// TODO: implement a reasonable Debug for `Offset*`.  This shoud look
//       something like `Offset<Type>`.

/// The `Ignored` type indicates that a type will not
/// be decoded, and instead skipped over.

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Ignored<T>(PhantomData<T>);

impl<'fnt, T> Decode<'fnt> for Ignored<T> {
    fn decode(_: &[u8]) -> Result<Self> {
        Ok(Ignored(PhantomData))
    }
}

impl<T> StaticEncodeSize for Ignored<T> where T: StaticEncodeSize {
    fn size() -> usize { T::size() }
}

impl<T> fmt::Debug for Ignored<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ignored")
    }
}

/// An `Array` data-type which represents a contiguous regoin of encoded `T`.
/// This type is often used for dynamic array sizes, whose sizes aren't knwon
/// until they are decoded (often by referencing a length attribute).  As such
/// `Array` implements `EncodeSize` but not `StaticEncodeSize`.  An `Array` also
/// requires `T` to have implement `StaticEncodeSize` to properly implement random
/// access.

#[derive(Copy, Clone)]
pub struct Array<'fnt, T> {
    buffer: &'fnt [u8],
    len: usize,
    _phantom: PhantomData<T>,
}

impl<'fnt, T> fmt::Debug for Array<'fnt, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Array")
    }
}


impl<'fnt, T> DecodeWith<'fnt, usize> for Array<'fnt, T> {
    fn decode_with(buffer: &'fnt [u8], param: usize) -> Result<Array<'fnt, T>> {
        Ok(Array {
            buffer,
            len: param,
            _phantom: PhantomData
        })
    }
}

impl<'fnt, T> EncodeSize for Array<'fnt, T> where T: StaticEncodeSize {
    fn encode_size(&self) -> usize {
        T::size() * self.len
    }
}

/// An iterator of type `T` constructed from an `Array<T>`.

pub struct ArrayIter<'fnt, T> {
    buffer: &'fnt [u8],
    len: usize,
    pos: usize,
    _phantom: PhantomData<T>,
}

impl<'fnt, T> Iterator for ArrayIter<'fnt, T>
where
    T: Decode<'fnt> + StaticEncodeSize
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.pos >= self.len {
            return None
        }

        self.pos += 1;
        self.buffer.decode_read::<T>().ok()
    }
}

impl<'fnt, T> IntoIterator for Array<'fnt, T>
where
    T: Decode<'fnt> + StaticEncodeSize
{
    type IntoIter = ArrayIter<'fnt, T>;
    type Item = T;

    fn into_iter(self) -> ArrayIter<'fnt, T> {
        ArrayIter {
            buffer: self.buffer,
            len: self.len,
            pos: 0,
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(overflowing_literals)]
    fn f2dot14_to_float() {
        assert_eq!(1.99993896484375,  f64::from(F2Dot14(0x7fff)));
        assert_eq!(1.75,              f64::from(F2Dot14(0x7000)));
        assert_eq!(0.00006103515625,  f64::from(F2Dot14(0x0001)));
        assert_eq!(0.0,               f64::from(F2Dot14(0x0000)));
        assert_eq!(-0.00006103515625, f64::from(F2Dot14(0xffff)));
        assert_eq!(-2.0,              f64::from(F2Dot14(0x8000)));

        assert_eq!(1.99993896484375,  f32::from(F2Dot14(0x7fff)));
        assert_eq!(1.75,              f32::from(F2Dot14(0x7000)));
        assert_eq!(0.00006103515625,  f32::from(F2Dot14(0x0001)));
        assert_eq!(0.0,               f32::from(F2Dot14(0x0000)));
        assert_eq!(-0.00006103515625, f32::from(F2Dot14(0xffff)));
        assert_eq!(-2.0,              f32::from(F2Dot14(0x8000)));
    }
}