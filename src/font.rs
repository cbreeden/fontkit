use primitives::Tag;
use decode::{StaticSize, Decode};
use error::{Error, Result};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Version {
    OpenType,
    TrueType,
}

static_size!(Version = 4);
impl<'fnt> Decode<'fnt> for Version {
    fn decode(buffer: &[u8]) -> Result<Version> {
        const VERSION1: [u8; 4] = [0x00, 0x01, 0x00, 0x00];
        let tag = Tag::decode(buffer)?;
        match &tag.0 {
            b"OTTO" => Ok(Version::OpenType),
            &VERSION1 | b"true" | b"typ1" => Ok(Version::TrueType),
            b"ttcf" => Err(Error::TtcfUnsupported),
            _ => Err(Error::InvalidData),
        }
    }
}