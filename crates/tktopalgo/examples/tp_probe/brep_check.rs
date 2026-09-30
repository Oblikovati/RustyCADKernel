//! The BRepCheck and BRepLib_ValidateEdge operations of the TKTopAlgo probe, the Rust
//! twin of tools/oracle/tktopalgo_probe_brepcheck.cxx (the request layouts are
//! documented there).
use super::common::{Reader, attempt, captured, end, flag, int, real, show};
use super::shapes::{TpShapes, tp_same_index, tp_shape_index, tp_shapes};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape, edge, face, shell, vertex, wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_check::{
    BRepCheck, BRepCheckEdge, BRepCheckFace, BRepCheckResult, BRepCheckShell, BRepCheckStatus,
    BRepCheckVertex, BRepCheckWire, ListOfStatus,
};
use rustycad_tktopalgo::brep_lib::BRepLibValidateEdge;

fn list(l: &ListOfStatus) {
    int(l.extent());
    for s in l {
        int(*s as i32);
    }
}
fn status(value: i32) -> BRepCheckStatus {
    BRepCheckStatus::from_index(value).expect("the corpus names valid statuses")
}
fn table_edge(t: &TpShapes, index: i32) -> Result<Edge, Failure> {
    if index < 0 {
        return Ok(Edge::default());
    }
    edge(&t.shapes[index as usize])
}

/// `tp_bck_contexts`: the context shapes as sorted (index, statuses) pairs.
fn contexts(t: &TpShapes, c: &dyn BRepCheckResult) -> Result<(), Failure> {
    let mut found: Vec<(i32, Vec<i32>)> = Vec::new();
    c.init_context_iterator()?;
    while c.more_shape_in_context() {
        let l = c.status_on_shape()?.iter().map(|s| *s as i32).collect();
        found.push((tp_shape_index(t, &c.contextual_shape()?), l));
        c.next_shape_in_context();
    }
    found.sort();
    int(found.len() as i32);
    for (index, l) in found {
        int(index);
        int(l.len() as i32);
        l.into_iter().for_each(int);
    }
    Ok(())
}

/// `tp_bck_summary`.
fn summary(t: &TpShapes, c: &dyn BRepCheckResult) {
    show(c.status(), |l| list(&l));
    for (i, s) in t.shapes.iter().enumerate() {
        if c.is_status_on_shape(s) {
            int(i as i32);
            show(c.status_on_shape_of(s), |l| list(&l));
        }
    }
    int(-9);
}

/// `tp_bck_result_member`: codes 90..99; false for another code.
fn result_member(r: &mut Reader<'_>, t: &TpShapes, c: &dyn BRepCheckResult, code: i32) -> bool {
    match code {
        90 => {
            let i = r.integer() as usize;
            attempt(|| c.in_context(&t.shapes[i]).map(|()| int(0)));
        }
        91 => c.blind(),
        92 => show(c.status(), |l| list(&l)),
        93 => show(c.status_on_shape_of(&t.shapes[r.integer() as usize]), |l| {
            list(&l)
        }),
        94 => flag(c.is_status_on_shape(&t.shapes[r.integer() as usize])),
        95 => attempt(|| contexts(t, c)),
        96 => {
            c.set_parallel(r.flag());
            flag(c.is_parallel());
        }
        97 => c.set_fail_status(&t.shapes[r.integer() as usize]),
        98 => {
            let i = r.integer() as usize;
            attempt(|| c.init(&t.shapes[i]).map(|()| int(0)));
        }
        99 => {
            flag(c.is_minimum());
            flag(c.is_blind());
        }
        _ => return false,
    }
    true
}

fn vertex_member(_: &mut Reader<'_>, _: &TpShapes, v: &BRepCheckVertex, code: i32) {
    if code == 1 {
        show(v.tolerance(), real);
    }
}

fn edge_member(r: &mut Reader<'_>, t: &TpShapes, e: &BRepCheckEdge, code: i32) {
    match code {
        1 => show(e.tolerance(), real),
        2 => e.set_geometric_controls(r.flag()),
        3 => flag(e.geometric_controls()),
        4 => e.set_exact_method(r.flag()),
        5 => flag(e.is_exact_method()),
        6 => attempt(|| e.set_status(status(r.integer()))),
        7 => {
            let i = r.integer();
            show(
                table_edge(t, i).and_then(|x| e.check_polygon_on_triangulation(&x)),
                |s| int(s as i32),
            );
        }
        _ => {}
    }
}

