//! ShapeBuild_ReShape probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_reshape.cxx (which documents the request).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, end, flag, int};
use super::shapes::{TpShapes, tp_same_index, tp_shapes};
use super::wire_data::sh_shape;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;

fn result(t: &TpShapes, s: &Shape) -> Result<(), Failure> {
    int(tp_same_index(t, s));
    int(if s.is_null() {
        -1
    } else {
        s.orientation() as i32
    });
    shape(s)
}

fn request(r: &mut Reader<'_>, t: &TpShapes, rs: &ShapeBuildReShape) -> Result<(), Failure> {
    match r.integer() {
        0 => {
            let s = sh_shape(t, r.integer());
            rs.replace(&s, &sh_shape(t, r.integer()))
        }
        1 => rs.remove(&sh_shape(t, r.integer())),
        _ => {
            rs.set_mode_consider_location(r.flag());
            Ok(())
        }
    }
}

fn apply(r: &mut Reader<'_>, t: &TpShapes, rs: &ShapeBuildReShape) {
    let op = r.integer();
    let s = sh_shape(t, r.integer());
    match op {
        0 => {
            let until = ShapeEnum::ALL[r.integer() as usize];
            attempt(|| result(t, &rs.apply(&s, until)?));
            ShapeExtendStatus::ALL
                .iter()
                .for_each(|&k| flag(rs.status_flag(k)));
        }
        1 => {
            let until = ShapeEnum::ALL[r.integer() as usize];
            let mode = r.integer();
            attempt(|| result(t, &rs.apply_with_build_mode(&s, until, mode)?));
        }
        2 => {
            let last = r.flag();
            attempt(|| {
                let (status, new_shape) = rs.status(&s, last)?;
                int(status);
                result(t, &new_shape)
            });
        }
        _ => {
            flag(rs.is_recorded(&s));
            attempt(|| result(t, &rs.value(&s)?));
        }
    }
}

/// `sh_reshape program nreq (req)... napply (apply)...`.
pub fn reshape_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shape program");
    let rs = Handle::new(ShapeBuildReShape::new());
    for _ in 0..r.integer() {
        attempt(|| request(&mut r, &t, &rs));
    }
    for _ in 0..r.integer() {
        apply(&mut r, &t, &rs);
    }
    flag(rs.is_kind_name("BRepTools_ReShape"));
    end();
}
