//! `Poly::Write`, `Poly::Dump` and the `Poly::Read*` functions.
mod read;
mod tokens;
mod write;
pub use read::{read_polygon2d, read_polygon3d, read_triangulation};
pub use write::{
    dump_polygon2d, dump_polygon3d, dump_triangulation, write_polygon2d, write_polygon3d,
    write_triangulation,
};
