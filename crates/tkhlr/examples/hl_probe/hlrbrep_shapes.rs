//! HLRBRep shape fixtures, the Rust twin of the shape part of
//! tools/oracle/tkhlr_probe_hlrbrep_fixtures.cxx.
use super::common::Reader;
use super::hlrbrep_faces::face;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Axis2;
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
    BRepPrimAPIMakeTorus,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn shape_of(maker: &mut dyn BRepBuilderAPIMakeShape) -> Result<Shape, Failure> {
    Ok(maker.shape()?.clone())
}

fn compound(r: &mut Reader, faces: bool) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut c = Compound::default();
    b.make_compound(&mut c);
    let mut c = Shape::from(c);
    for _ in 0..r.integer() {
        let s = if faces {
            Shape::from(face(r)?)
        } else {
            shape(r)?
        };
        b.add(&mut c, &s)?;
    }
    Ok(c)
}

/// `hk ...`: see `hb_shape`.
pub fn shape(r: &mut Reader) -> Result<Shape, Failure> {
    let kind = r.integer();
    match kind {
        6 => return Ok(Shape::from(face(r)?)),
        5 | 7 => return compound(r, kind == 5),
        _ => {}
    }
    let (l, n, x) = (r.pnt(), r.dir(), r.dir());
    let a = Axis2::with_x_direction(l, n, x)?;
    match kind {
        0 => {
            let (dx, dy, dz) = (r.real(), r.real(), r.real());
            shape_of(&mut BRepPrimAPIMakeBox::with_axes(a, dx, dy, dz))
        }
        1 => {
            let rr = r.real();
            shape_of(&mut BRepPrimAPIMakeCylinder::with_axes(a, rr, r.real()))
        }
        2 => {
            let (r1, r2) = (r.real(), r.real());
            shape_of(&mut BRepPrimAPIMakeCone::with_axes(a, r1, r2, r.real())?)
        }
        3 => shape_of(&mut BRepPrimAPIMakeSphere::with_axis(a, r.real())?),
        _ => {
            let r1 = r.real();
            shape_of(&mut BRepPrimAPIMakeTorus::with_axes(a, r1, r.real())?)
        }
    }
}
