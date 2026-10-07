//! ShapeFix_Face probe operation, the Rust twin of tools/oracle/tkshhealing_probe_fix_face.cxx
//! (which documents the requests).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, end, flag, int};
use super::unify::wire;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Face, Shape, Wire};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Plane3;
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis, BRepPrimAPIMakeSphere,
    BRepPrimAPIMakeTorus,
};
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixFace, ShapeFixFaceWireMap};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon};

fn polygon(r: &mut Reader<'_>) -> Result<Wire, Failure> {
    let mut maker = BRepBuilderAPIMakePolygon::new();
    for _ in 0..r.integer() {
        maker.add_point(r.pnt())?;
    }
    maker.close()?;
    maker.wire()
}

fn planar(r: &mut Reader<'_>) -> Result<Face, Failure> {
    let n = r.integer();
    let mut wires = Vec::new();
    for _ in 0..n {
        wires.push(polygon(r)?);
    }
    let reversed: Vec<bool> = (0..n).map(|_| r.flag()).collect();
    for (w, rev) in wires.iter_mut().zip(&reversed) {
        if *rev {
            w.reverse();
        }
    }
    let mut face =
        BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &wires[0], true)?.face()?;
    let builder = BRepBuilder::default();
    for w in &wires[1..] {
        builder.add(&mut face, w)?;
    }
    Ok(face)
}

fn lateral(r: &mut Reader<'_>) -> Result<Face, Failure> {
    let kind = r.integer();
    let angle = r.real();
    match kind {
        0 => {
            let radius = r.real();
            BRepPrimAPIMakeCylinder::with_angle(radius, r.real(), angle)?.face()
        }
        1 => {
            let (r1, r2) = (r.real(), r.real());
            BRepPrimAPIMakeCone::with_angle(r1, r2, r.real(), angle)?.face()
        }
        2 => BRepPrimAPIMakeSphere::with_angle(r.real(), angle)?.face(),
        _ => {
            let r1 = r.real();
            BRepPrimAPIMakeTorus::with_angle(r1, r.real(), angle)?.face()
        }
    }
}

/// The `sh_fix_face` input face.
pub(super) fn input(r: &mut Reader<'_>) -> Result<Face, Failure> {
    match r.integer() {
        0 => planar(r),
        1 => lateral(r),
        _ => {
            let w = topods::wire(&wire(r)?)?;
            BRepBuilderAPIMakeFace::from_wire(&w, true)?.face()
        }
    }
}

fn status(f: &ShapeFixFace) {
    use ShapeExtendStatus::*;
    for s in [
        Done1, Done2, Done3, Done4, Done5, Done6, Done7, Done8, Fail1, Fail2, Fail3,
    ] {
        flag(f.status(s));
    }
}

fn run(f: &ShapeFixFace, op: i32) -> Result<(), Failure> {
    match op {
        0 => flag(f.perform()?),
        11..=14 => {
            match op {
                11 => f.set_fix_wire_mode(0),
                12 => f.set_fix_loop_wires_mode(0),
                13 => f.set_fix_split_face_mode(0),
                _ => f.set_fix_intersecting_wires_mode(0),
            }
            flag(f.perform()?);
        }
        1 => flag(f.fix_orientation()?),
        2 => flag(f.fix_add_natural_bound()?),
        3 => flag(f.fix_missing_seam()?),
        4 | 5 => flag(f.fix_small_area_wire(op == 5)?),
        6 => flag(f.fix_intersecting_wires()?),
        7 => flag(f.fix_wires_two_coinc_edges()?),
        8 => flag(f.fix_periodic_degenerated()?),
        9 => {
            let mut wires: Sequence<Shape> = Sequence::default();
            flag(f.fix_loop_wire(&mut wires)?);
            int(wires.len() as i32);
            for w in wires.iter() {
                shape(w)?;
            }
        }
        _ => {
            let mut map = ShapeFixFaceWireMap::default();
            flag(f.fix_orientation_with_map(&mut map)?);
            int(map.extent() as i32);
            flag(f.fix_split_face(&map)?);
        }
    }
    Ok(())
}

/// `sh_fix_face (input) ctx prec maxtol op`.
pub fn fix_face_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut face = Face::default();
    attempt(|| {
        face = input(&mut r)?;
        Ok(())
    });
    let ctx = r.flag();
    let (prec, maxtol) = (r.real(), r.real());
    let op = r.integer();
    if face.is_null() {
        end();
        return;
    }
    let fixer = ShapeFixFace::new(face.into_shape());
    if ctx {
        fixer.set_context(&Handle::new(ShapeBuildReShape::new()));
    }
    fixer.set_precision(prec);
    fixer.set_max_tolerance(maxtol);
    attempt(|| {
        run(&fixer, op)?;
        status(&fixer);
        shape(fixer.face()?.as_ref())?;
        shape(&fixer.result())
    });
    end();
}
