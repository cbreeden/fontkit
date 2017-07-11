use decode::{Decode, StaticEncodeSize, DecodeRead};
use primitives::Fixed;
use error::{Error, Result};

pub enum Maxp {
    Version05(Version05),
    Version1(Version1),
}

impl<'fnt> Decode<'fnt> for Maxp {
    fn decode(buffer: &'fnt [u8]) -> Result<Maxp> {
        let version = u32::decode(buffer)?;
        match version {
            0x00005000 => Ok(Maxp::Version05(Version05::decode(buffer)?)),
            0x00010000 => Ok(Maxp::Version1(Version1::decode(buffer)?)),
            _ => Err(Error::UnsupportedVersion),
        }
    }
}

impl Maxp {
    pub fn get_num_glyphs(&self) -> u16 {
        match *self {
            Maxp::Version05(ref t) => t.num_glyphs,
            Maxp::Version1(ref t) => t.num_glyphs,
        }
    }
}

#[derive(Decode, StaticEncodeSize, Debug, PartialEq)]
pub struct Version05 {
    pub version: Fixed,
    pub num_glyphs: u16,
}

#[derive(Decode, StaticEncodeSize, Debug, PartialEq)]
pub struct Version1 {
    pub version: Fixed,
    pub num_glyphs: u16,
    pub max_points: u16,
    pub max_contours: u16,
    pub max_composite_points: u16,
    pub max_composite_contours: u16,
    pub max_zones: u16,
    pub max_twilight_points: u16,
    pub max_storage: u16,
    pub max_function_defs: u16,
    pub max_instruction_defs: u16,
    pub max_stack_elements: u16,
    pub max_size_of_instructions: u16,
    pub max_component_elements: u16,
    pub max_component_depth: u16,    
}