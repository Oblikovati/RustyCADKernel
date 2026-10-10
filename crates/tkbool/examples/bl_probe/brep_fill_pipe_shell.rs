//! BRepFill_PipeShell probe operations, the Rust twin of
//! tools/oracle/tkbool_probe_brepfill_pipe_shell.cxx (the records are documented there).
use super::brep_fill_generator::{failed, list};
use super::brep_fill_location_law::at;
use super::common::{Reader, attempt, end, flag, int, real};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::brep_fill::{BRepFillPipeShell, TransitionStyle, TypeOfContact};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::law::{LawFunction, LawLinear};
use rustycad_tkmath::gp::Axis2;

/// The transition of a request (0 modified, 1 right, 2 round).
fn transition_of(value: i32) -> TransitionStyle {
    match value {
        1 => TransitionStyle::Right,
        2 => TransitionStyle::Round,
        _ => TransitionStyle::Modified,
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

/// `bl_pipe_mode`.
fn pipe_mode(
    r: &mut Reader<'_>,
    t: &TpShapes,
    pipe: &mut BRepFillPipeShell,
) -> Result<(), Failure> {
    match r.integer() {
        1 => pipe.set_frenet(true),
        2 => pipe.set_frenet(false),
        3 => pipe.set_discrete(),
        4 => {
            let (o, d, x) = (r.pnt(), r.dir(), r.dir());
            pipe.set_axe(&Axis2::with_x_direction(o, d, x)?)
        }
        5 => pipe.set_bi_normal(r.dir()),
        6 => {
            flag(pipe.set_spine_support(&at(t, r.integer())?)?);
            Ok(())
        }
        7 => {
            let g = topods::wire(&at(t, r.integer())?)?;
            let c = r.flag();
            pipe.set_auxiliary_spine(&g, c, contact_of(r.integer()))
        }
        _ => Ok(()),
    }
}

/// The pipe of the spine with its location mode.
fn pipe_of(r: &mut Reader<'_>, t: &TpShapes) -> Result<BRepFillPipeShell, Failure> {
    let spine = topods::wire(&at(t, r.integer())?)?;
    let mut pipe = BRepFillPipeShell::new(&spine)?;
    pipe_mode(r, t, &mut pipe)?;
    Ok(pipe)
}

/// The settings and the profiles of the request; the first profile.
fn profiles(r: &mut Reader<'_>, t: &TpShapes, pipe: &mut BRepFillPipeShell) -> Shape {
    let transition = transition_of(r.integer());
    let (angmin, angmax) = (r.real(), r.real());
    pipe.set_transition(transition, angmin, angmax);
    pipe.set_max_degree(r.integer());
    pipe.set_max_segments(r.integer());
    pipe.set_force_approx_c1(r.flag());
    let mut first = Shape::default();
    for k in 0..r.integer() {
        let p = at(t, r.integer()).unwrap_or_default();
        let v = r.integer();
        let (c, w) = (r.flag(), r.flag());
        if k == 0 {
            first = p.clone();
        }
        if v < 0 {
            attempt(|| pipe.add(&p, c, w));
        } else {
            attempt(|| pipe.add_at(&p, &topods::vertex(&at(t, v)?)?, c, w));
        }
    }
    if r.flag() {
        let lin = LawLinear::new();
        let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
        attempt(|| {
            lin.set(a, b, c, d);
            let law: Handle<dyn LawFunction> = Handle::new(lin).cast();
            pipe.set_law(&first, &law, false, false)
        });
    }
    first
}

/// The outputs after Build.
fn print_pipe(pipe: &mut BRepFillPipeShell, t: &TpShapes, built: bool, solid: bool) {
    int(pipe.get_status() as i32);
    if built {
        real(pipe.error_on_surface());
    }
    attempt(|| shape(pipe.shape()));
    attempt(|| shape(pipe.first_shape()));
    attempt(|| shape(pipe.last_shape()));
    if solid && built {
        attempt(|| {
            flag(pipe.make_solid()?);
            shape(pipe.shape())
        });
    }
    let mut map = IndexedMapOfShape::default();
    if !pipe.shape().is_null() {
        topexp::map_shapes(pipe.shape(), &mut map, true, true);
    }
    for s in &t.shapes {
        attempt(|| list(&map, &pipe.generated(s)));
    }
    if built {
        attempt(|| {
            let sections = pipe.simulate(3)?;
            int(sections.len() as i32);
            for s in sections.iter() {
                shape(s)?;
            }
            Ok(())
        });
    }
}

/// `bf_pipe_shell program spine mode... transition angmin angmax maxdeg maxseg c1 n
/// (profile vertex contact correction)... law [pdeb valdeb pfin valfin] solid`.
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
    flag(pipe.is_ready());
    let built = match pipe.build() {
        Ok(built) => built,
        Err(error) => return failed(error),
    };
    flag(built);
    print_pipe(&mut pipe, &t, built, solid);
    end();
}
