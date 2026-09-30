// Rust translation of BRepBuilderAPI_MakeShape.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_MakeShape`: the root of every shape construction, which stores the
//! result and traces the history of the sub-shapes.
use super::command::BRepBuilderAPICommand;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use std::ops::{Deref, DerefMut};

/// The protected data of `BRepBuilderAPI_MakeShape` (docs/topalgo-design.md section 10):
/// the command, `myShape` and `myGenerated`. Every implementor of
/// [`BRepBuilderAPIMakeShape`] embeds one; it dereferences to the command.
///
/// ```
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase;
/// let base = BRepBuilderAPIMakeShapeBase::new();
/// assert!(!base.is_done());
/// assert!(base.my_shape.is_null());
/// ```
#[derive(Clone, Default)]
pub struct BRepBuilderAPIMakeShapeBase {
    command: BRepBuilderAPICommand,
    /// `myShape`: the shape built by the algorithm.
    pub my_shape: Shape,
    /// `myGenerated`: the list the history members return.
    pub my_generated: List<Shape>,
}

impl Deref for BRepBuilderAPIMakeShapeBase {
    type Target = BRepBuilderAPICommand;
    fn deref(&self) -> &BRepBuilderAPICommand {
        &self.command
    }
}

impl DerefMut for BRepBuilderAPIMakeShapeBase {
    fn deref_mut(&mut self) -> &mut BRepBuilderAPICommand {
        &mut self.command
    }
}

impl BRepBuilderAPIMakeShapeBase {
    /// `BRepBuilderAPI_MakeShape()` (protected): not done, a null shape and an empty
    /// list; `BRepBuilderAPIMakeShapeBase::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The base `Generated(S)` and `Modified(S)`: clears the list and returns it;
    /// `base.cleared_generated()`.
    pub fn cleared_generated(&mut self) -> &List<Shape> {
        self.my_generated.clear();
        &self.my_generated
    }
}

/// `BRepBuilderAPI_MakeShape` (docs/topalgo-design.md section 10): the virtual members
/// of the root class, with the native base bodies as provided methods. An algorithm
/// embeds a [`BRepBuilderAPIMakeShapeBase`], returns it from the two accessors and
/// overrides the members its native header overrides.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeVertex};
/// let mut maker = BRepBuilderAPIMakeVertex::new(Point3::new(1.0, 2.0, 3.0))?;
/// assert!(maker.is_done());
/// let vertex = maker.shape()?.clone();
/// assert!(maker.modified(&vertex)?.is_empty());
/// assert!(!maker.is_deleted(&vertex)?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait BRepBuilderAPIMakeShape {
    /// The embedded protected data.
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase;

    /// The embedded protected data, to write.
    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase;

    /// `IsDone()` (virtual in `BRepBuilderAPI_Command`): the done flag;
    /// `maker.is_done()`.
    fn is_done(&self) -> bool {
        self.make_shape_base().is_done()
    }

    /// `Check()` (not virtual; never overridden): StdFail_NotDone when the done flag is
    /// false, whatever the virtual `IsDone()` answers; `maker.check()?`.
    fn check(&self) -> Result<(), Failure> {
        self.make_shape_base().check()
    }

    /// `Build(theRange)`: called by `Shape()` when the algorithm is not done; does
    /// nothing in the base; `maker.build(&ProgressRange::default())?`.
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        Ok(())
    }

    /// `Shape()` and `operator TopoDS_Shape()`: builds and checks the algorithm when
    /// `IsDone()` is false, then returns the shape; StdFail_NotDone when it is still
    /// not done; `maker.shape()?`.
    fn shape(&mut self) -> Result<&Shape, Failure> {
        if !self.is_done() {
            self.build(&ProgressRange::default())?;
            self.check()?;
        }
        Ok(&self.make_shape_base().my_shape)
    }

    /// `Generated(S)`: the shapes generated from `S` (none in the base, whose body
    /// clears the list); `maker.generated(&shape)?`.
    fn generated(&mut self, _shape: &Shape) -> Result<&List<Shape>, Failure> {
        Ok(self.make_shape_base_mut().cleared_generated())
    }

    /// `Modified(S)`: the shapes `S` was modified into (none in the base, whose body
    /// clears the list); `maker.modified(&shape)?`.
    fn modified(&mut self, _shape: &Shape) -> Result<&List<Shape>, Failure> {
        Ok(self.make_shape_base_mut().cleared_generated())
    }

    /// `IsDeleted(S)`: whether `S` was deleted (false in the base);
    /// `maker.is_deleted(&shape)?`.
    fn is_deleted(&mut self, _shape: &Shape) -> Result<bool, Failure> {
        Ok(false)
    }
}
