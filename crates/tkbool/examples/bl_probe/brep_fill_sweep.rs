//! BRepFill_Sweep probe operations, the Rust twin of
//! tools/oracle/tkbool_probe_brepfill_sweep.cxx (the records are documented there).
use super::brep_fill_generator::{failed, indexed};
use super::brep_fill_location_law::{at, law_of};
use super::common::{Reader, attempt, end, flag, int, real};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::brep_fill::{
    BRepFillLocationLaw, BRepFillSectionLaw, BRepFillShapeLaw, BRepFillSweep, SweepBuildOptions,
    SweepHistory, SweepTablesMap, TransitionStyle,
};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape, Wire};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::HArray2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `bl_sweep_table`.
fn sweep_table(result: &IndexedMapOfShape, table: &Handle<HArray2<Shape>>) -> Result<(), Failure> {
    let Some(shared) = table.get() else {
        int(-1);
        return Ok(());
    };
    let t = shared.array2();
    for bound in [t.lower_row(), t.upper_row(), t.lower_col(), t.upper_col()] {
        int(bound);
    }
    for i in t.lower_row()..=t.upper_row() {
        for j in t.lower_col()..=t.upper_col() {
            indexed(
                result,
                t.value(i, j)
                    .map_err(|e| Failure::out_of_range(&e.to_string()))?,
            )?;
        }
    }
    Ok(())
}

/// The section law of the vertex or wire.
fn section_law(section: &Shape) -> Result<Handle<dyn BRepFillSectionLaw>, Failure> {
    if section.shape_type()? == ShapeEnum::Vertex {
        let law = BRepFillShapeLaw::with_vertex(&topods::vertex(section)?, true)?;
        return Ok(Handle::new(law).cast());
    }
    Ok(Handle::new(BRepFillShapeLaw::with_wire(&topods::wire(section)?, true)?).cast())
}

/// The transition of a request (0 modified, 1 right, 2 round).
fn transition_of(value: i32) -> TransitionStyle {
    match value {
        1 => TransitionStyle::Right,
        2 => TransitionStyle::Round,
        _ => TransitionStyle::Modified,
    }
}

/// The settings of a request after the laws.
struct SweepRequest {
    transition: TransitionStyle,
    kpart: bool,
    bounds: i32,
    tol3d: f64,
    angmin: f64,
    angmax: f64,
    c1: bool,
}

/// The settings read from the request.
fn request_of(r: &mut Reader<'_>) -> SweepRequest {
    let (transition, kpart, bounds) = (transition_of(r.integer()), r.flag(), r.integer());
    let (tol3d, angmin, angmax, c1) = (r.real(), r.real(), r.real(), r.flag());
    SweepRequest {
        transition,
        kpart,
        bounds,
        tol3d,
        angmin,
        angmax,
        c1,
    }
}

/// The sweep set up as the request says.
fn sweep_of(
    sec: &Handle<dyn BRepFillSectionLaw>,
    loc: &Handle<dyn BRepFillLocationLaw>,
    section: &Shape,
    q: &SweepRequest,
) -> BRepFillSweep {
    let mut sweep = BRepFillSweep::new(sec, loc, q.kpart);
    sweep.set_tolerance(q.tol3d, 1.0, 1.0e-5, 1.0e-2);
    sweep.set_angular_control(q.angmin, q.angmax);
    sweep.set_force_approx_c1(q.c1);
    if q.bounds > 0 && section.shape_type().ok() == Some(ShapeEnum::Wire) {
        let w = Wire::unchecked(section.clone());
        let none = Wire::default();
        let first = if q.bounds == 3 { &none } else { &w };
        let last = if q.bounds >= 2 { &w } else { &none };
        attempt(|| sweep.set_bounds(first, last));
    }
    sweep
}

/// The outputs of a built sweep.
fn print_sweep(sweep: &BRepFillSweep, sec: &dyn BRepFillSectionLaw) {
    flag(sweep.is_done());
    real(sweep.error_on_surface());
    let result = sweep.shape();
    attempt(|| shape(&result));
    let mut map = IndexedMapOfShape::default();
    if !result.is_null() {
        topexp::map_shapes(&result, &mut map, true, true);
    }
    attempt(|| sweep_table(&map, &sweep.sub_shape()));
    attempt(|| sweep_table(&map, &sweep.inter_faces()));
    attempt(|| sweep_table(&map, &sweep.sections()));
    if !sec.is_vertex() {
        for i in 1..=sec.nb_law().unwrap_or(0) {
            attempt(|| {
                int(sweep.tape(i)?.nb_children());
                Ok(())
            });
        }
    }
}

/// `bf_sweep program path section law... transition kpart bounds tol3d angmin angmax c1`.
pub fn sweep_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t: TpShapes = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let (path, section) = match (at(&t, r.integer()), at(&t, r.integer())) {
        (Ok(p), Ok(s)) => (p, s),
        (Err(error), _) | (_, Err(error)) => return failed(error),
    };
    let laws = law_of(&mut r, &t, &path).and_then(|loc| Ok((loc, section_law(&section)?)));
    let (loc, sec) = match laws {
        Ok(laws) => laws,
        Err(error) => return failed(error),
    };
    let q = request_of(&mut r);
    let Some(loc) = loc else {
        return end();
    };
    let mut sweep = sweep_of(&sec, &loc, &section, &q);
    let (mut reversed, mut tapes, mut rails) = (
        MapOfShape::default(),
        SweepTablesMap::default(),
        SweepTablesMap::default(),
    );
    let mut hist = SweepHistory {
        reversed_edges: &mut reversed,
        tapes: &mut tapes,
        rails: &mut rails,
    };
    let options = SweepBuildOptions {
        transition: q.transition,
        ..SweepBuildOptions::default()
    };
    if let Err(error) = sweep.build(&mut hist, &options) {
        return failed(error);
    }
    if let Some(sec) = sec.get() {
        print_sweep(&sweep, sec);
    }
    int(reversed.extent() as i32);
    int(tapes.extent() as i32);
    int(rails.extent() as i32);
    end();
}
