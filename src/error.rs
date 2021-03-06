use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    UnexpectedEof,
    InvalidData,
    UnsupportedCmapFormat,
    UnsupportedVersion,
    TtcfUnsupported,
}