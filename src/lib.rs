#[macro_use]
extern crate derive_more;
extern crate byteorder;
#[macro_use]
extern crate decode_derive;

#[macro_use]
mod macros;
pub mod table;
pub mod primitives;
pub mod error;
pub mod decode;
pub mod font;