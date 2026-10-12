//! Native open/closed shaded grouping walks located topology and ignores one-dimensional leaves.
use super::{StdPrsShadedShape, StdPrsToolTriangulatedShape};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Compound, Shape, ShapeIterator},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Native grouping interpretation; `StdPrsVolume::Autodetection` separates solid shells.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StdPrsVolume {
    /// Native default: closed solid shells and opened faces form separate groups.
    Autodetection = 0,
    /// Caller explicitly declares a closed volume.
    Closed = 1,
    /// Caller explicitly declares an opened volume.
    Opened = 2,
}
impl StdPrsShadedShape {
    /// Append actual subshapes to the provided compounds; `Self::explore_solids(&shape,&builder,&mut closed,&mut opened,true)?`.
    pub fn explore_solids(
        shape: &Shape,
        builder: &BRepBuilder,
        closed: &mut Compound,
        opened: &mut Compound,
        ignore_1d: bool,
    ) -> Result<(), Failure> {
        if shape.is_null() {
            return Ok(());
        }
        explore_shaded_volume_kind(shape, builder, closed, opened, ignore_1d)
    }
}
fn append_shaded_compound_children(
    shape: &Shape,
    builder: &BRepBuilder,
    closed: &mut Compound,
    opened: &mut Compound,
    ignore_1d: bool,
) -> Result<(), Failure> {
    for child in ShapeIterator::new(shape, true, true) {
        StdPrsShadedShape::explore_solids(&child, builder, closed, opened, ignore_1d)?;
    }
    Ok(())
}
fn append_shaded_solid_children(
    shape: &Shape,
    builder: &BRepBuilder,
    closed: &mut Compound,
    opened: &mut Compound,
) -> Result<(), Failure> {
    for child in ShapeIterator::new(shape, true, true) {
        let closed_child = shaded_shell_is_closed(&child)?;
        builder.add(
            if closed_child {
                &mut *closed
            } else {
                &mut *opened
            },
            &child,
        )?;
    }
    Ok(())
}

fn explore_shaded_volume_kind(
    shape: &Shape,
    builder: &BRepBuilder,
    closed: &mut Compound,
    opened: &mut Compound,
    ignore_1d: bool,
) -> Result<(), Failure> {
    match shape.shape_type()? {
        ShapeEnum::Compound | ShapeEnum::CompSolid => {
            append_shaded_compound_children(shape, builder, closed, opened, ignore_1d)
        }
        ShapeEnum::Solid => append_shaded_solid_children(shape, builder, closed, opened),
        ShapeEnum::Shell | ShapeEnum::Face => builder.add(opened, shape),
        ShapeEnum::Wire | ShapeEnum::Edge | ShapeEnum::Vertex if !ignore_1d => {
            builder.add(opened, shape)
        }
        _ => Ok(()),
    }
}

fn shaded_shell_is_closed(shape: &Shape) -> Result<bool, Failure> {
    Ok(shape.shape_type()? == ShapeEnum::Shell
        && BRepTool::is_closed(shape)?
        && StdPrsToolTriangulatedShape::is_triangulated(shape)?)
}
