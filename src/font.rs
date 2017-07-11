use primitives::{Tag, Ignored, Array};
use decode::{StaticEncodeSize, Decode, DecodeRead, DecodeWithRead};
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

#[derive(Decode, Debug)]
pub struct OffsetTable<'fnt> {
    sfnt_version: Version,
    num_tables: u16,
    search_range: Ignored<u16>,
    entry_selector: Ignored<u16>,
    range_shift: Ignored<u16>,
    #[WithParam = "num_tables as usize"]
    tables: Array<'fnt, TableRecord<'fnt>>,
}

#[derive(Decode, StaticEncodeSize, Debug, PartialEq)]
pub struct TableRecord<'fnt> {
    buffer: &'fnt [u8],
    pub tag: Tag,
    pub check_sum: u32,
    pub offset: u32,
    pub length: u32,
}

#[test]
fn try() {
    let data = open_file!("data/DroidSerif.ttf");
    let font = OffsetTable::decode(&data).expect("failed to read offset table");
    for tbl in font.tables {
        println!("{:?}", tbl);
    }

    panic!();
}