macro_rules! static_size {
    ($($type:ty = $size:expr),* $(,)*) => (
        $(
        impl StaticSize for $type {
            fn size() -> usize {
                $size
            }
        }
        )*
    )
}