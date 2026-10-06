//! Private native packed-node methods exposed through a real Rust node representation.
#[path = "packed_map_node_contracts/construction.rs"]
mod construction;
#[path = "packed_map_node_contracts/cursors.rs"]
mod cursors;
#[path = "packed_map_node_contracts/fixture.rs"]
mod fixture;
#[path = "packed_map_node_contracts/mutations.rs"]
mod mutations;
#[path = "packed_map_node_contracts/ownership.rs"]
mod ownership;

macro_rules! node_width_tests {
    ($module:ident, $integer:ty) => {
        mod $module {
            #[test]
            fn constructors() {
                super::construction::constructors::<$integer>(stringify!($integer));
            }
            #[test]
            fn layout() {
                super::construction::layout::<$integer>(stringify!($integer));
            }
            #[test]
            fn mutations() {
                super::mutations::mutations::<$integer>(stringify!($integer));
            }
            #[test]
            fn cursors() {
                super::cursors::cursors::<$integer>(stringify!($integer));
            }
            #[test]
            fn every_bit() {
                super::cursors::one_bit::<$integer>(stringify!($integer));
            }
        }
    };
}
node_width_tests!(signed_32, i32);
node_width_tests!(unsigned_32, u32);
node_width_tests!(signed_64, i64);
node_width_tests!(unsigned_64, u64);
node_width_tests!(signed_byte, i8);
node_width_tests!(boolean, bool);
