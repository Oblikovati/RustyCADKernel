// Rust translation of BRepPrimAPI_MakeOneAxis.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrimAPI_MakeOneAxis`: the root of the makers of rotational primitives.
use crate::brep_prim::BRepPrimOneAxis;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Face, Shape, Shell, Solid};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};

/// The first half of the native pattern `B.MakeSolid(TopoDS::Solid(myShape));
/// B.Add(myShape, shell); Done();` of the solid makers: a new empty solid replaces the
/// shape (Standard_TypeMismatch when the shape is of another type). The shell is asked
/// for after it, so a failing shell leaves the empty solid, as natively.
pub(crate) fn reset_solid(base: &mut BRepBuilderAPIMakeShapeBase) -> Result<(), Failure> {
    let mut solid = topods::solid(&base.my_shape)?;
    BRepBuilder::default().make_solid(&mut solid);
    base.my_shape = solid.into();
    Ok(())
}

/// The second half of the pattern: adds `shell` to the solid and marks the maker done.
pub(crate) fn add_shell_done(
    base: &mut BRepBuilderAPIMakeShapeBase,
    shell: &Shell,
) -> Result<(), Failure> {
    BRepBuilder::default().add(&mut base.my_shape, shell)?;
    base.set_done();
    Ok(())
}

/// `BRepPrimAPI_MakeOneAxis::Build()`: a new solid holding the shell of the one-axis
/// algorithm becomes the shape; `one_axis_build(&mut maker)?`.
pub fn one_axis_build<T: BRepPrimAPIMakeOneAxis + ?Sized>(maker: &mut T) -> Result<(), Failure> {
    reset_solid(maker.make_shape_base_mut())?;
    let shell = maker.one_axis().shell()?;
    add_shell_done(maker.make_shape_base_mut(), &shell)
}

/// `BRepPrimAPI_MakeOneAxis` (docs/prim-design.md section 4): a maker of a rotational
/// primitive, which gives its `BRepPrim_OneAxis` algorithm (the native `void*
/// OneAxis()`); its `Build` is [`one_axis_build`].
///
/// ```
/// use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis};
/// let mut maker = BRepPrimAPIMakeCylinder::new(1.0, 2.0);
/// assert!(!maker.face()?.is_null());
/// assert!(!maker.solid()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait BRepPrimAPIMakeOneAxis: BRepBuilderAPIMakeShape {
    /// `OneAxis()`: the one-axis algorithm of the maker.
    fn one_axis(&mut self) -> &mut dyn BRepPrimOneAxis;

    /// `Face()` and `operator TopoDS_Face()`: builds, then returns the lateral face;
    /// `maker.face()?`.
    fn face(&mut self) -> Result<Face, Failure> {
        self.build(&ProgressRange::default())?;
        self.one_axis().lateral_face()
    }

    /// `Shell()` and `operator TopoDS_Shell()`: builds, then returns the shell;
    /// `maker.shell()?`.
    fn shell(&mut self) -> Result<Shell, Failure> {
        self.build(&ProgressRange::default())?;
        self.one_axis().shell()
    }

    /// `Solid()` and `operator TopoDS_Solid()`: builds, then returns the solid;
    /// `maker.solid()?`.
    fn solid(&mut self) -> Result<Solid, Failure> {
        self.build(&ProgressRange::default())?;
        let shape: &Shape = &self.make_shape_base().my_shape;
        topods::solid(shape)
    }
}

/// Implements `BRepBuilderAPIMakeShape` (with `Build` = [`one_axis_build`]) and
/// [`BRepPrimAPIMakeOneAxis`] for a one-axis maker with the fields `base` and `$algo`.
macro_rules! one_axis_maker {
    ($maker:ty, $algo:ident) => {
        impl rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape for $maker {
            fn make_shape_base(
                &self,
            ) -> &rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase {
                &self.base
            }
            fn make_shape_base_mut(
                &mut self,
            ) -> &mut rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase {
                &mut self.base
            }
            /// `BRepPrimAPI_MakeOneAxis::Build()`.
            fn build(
                &mut self,
                _range: &rustycad_tkernel::message::ProgressRange<'_>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                $crate::brep_prim_api::one_axis_build(self)
            }
        }
        impl $crate::brep_prim_api::BRepPrimAPIMakeOneAxis for $maker {
            fn one_axis(&mut self) -> &mut dyn $crate::brep_prim::BRepPrimOneAxis {
                &mut self.$algo
            }
        }
    };
}
pub(crate) use one_axis_maker;
