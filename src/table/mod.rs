// pub mod maxp;

use primitives::Tag;

/// Tagged tables are tables that are accessed from the Font.
pub trait TaggedTable<'tbl> {
    fn tag() -> Tag;
}

macro_rules! impl_tagged_table {
    ($($name:ty => $tag:expr),* $(,)*) => (
        $(
        impl<'tbl> TaggedTable<'tbl> for $name {
            fn tag() -> Tag {
                Tag($tag)
            }
        }
        )*
    )
}