//! BRepFill_Draft probe operations, the Rust twin of
//! tools/oracle/tkbool_probe_brepfill_draft.cxx (the records are documented there).
use super::brep_fill_generator::{failed, list};
use super::brep_fill_location_law::at;
use super::common::{Reader, attempt, end, flag};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::brep_fill::{BRepFillDraft, TransitionStyle};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};

/// The transition of a request (0 modified, 1 right, 2 round).
fn transition_of(value: i32) -> TransitionStyle {
    match value {
        1 => TransitionStyle::Right,
        2 => TransitionStyle::Round,
        _ => TransitionStyle::Modified,
    }
}

/// The draft of the request with its options.
fn draft_of(r: &mut Reader<'_>, t: &TpShapes) -> Result<BRepFillDraft, Failure> {
    let s = at(t, r.integer())?;
    let (dir, angle) = (r.dir(), r.real());
    let style = transition_of(r.integer());
    let (angmin, angmax, internal) = (r.real(), r.real(), r.flag());
    let mut draft = BRepFillDraft::new(&s, dir, angle)?;
    draft.set_options(style, angmin, angmax);
    draft.set_draft(internal);
    Ok(draft)
}

/// The Perform of the request.
fn perform(r: &mut Reader<'_>, t: &TpShapes, draft: &mut BRepFillDraft) -> Result<(), Failure> {
    match r.integer() {
        0 => draft.perform_length(r.real()),
        1 => {
            let (p, n, keep) = (r.pnt(), r.dir(), r.flag());
            let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_point(p, n)?).cast();
            draft.perform_surface(&plane, keep)
        }
        _ => {
            let stop = at(t, r.integer())?;
            draft.perform_shape(&stop, r.flag())
        }
    }
}

/// `bf_draft program shape dir angle style angmin angmax internal mode ...`.
pub fn draft_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let mut draft = match draft_of(&mut r, &t) {
        Ok(draft) => draft,
        Err(error) => return failed(error),
    };
    if let Err(error) = perform(&mut r, &t, &mut draft) {
        return failed(error);
    }
    flag(draft.is_done());
    attempt(|| shape(&draft.shape()));
    attempt(|| shape(draft.shell().as_ref()));
    let mut map = IndexedMapOfShape::default();
    if !draft.shape().is_null() {
        topexp::map_shapes(&draft.shape(), &mut map, true, true);
    }
    if draft.is_done() {
        for g in &t.shapes {
            attempt(|| list(&map, draft.generated(g)?));
        }
    }
    end();
}
