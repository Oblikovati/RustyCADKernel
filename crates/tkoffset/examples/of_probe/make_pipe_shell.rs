//! BRepOffsetAPI_MakePipeShell probe operations, the Rust twin of
//! tools/oracle/tkoffset_probe_brepoffsetapi_make_pipe_shell.cxx (the records are
//! documented there).
use super::common::{Reader, attempt, end, flag, int, real};
use super::make_offset::{at, failed};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::brep_fill::TypeOfContact;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Axis2;
use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIMakePipeShell;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSweep;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransitionMode};

/// `of_list`: the indices of a list in the result map (-1 for a null shape).
pub fn list(map: &IndexedMapOfShape, l: &List<Shape>) {
    int(l.len() as i32);
    for s in l.iter() {
        int(if s.is_null() {
            -1
        } else {
            map.find_index(s) as i32
        });
    }
}

/// The contact of a request (0 none, 1 contact, 2 contact on border).
fn contact_of(value: i32) -> TypeOfContact {
    match value {
        1 => TypeOfContact::Contact,
        2 => TypeOfContact::ContactOnBorder,
        _ => TypeOfContact::NoContact,
    }
}

/// The transition of a request (0 transformed, 1 right corner, 2 round corner).
fn transition_of(value: i32) -> BRepBuilderAPITransitionMode {
    match value {
        1 => BRepBuilderAPITransitionMode::RightCorner,
        2 => BRepBuilderAPITransitionMode::RoundCorner,
        _ => BRepBuilderAPITransitionMode::Transformed,
    }
}

/// `of_pipe_mode`.
fn pipe_mode(
    r: &mut Reader<'_>,
    t: &TpShapes,
    pipe: &mut BRepOffsetAPIMakePipeShell,
) -> Result<(), Failure> {
    match r.integer() {
        1 => pipe.set_mode_frenet(true),
        2 => pipe.set_mode_frenet(false),
        3 => pipe.set_discrete_mode(),
        4 => {
            let (o, d, x) = (r.pnt(), r.dir(), r.dir());
            pipe.set_mode_axe(&Axis2::with_x_direction(o, d, x)?)
        }
        5 => pipe.set_mode_bi_normal(r.dir()),
        6 => {
            flag(pipe.set_mode_support(&at(t, r.integer())?)?);
            Ok(())
        }
        7 => {
            let g = topods::wire(&at(t, r.integer())?)?;
            let c = r.flag();
            pipe.set_mode_auxiliary(&g, c, contact_of(r.integer()))
        }
        _ => Ok(()),
    }
}

/// The maker of the spine with its location mode.
fn pipe_of(r: &mut Reader<'_>, t: &TpShapes) -> Result<BRepOffsetAPIMakePipeShell, Failure> {
    let spine = topods::wire(&at(t, r.integer())?)?;
    let mut pipe = BRepOffsetAPIMakePipeShell::new(&spine)?;
    pipe_mode(r, t, &mut pipe)?;
    Ok(pipe)
}

/// The transition and the profiles of the request.
fn profiles(r: &mut Reader<'_>, t: &TpShapes, pipe: &mut BRepOffsetAPIMakePipeShell) {
    let transition = r.integer();
    if transition >= 0 {
        pipe.set_transition_mode(transition_of(transition));
    }
    for _ in 0..r.integer() {
        let p = at(t, r.integer()).unwrap_or_default();
        let v = r.integer();
        let (c, w) = (r.flag(), r.flag());
        if v < 0 {
            attempt(|| pipe.add(&p, c, w));
        } else {
            attempt(|| pipe.add_at(&p, &topods::vertex(&at(t, v)?)?, c, w));
        }
    }
}

/// The records after Build.
fn print_built(pipe: &mut BRepOffsetAPIMakePipeShell, t: &TpShapes, solid: bool) {
    let done = pipe.is_done();
    flag(done);
    int(pipe.get_status() as i32);
    let mut map = IndexedMapOfShape::default();
    if done {
        real(pipe.error_on_surface());
        attempt(|| pipe.shape().and_then(shape));
    }
    attempt(|| pipe.first_shape().and_then(|s| shape(&s)));
    attempt(|| pipe.last_shape().and_then(|s| shape(&s)));
    if solid && done {
        attempt(|| {
            flag(pipe.make_solid()?);
            shape(pipe.shape()?)
        });
    }
    if done {
        let result = pipe.shape().cloned().unwrap_or_default();
        topexp::map_shapes(&result, &mut map, true, true);
    }
    for s in &t.shapes {
        attempt(|| pipe.generated(s).map(|l| list(&map, l)));
    }
    if done {
        attempt(|| pipe.simulate(3).map(|l| int(l.len() as i32)));
    }
}

/// `oa_pipe_shell program spine mode... transition n (profile vertex contact correction)... solid`.
pub fn pipe_shell_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let mut pipe = match pipe_of(&mut r, &t) {
        Ok(pipe) => pipe,
        Err(error) => return failed(error),
    };
    profiles(&mut r, &t, &mut pipe);
    let solid = r.flag();
    attempt(|| pipe.make_solid().map(flag));
    flag(pipe.is_ready());
    if let Err(error) = pipe.build(&ProgressRange::default()) {
        return failed(error);
    }
    print_built(&mut pipe, &t, solid);
    end();
}
