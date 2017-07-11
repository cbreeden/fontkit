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

macro_rules! open_file {
    ($name:expr) => ({
        use std::fs::File;
        use std::io::BufReader;
        use std::io::prelude::*;

        let file = File::open($name).expect("unable to open file");

        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data).expect("error reading font");

        data
    })
}