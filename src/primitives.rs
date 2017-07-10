//! This module contains a list of font primitives.  Font primitives are data
//! types that are built into the OpenType/TrueType font specification.

use std::marker::PhantomData;
use std::fmt;

use error::Result;
use decode::{Decode, StaticSize};
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

/// A 16.16 version containing a major (16-bit) and minor (16-bit).
/// TODO: This should consist of two 16-bit unsigned integers.
pub struct FixedVersion(Fixed);

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
            impl StaticSize for $type {
                fn size() -> usize { ::std::mem::size_of::<$type>() }
            }

            impl<'fnt> Decode<'fnt> for $type {
                fn decode(buffer: &[u8]) -> Result<$type> {
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

/// An unsigned 64-bit date and time represented in the number of seconds
/// since midnight, January 1, 1904.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LongDateTime(u64);

/// An array of 4 bytes used to identify scripts, language systems, features,
/// baselines, and table names.  The bytes are either in Latin-1 or
/// treated as a 32-bit native endian indentifying integer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Tag(u32);

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