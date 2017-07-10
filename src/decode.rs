//! This module contains the traits used for the decoding font files.
//! This includes `primitive` types and `table` types.
//! This module also provides a wrapper around the `byteorder` create,
//! since every datatype found in fonts are `BigEndian`.

use error::{Error, Result};

/// Types whose sizes are statically known should implement this trait.
/// It's important to note that `size` refers to the encoding size in
/// a font file, and not the stack or heap size of the type itself.
/// For instance, a type `U24(u32)` to represent a 24-bit unsized integer
/// has a stack size of 4-bytes, but should implement a
/// `StaticSize::size() == 4`.

pub trait StaticSize {
    /// The size in bytes of the _decoded_ type.
    fn size() -> usize;
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
    fn read<T: Decode<'fnt> + StaticSize>(&mut self) -> Result<T>;
}

impl<'b: 'fnt, 'fnt> DecodeRead<'fnt> for &'b [u8] {
    fn read<T: Decode<'fnt> + StaticSize>(&mut self) -> Result<T> {
        if self.len() < T::size() {
            return Err(Error::UnexpectedEof);
        }

        let ret = T::decode(self);
        *self = &self[T::size()..];
        ret
    }
}

/// Some tables require offsets to be relative to a parent table.
/// For these situations, the `DecodeWith<Param>` trait provides the
/// same interface as `Decode` except that it provides a parameter
/// which can be used in the implementation.

// TODO: Investigate if P should be an associated type if the ATC RFC merges.

pub trait DecodeWith<'fnt, P>: Sized {
    fn decode_with(&'fnt [u8], param: P) -> Result<Self>;
}

/// The `DecodeWithRead` trait provides a `Read`-like interface
/// to decoding a type.  This trait is automatically implemented
/// for types that implement `DecodeWith` and `StaticSize` automatically.

pub trait DecodeWithRead<'fnt, P>: Sized {
    fn read_with<T>(&mut self, param: P) -> Result<T> where T: DecodeWith<'fnt, P> + StaticSize;
}

impl<'b: 'fnt, 'fnt, P> DecodeWithRead<'fnt, P> for &'b [u8] {
    fn read_with<T>(&mut self, param: P) -> Result<T>
        where T: DecodeWith<'fnt, P> + StaticSize
    {
        if self.len() < T::size() {
            return Err(Error::UnexpectedEof);
        }

        let ret = T::decode_with(self, param);
        *self = &self[T::size()..];
        ret
    }
}