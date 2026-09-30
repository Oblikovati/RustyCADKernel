// Rust translation of ModelingData/TKBRep/TopoDS/TopoDS_TVertex, TopoDS_TEdge,
// TopoDS_TWire, TopoDS_TFace, TopoDS_TShell, TopoDS_TSolid, TopoDS_TCompSolid and
// TopoDS_TCompound (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The TShape classes of the eight shape types (docs/topology-design.md section 2).
use super::tshape::{TShape, TShapeBase};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Declares a concrete TShape class: its base data, its descriptor, a constructor
/// that applies the native constructor flags, and `EmptyCopy` returning a new
/// instance of the same class, as each native `EmptyCopy` does.
macro_rules! concrete_tshape {
    ($(#[$doc:meta])* $class:ident, $native:literal, $kind:expr, |$base:ident| $defaults:block) => {
        $(#[$doc])*
        pub struct $class {
            transient: StandardTransient,
            tshape: TShapeBase,
        }
        impl_transient!($class, $native, dyn TShape, transient, interfaces: [dyn TShape]);
        impl $class {
            /// The native constructor: the type and its default flags.
            pub fn new() -> Self {
                let $base = TShapeBase::new($kind);
                $defaults
                Self {
                    transient: StandardTransient::default(),
                    tshape: $base,
                }
            }
        }
        impl Default for $class {
            fn default() -> Self {
                Self::new()
            }
        }
        impl TShape for $class {
            fn tshape_base(&self) -> &TShapeBase {
                &self.tshape
            }
            fn empty_copy(&self) -> Result<Handle<dyn TShape>, Failure> {
                Ok(Handle::new(Self::new()).into_cast())
            }
        }
    };
}

concrete_tshape!(
    /// `TopoDS_TEdge`: its constructor is protected natively (BRep_TEdge derives from
    /// it), but its `EmptyCopy` returns a plain `TopoDS_TEdge`; `TEdge::new()`.
    TEdge, "TopoDS_TEdge", ShapeEnum::Edge, |base| {}
);
concrete_tshape!(
    /// `TopoDS_TWire`: an empty wire; `TWire::new()`.
    TWire, "TopoDS_TWire", ShapeEnum::Wire, |base| {}
);
concrete_tshape!(
    /// `TopoDS_TFace`: a face without geometry; `TFace::new()`.
    TFace, "TopoDS_TFace", ShapeEnum::Face, |base| {}
);
concrete_tshape!(
    /// `TopoDS_TShell`: an empty shell; `TShell::new()`.
    TShell, "TopoDS_TShell", ShapeEnum::Shell, |base| {}
);
concrete_tshape!(
    /// `TopoDS_TSolid`: an empty solid, created not orientable; `TSolid::new()`.
    TSolid, "TopoDS_TSolid", ShapeEnum::Solid, |base| {
        base.set_bit(super::bit_layout::BIT_ORIENTABLE, false);
    }
);
concrete_tshape!(
    /// `TopoDS_TCompSolid`: an empty composite solid; `TCompSolid::new()`.
    TCompSolid, "TopoDS_TCompSolid", ShapeEnum::CompSolid, |base| {}
);
concrete_tshape!(
    /// `TopoDS_TCompound`: an empty compound, created not orientable; `TCompound::new()`.
    TCompound, "TopoDS_TCompound", ShapeEnum::Compound, |base| {
        base.set_bit(super::bit_layout::BIT_ORIENTABLE, false);
    }
);

/// `TopoDS_TVertex`: the abstract vertex TShape. It has no `EmptyCopy`, so it is not a
/// [`TShape`] by itself: a derived class (BRep_TVertex, or a test fake) embeds it as
/// its parent field, implements `TShape` with [`TVertex::tshape_base`] and supplies
/// `empty_copy`. Its constructor makes the vertex Closed and Convex.
///
/// ```
/// use rustycad_tkbrep::topods::TVertex;
/// let vertex = TVertex::new();
/// assert_eq!(vertex.tshape_base().state() & 0x0500, 0x0500);
/// ```
pub struct TVertex {
    transient: StandardTransient,
    tshape: TShapeBase,
}
impl_transient!(TVertex, "TopoDS_TVertex", dyn TShape, transient);

impl TVertex {
    /// The protected `TopoDS_TVertex()`, for derived classes; `TVertex::new()`.
    pub fn new() -> Self {
        let tshape = TShapeBase::new(ShapeEnum::Vertex);
        tshape.set_bit(super::bit_layout::BIT_CLOSED, true);
        tshape.set_bit(super::bit_layout::BIT_CONVEX, true);
        Self {
            transient: StandardTransient::default(),
            tshape,
        }
    }

    /// The embedded `TopoDS_TShape` data, which a derived class returns from
    /// `TShape::tshape_base`; `vertex.tshape_base()`.
    pub fn tshape_base(&self) -> &TShapeBase {
        &self.tshape
    }
}

impl Default for TVertex {
    fn default() -> Self {
        Self::new()
    }
}
