macro_rules! static_size {
    ($($type:ty = $size:expr),* $(,)*) => (
        $(
        impl StaticEncodeSize for $type {
            fn size() -> usize {
                $size
            }
        }
        )*
    )
}

macro_rules! required_len {
    ($buffer:expr, $len:expr) => (
        if $buffer.len() < $len {
            return Err(Error::UnexpectedEof)
        }
    )
}

#[cfg(test)]
macro_rules! open_file {
    ($name:expr) => ({
        use std::fs::File;
        use std::io::BufReader;
        use std::io::prelude::*;

        let file = File::open($name).expect("unable to open file");

        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data).expect("error reading file");

        data
    })
}

// macro_rules! versioned_table {
//     (@match $var:expr, $($dst:ty = $tag:expr;)*) => {
//         match $var {
//             $(
//                 $tag => { panic!(stringify!($dst)) },
//             )*
//             _ => panic!("No match"),
//         }
//     };
//
//     (@match $($tt:tt)*) => {
//         panic!(stringify!($($tt)*));
//     };
//
//     ($name:ty, $ty:ty => $($tt:tt)*) => {
//         impl<'fnt> Decode<'fnt> for $name {
//             fn decode(buffer: &'fnt [u8]) -> Result<Self> {
//                 let tag = <$ty as Decode> :: decode(buffer)?;
//
//                 //panic!(stringify!(@match $bl));
//                 versioned_table!(@match tag, $($tt)*);
//                 unimplemented!()
//             }
//         }
//     };
// }
//
// versioned_table! {Maxp,
//     u32 =>
//         Version05 = 0x00005000;
//         Version1  = 0x00010000;
// }