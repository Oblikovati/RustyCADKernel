// Rust translation of the step every constructor, Init and Add of the BRepBuilderAPI
// makers repeats (BRepBuilderAPI_Make*.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shared step of the wrapping makers (docs/topalgo-design.md section 10): after the
//! BRepLib member runs, a done BRepLib maker marks the command done and gives its shape.
use super::make_shape::BRepBuilderAPIMakeShapeBase;
use crate::brep_lib::BRepLibMakeShape;
use rustycad_tkernel::standard::Failure;

/// `if (myMakeX.IsDone()) { Done(); myShape = myMakeX.Shape(); }`: a maker that is not
/// done leaves the command and the shape as they were.
pub(super) fn adopt(
    base: &mut BRepBuilderAPIMakeShapeBase,
    lib: &BRepLibMakeShape,
) -> Result<(), Failure> {
    if lib.is_done() {
        base.set_done();
        base.my_shape = lib.shape()?.clone();
    }
    Ok(())
}

/// The constructors that build the BRepLib maker of the same Rust name and wrap it: each
/// entry is `name(args) = "native signature";`. The maker provides `wrap`.
macro_rules! wrapped_constructors {
    ($owner:ident, $lib:ident; $($name:ident($($arg:ident: $ty:ty),*) = $native:literal;)*) => {
        $(
            #[doc = concat!("`", $native, "`: wraps [`", stringify!($lib), "::", stringify!($name),
                "`] and takes its shape when it is done; `", stringify!($owner), "::",
                stringify!($name), "(..)?`.")]
            pub fn $name($($arg: $ty),*) -> Result<Self, Failure> {
                Self::wrap($lib::$name($($arg),*)?)
            }
        )*
    };
}

/// The members (Init, Add) that run the BRepLib member of the same Rust name on the
/// wrapped maker, then take its shape when it is done: each entry is
/// `name(args) = "native signature";`. The maker provides `adopt`.
macro_rules! wrapped_updates {
    ($owner:ident, $field:ident; $($name:ident($($arg:ident: $ty:ty),*) = $native:literal;)*) => {
        $(
            #[doc = concat!("`", $native, "`: runs the BRepLib member of the same name on the wrapped maker",
                " and takes its shape when it is done; `maker.", stringify!($name), "(..)?`.")]
            pub fn $name(&mut self, $($arg: $ty),*) -> Result<(), Failure> {
                self.$field.$name($($arg),*)?;
                self.adopt()
            }
        )*
    };
}

pub(super) use {wrapped_constructors, wrapped_updates};
