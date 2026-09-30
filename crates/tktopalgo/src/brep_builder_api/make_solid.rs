// Rust translation of BRepBuilderAPI_MakeSolid.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_MakeSolid`: a solid from shells, a compsolid or another solid, built
//! by the wrapped `BRepLib_MakeSolid`.
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::wrap::{self, wrapped_constructors, wrapped_updates};
use crate::brep_lib::{BRepLibMakeSolid, BRepLibShapeModification};
use rustycad_tkbrep::topods::{self, CompSolid, Shape, Shell, Solid};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `BRepBuilderAPI_MakeSolid` (docs/topalgo-design.md section 10): each constructor and
/// Add runs the `BRepLib_MakeSolid` member of the same Rust name; `IsDone` is the flag of
/// the BRepLib maker and `IsDeleted` reports the faces a compsolid merged away.
///
/// The empty constructor leaves the command not done and the shape null while the
/// wrapped maker is done (natively too), so `shape()` returns the null shape until the
/// first Add.
///
/// ```
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeSolid};
/// let mut maker = BRepBuilderAPIMakeSolid::new()?;
/// assert!(maker.is_done());
/// assert!(maker.shape()?.is_null());
/// assert!(!maker.solid()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepBuilderAPIMakeSolid {
    base: BRepBuilderAPIMakeShapeBase,
    my_make_solid: BRepLibMakeSolid,
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPIMakeSolid {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `IsDone()`: the flag of the BRepLib maker; `maker.is_done()`.
    fn is_done(&self) -> bool {
        self.my_make_solid.is_done()
    }

    /// `IsDeleted(S)`: true for a face whose BRepLib status is `BRepLib_Deleted`;
    /// `maker.is_deleted(&face)?`.
    fn is_deleted(&mut self, shape: &Shape) -> Result<bool, Failure> {
        if shape.shape_type()? != ShapeEnum::Face {
            return Ok(false);
        }
        let status = self.my_make_solid.face_status(&topods::face(shape)?);
        Ok(status == BRepLibShapeModification::Deleted)
    }
}

impl BRepBuilderAPIMakeSolid {
    /// `BRepBuilderAPI_MakeSolid()`: the BRepLib maker of an empty solid, without taking
    /// its shape (see the type documentation); `BRepBuilderAPIMakeSolid::new()?`.
    pub fn new() -> Result<Self, Failure> {
        Ok(Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_solid: BRepLibMakeSolid::new()?,
        })
    }

    /// Wraps a constructed BRepLib maker and takes its shape when it is done.
    fn wrap(my_make_solid: BRepLibMakeSolid) -> Result<Self, Failure> {
        let mut maker = Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_solid,
        };
        maker.adopt()?;
        Ok(maker)
    }

    /// The step after each BRepLib call: done and the shape when the BRepLib maker is.
    fn adopt(&mut self) -> Result<(), Failure> {
        wrap::adopt(&mut self.base, &self.my_make_solid)
    }

    wrapped_constructors!(BRepBuilderAPIMakeSolid, BRepLibMakeSolid;
        from_comp_solid(comp_solid: &CompSolid) = "BRepBuilderAPI_MakeSolid(const TopoDS_CompSolid& S)";
        from_shell(shell: &Shell) = "BRepBuilderAPI_MakeSolid(const TopoDS_Shell& S)";
        from_two_shells(s1: &Shell, s2: &Shell) = "BRepBuilderAPI_MakeSolid(S1, S2)";
        from_three_shells(s1: &Shell, s2: &Shell, s3: &Shell) = "BRepBuilderAPI_MakeSolid(S1, S2, S3)";
        from_solid(solid: &Solid) = "BRepBuilderAPI_MakeSolid(const TopoDS_Solid& So)";
        from_solid_shell(solid: &Solid, shell: &Shell) = "BRepBuilderAPI_MakeSolid(So, S)";
    );

    wrapped_updates!(BRepBuilderAPIMakeSolid, my_make_solid;
        add(shell: &Shell) = "Add(S)";
    );

    /// `Solid()` and `operator TopoDS_Solid()`: the solid of the BRepLib maker;
    /// `maker.solid()?`.
    pub fn solid(&self) -> Result<Solid, Failure> {
        self.my_make_solid.solid()
    }
}