fn wire_member(r: &mut Reader<'_>, t: &TpShapes, w: &BRepCheckWire, code: i32) {
    let face_at = |i: i32| face(&t.shapes[i as usize]);
    match code {
        1 => show(w.closed(r.flag()), |s| int(s as i32)),
        2 => {
            let (f, u) = (r.integer(), r.flag());
            show(
                face_at(f).and_then(|f| w.closed2d(&f, u)),
                |s| int(s as i32),
            );
        }
        3 => {
            let (f, u) = (r.integer(), r.flag());
            show(face_at(f).and_then(|f| w.orientation(&f, u)), |s| {
                int(s as i32)
            });
        }
        4 => {
            let (f, i1, i2, u) = (r.integer(), r.integer(), r.integer(), r.flag());
            let (mut e1, mut e2) = (
                table_edge(t, i1).expect("edge"),
                table_edge(t, i2).expect("edge"),
            );
            show(
                face_at(f).and_then(|f| w.self_intersect(&f, &mut e1, &mut e2, u)),
                |s| int(s as i32),
            );
            int(tp_shape_index(t, &e1));
            int(tp_shape_index(t, &e2));
        }
        5 => w.set_geometric_controls(r.flag()),
        6 => flag(w.geometric_controls()),
        7 => attempt(|| w.set_status(status(r.integer()))),
        _ => {}
    }
}

fn face_member(r: &mut Reader<'_>, _: &TpShapes, f: &BRepCheckFace, code: i32) {
    match code {
        1 => show(f.intersect_wires(r.flag()), |s| int(s as i32)),
        2 => show(f.classify_wires(r.flag()), |s| int(s as i32)),
        3 => show(f.orientation_of_wires(r.flag()), |s| int(s as i32)),
        4 => attempt(|| f.set_unorientable()),
        5 => show(f.is_unorientable(), flag),
        6 => attempt(|| f.set_status(status(r.integer()))),
        7 => f.set_geometric_controls(r.flag()),
        8 => flag(f.geometric_controls()),
        _ => {}
    }
}

/// `tp_bck_sets`: the count, then each set's closed flag and sorted face indices.
fn sets(t: &TpShapes, s: &BRepCheckShell) -> Result<(), Failure> {
    let mut found: List<Shape> = List::default();
    int(s.nb_connected_set(&mut found)?);
    for set in &found {
        flag(set.closed()?);
        let mut faces: Vec<i32> = Explorer::new(set, ShapeEnum::Face, ShapeEnum::Shape)?
            .map(|f| tp_same_index(t, &f))
            .collect();
        faces.sort();
        int(faces.len() as i32);
        faces.into_iter().for_each(int);
    }
    Ok(())
}

fn shell_member(r: &mut Reader<'_>, t: &TpShapes, s: &BRepCheckShell, code: i32) {
    match code {
        1 => show(s.closed(r.flag()), |x| int(x as i32)),
        2 => show(s.orientation(r.flag()), |x| int(x as i32)),
        3 => attempt(|| s.set_unorientable()),
        4 => show(s.is_unorientable(), flag),
        5 => attempt(|| sets(t, s)),
        _ => {}
    }
}

/// `tp_bck_checker`.
fn checker(kind: i32, s: &Shape) -> Result<Handle<dyn BRepCheckResult>, Failure> {
    Ok(match kind {
        0 => Handle::new(BRepCheckVertex::new(&vertex(s)?)?).cast(),
        1 => Handle::new(BRepCheckEdge::new(&edge(s)?)?).cast(),
        2 => Handle::new(BRepCheckWire::new(&wire(s)?)?).cast(),
        3 => Handle::new(BRepCheckFace::new(&face(s)?)?).cast(),
        _ => Handle::new(BRepCheckShell::new(&shell(s)?)?).cast(),
    })
}

/// `tp_bck_member`.
fn member(r: &mut Reader<'_>, t: &TpShapes, c: &Handle<dyn BRepCheckResult>, kind: i32, code: i32) {
    let checker: &dyn BRepCheckResult = c.get().expect("checker");
    if result_member(r, t, checker, code) {
        return;
    }
    match kind {
        0 => vertex_member(
            r,
            t,
            c.cast::<BRepCheckVertex>().get().expect("vertex"),
            code,
        ),
        1 => edge_member(r, t, c.cast::<BRepCheckEdge>().get().expect("edge"), code),
        2 => wire_member(r, t, c.cast::<BRepCheckWire>().get().expect("wire"), code),
        3 => face_member(r, t, c.cast::<BRepCheckFace>().get().expect("face"), code),
        _ => shell_member(r, t, c.cast::<BRepCheckShell>().get().expect("shell"), code),
    }
}

/// `bck_check kind program index script`.
pub fn check_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let t = tp_shapes(&mut r).expect("shape program");
    let index = r.integer() as usize;
    let c = match checker(kind, &t.shapes[index]) {
        Ok(c) => c,
        Err(error) => {
            print!("{} ", error.exception_type());
            end();
            return;
        }
    };
    for _ in 0..r.integer() {
        let code = r.integer();
        member(&mut r, &t, &c, kind, code);
    }
    summary(&t, c.get().expect("checker"));
    end();
}

