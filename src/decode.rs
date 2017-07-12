//! This module contains the traits used for the decoding font files.
//! This includes `primitive` types and `table` types.
//! This module also provides a wrapper around the `byteorder` create,
//! since every datatype found in fonts are `BigEndian`.

use error::Result;

/// Types whose sizes are statically known should implement this trait.
/// It's important to note that `size` refers to the encoding size in
/// a font file, and not the stack or heap size of the type itself.
/// For instance, a type `U24(u32)` to represent a 24-bit unsized integer
/// has a stack size of 4-bytes, but should implement a
/// `StaticSize::size() == 4`.

pub trait EncodeSize {
    /// The size in bytes of the _decoded_ type.
    fn encode_size(&self) -> usize;
}

pub trait StaticEncodeSize {
    fn size() -> usize;
}

impl<T: StaticEncodeSize> EncodeSize for T {
    fn encode_size(&self) -> usize {
        Self::size()
    }
}

/// The `Decode` trait provides the logic for taking a slice of bytes
/// in a font file and decoding them into a rust type.

pub trait Decode<'fnt>: Sized {
    fn decode(&'fnt [u8]) -> Result<Self>;
}

/// The `DecodeRead` trait provides a `Read`-like interface
/// to decoding a type.  This trait is automatically implemented
/// for types that implement `Decode` and `StaticSize` automatically.

pub trait DecodeRead<'fnt>: Sized {
    fn decode_read<T: Decode<'fnt> + EncodeSize>(&mut self) -> Result<T>;
}

impl<'b: 'fnt, 'fnt> DecodeRead<'fnt> for &'b [u8] {
    #[inline]
    fn decode_read<T: Decode<'fnt> + EncodeSize>(&mut self) -> Result<T> {
        let ret = T::decode(self)?;
        *self = &self[ret.encode_size()..];
        Ok(ret)
    }
}

/// Some tables require offsets to be relative to a parent table.
/// For these situations, the `DecodeWith<Param>` trait provides the
/// same interface as `Decode` except that it provides a parameter
/// which can be used in the implementation.

// TODO: Investigate if P should be an associated type if the ATC RFC merges.

pub trait Decode1<'fnt, P>: Sized {
    fn decode(&'fnt [u8], param: P) -> Result<Self>;
}

/// The `DecodeWithRead` trait provides a `Read`-like interface
/// to decoding a type.  This trait is automatically implemented
/// for types that implement `DecodeWith` and `StaticSize` automatically.

pub trait DecodeRead1<'fnt, P>: Sized {
    fn decode_read1<T>(&mut self, param: P) -> Result<T>
        where T: Decode1<'fnt, P> + EncodeSize;
}

impl<'b: 'fnt, 'fnt, P> DecodeRead1<'fnt, P> for &'b [u8] {
    #[inline]
    fn decode_read1<T>(&mut self, param: P) -> Result<T>
        where T: Decode1<'fnt, P> + EncodeSize
    {
        let ret = T::decode(self, param)?;
        *self = &self[ret.encode_size()..];
        Ok(ret)
    }
}