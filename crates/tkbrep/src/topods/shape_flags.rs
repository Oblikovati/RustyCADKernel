// Rust translation of the flag members of ModelingData/TKBRep/TopoDS/TopoDS_Shape.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The TShape flags seen through a shape (docs/topology-design.md section 4). Each
//! native member forwards to the TShape without a null check; the Rust members raise
//! Standard_NullObject for a null shape instead of dereferencing it.
use super::Shape;
use rustycad_tkernel::standard::Failure;

/// Declares a flag getter and setter pair forwarding to the TShape.
macro_rules! shape_flag {
    ($get:ident, $set:ident, $native:literal) => {
        #[doc = concat!("`", $native, "()` of the TShape; `shape.", stringify!($get), "()?`.")]
        pub fn $get(&self) -> Result<bool, Failure> {
            Ok(self
                .checked_tshape(concat!("TopoDS_Shape::", $native))?
                .$get())
        }
        #[doc = concat!("`", $native, "(bool)` of the shared TShape; `shape.", stringify!($set), "(true)?`.")]
        pub fn $set(&self, value: bool) -> Result<(), Failure> {
            self.checked_tshape(concat!("TopoDS_Shape::", $native))?
                .$set(value);
            Ok(())
        }
    };
}

impl Shape {
    shape_flag!(free, set_free, "Free");
    shape_flag!(locked, set_locked, "Locked");
    shape_flag!(modified, set_modified, "Modified");
    shape_flag!(checked, set_checked, "Checked");
    shape_flag!(orientable, set_orientable, "Orientable");
    shape_flag!(closed, set_closed, "Closed");
    shape_flag!(infinite, set_infinite, "Infinite");
    shape_flag!(convex, set_convex, "Convex");
}