/// `bck_status s`.
pub fn status_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = r.integer();
    captured(|console| {
        if let Some(value) = BRepCheckStatus::from_index(s) {
            let mut text: Vec<u8> = Vec::new();
            BRepCheck::print(value, &mut text).map_err(|e| Failure::new(&e.to_string()))?;
            console.push_str(&String::from_utf8_lossy(&text));
        }
        Ok(())
    });
    end();
}

/// `bck_add n (s)...`.
pub fn add_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut l = ListOfStatus::default();
    for _ in 0..r.integer() {
        BRepCheck::add(&mut l, status(r.integer()));
        list(&l);
    }
    end();
}

/// `bck_prec program kind a b`.
pub fn prec_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("shape program");
    let (kind, a, b) = (r.integer(), r.integer() as usize, r.integer());
    let value = (|| -> Result<f64, Failure> {
        match kind {
            0 => BRepCheck::prec_curve(&GeomAdaptorCurve::with_curve(&t.curves[a])?),
            1 => {
                let s: Handle<dyn Adaptor3dSurface> =
                    Handle::new(GeomAdaptorSurface::with_surface(&t.surfaces[a])?).cast();
                BRepCheck::prec_surface(&s)
            }
            _ => {
                let c: Handle<dyn Adaptor2dCurve2d> =
                    Handle::new(Geom2dAdaptorCurve::with_curve(&t.pcurves[a])?).cast();
                let s: Handle<dyn Adaptor3dSurface> =
                    Handle::new(GeomAdaptorSurface::with_surface(&t.surfaces[b as usize])?).cast();
                BRepCheck::prec_curve(&Adaptor3dCurveOnSurface::with_curve(&c, &s)?)
            }
        }
    })();
    show(value, real);
    end();
}

/// `bck_selfint program wire face e1 e2`.
pub fn selfint_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("shape program");
    let (w, f, i1, i2) = (
        r.integer() as usize,
        r.integer() as usize,
        r.integer(),
        r.integer(),
    );
    let (mut e1, mut e2) = (
        table_edge(&t, i1).expect("edge"),
        table_edge(&t, i2).expect("edge"),
    );
    let found = wire(&t.shapes[w])
        .and_then(|w| Ok((w, face(&t.shapes[f])?)))
        .and_then(|(w, f)| BRepCheck::self_intersection(&w, &f, &mut e1, &mut e2));
    show(found, flag);
    int(tp_shape_index(&t, &e1));
    int(tp_shape_index(&t, &e2));
    end();
}

/// `tp_ble_reference`.
fn reference(r: &mut Reader<'_>, t: &TpShapes) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let (kind, c, f, l) = (r.integer(), r.integer() as usize, r.real(), r.real());
    if kind == 0 {
        return Ok(Handle::new(GeomAdaptorCurve::with_range(&t.curves[c], f, l)?).cast());
    }
    let s = r.integer() as usize;
    Ok(curve_on_surface(t, c, (f, l), s)?.cast())
}

fn curve_on_surface(
    t: &TpShapes,
    pc: usize,
    (f, l): (f64, f64),
    s: usize,
) -> Result<Handle<Adaptor3dCurveOnSurface>, Failure> {
    let c: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(&t.pcurves[pc], f, l)?).cast();
    let surface: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&t.surfaces[s])?).cast();
    Ok(Handle::new(Adaptor3dCurveOnSurface::with_curve(
        &c, &surface,
    )?))
}

/// `ble_validate program reference (pc f l s) sameparameter script`.
pub fn validate_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("shape program");
    let reference = reference(&mut r, &t).expect("reference curve");
    let (pc, f, l, s) = (
        r.integer() as usize,
        r.real(),
        r.real(),
        r.integer() as usize,
    );
    let other = curve_on_surface(&t, pc, (f, l), s).expect("curve on surface");
    let mut v = BRepLibValidateEdge::new(&reference, &other, r.flag());
    for _ in 0..r.integer() {
        match r.integer() {
            1 => v.set_exact_method(r.flag()),
            2 => v.set_parallel(r.flag()),
            3 => v.set_control_points_number(r.integer()),
            4 => {
                let x = r.real();
                attempt(|| v.set_exit_if_tolerance_exceeded(x));
            }
            5 => attempt(|| v.process()),
            6 => flag(v.is_done()),
            7 => real(v.get_max_distance()),
            8 => show(v.check_tolerance(r.real()), flag),
            9 => {
                let mut x = r.real();
                v.update_tolerance(&mut x);
                real(x);
            }
            10 => flag(v.is_exact_method()),
            _ => flag(v.is_parallel()),
        }
    }
    end();
}
