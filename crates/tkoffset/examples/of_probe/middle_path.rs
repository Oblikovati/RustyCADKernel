//! BRepOffsetAPI_MiddlePath probe operations, the Rust twin of
//! tools/oracle/tkoffset_probe_brepoffsetapi_middle_path.cxx (the records are documented
//! there).
use super::common::{Reader, attempt, end, flag};
use super::make_offset::{at, failed};
use super::shape::shape;
use super::shapes::tp_shapes;
use super::tool::{primitive, sub};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkoffset::brep_offset_api::{BRepOffsetAPIMakePipe, BRepOffsetAPIMiddlePath};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSweep;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

/// The outer wire of a face as a shape.
fn outer(face: &Shape) -> Result<Shape, Failure> {
    Ok(brep_tools::outer_wire(&topods::face(face)?)?.into_shape())
}

/// The middle path of a shape between two shapes (`of_middle_path`).
fn middle_path(shape_of: &Shape, first: &Shape, last: &Shape) {
    let mut path = match BRepOffsetAPIMiddlePath::new(shape_of, first, last) {
        Ok(path) => path,
        Err(error) => return failed(error),
    };
    attempt(|| path.build(&ProgressRange::default()));
    flag(path.is_done());
    if path.is_done() {
        attempt(|| path.shape().and_then(shape));
    }
    end();
}

/// The pipe of the request with its first and last shapes.
fn pipe_shapes(r: &mut Reader<'_>) -> Result<(Shape, Shape, Shape), Failure> {
    let t = tp_shapes(r)?;
    let spine = topods::wire(&at(&t, r.integer())?)?;
    let profile = at(&t, r.integer())?;
    let wires = r.flag();
    let mut pipe = BRepOffsetAPIMakePipe::new(&spine, &profile)?;
    let s = pipe.shape()?.clone();
    let (mut first, mut last) = (pipe.first_shape()?, pipe.last_shape()?);
    if wires && first.shape_type()? == ShapeEnum::Face {
        first = outer(&first)?;
        last = outer(&last)?;
    }
    Ok((s, first, last))
}

/// `oa_middle program spine profile wires`.
pub fn middle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match pipe_shapes(&mut r) {
        Ok((s, first, last)) => middle_path(&s, &first, &last),
        Err(error) => failed(error),
    }
}

/// The primitive of the request with its two faces.
fn primitive_shapes(r: &mut Reader<'_>) -> Result<(Shape, Shape, Shape), Failure> {
    let s = primitive(r)?;
    let mut first = sub(&s, ShapeEnum::Face, r.integer())?;
    let mut last = sub(&s, ShapeEnum::Face, r.integer())?;
    if r.flag() {
        first = outer(&first)?;
        last = outer(&last)?;
    }
    Ok((s, first, last))
}

/// `oa_middle_prim prim start end wires`.
pub fn middle_prim_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match primitive_shapes(&mut r) {
        Ok((s, first, last)) => middle_path(&s, &first, &last),
        Err(error) => failed(error),
    }
}
