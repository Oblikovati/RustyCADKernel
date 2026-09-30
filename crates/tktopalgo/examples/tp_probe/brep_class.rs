//! BRepClass probe operations over faces read by `tp_face`, the Rust twin of
//! tools/oracle/tktopalgo_probe_brepclass.cxx. Edges are printed as their index among
//! the edges of the FORWARD face (`tp_edge_index`).
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, xy};
use super::face::{TpFace, tp_edge_index, tp_face};
use super::int_res2d::inter;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::int_res2d::Position;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Line2, Point2};
use rustycad_tktopalgo::brep_class::{
    BRepClassEdge, BRepClassFaceClassifier, BRepClassFaceExplorer, BRepClassIntersector,
    FClass2dOfFClassifier, FClassifier, FacePassiveClassifier,
};

fn orientation(r: &mut Reader<'_>) -> Orientation {
    Orientation::ALL[r.integer() as usize]
}

/// `tp_edge_data`: edge, face, next edge, pcurve range, box and switches.
fn edge_data(f: &TpFace, e: &BRepClassEdge) -> Result<(), Failure> {
    int(tp_edge_index(&f.face, e.edge()));
    flag(e.face().is_null());
    if !e.face().is_null() {
        flag(e.face().is_same(&f.face));
        int(e.face().orientation() as i32);
    }
    int(tp_edge_index(&f.face, e.next_edge()));
    flag(e.curve().is_null());
    real(e.first_parameter());
    real(e.last_parameter());
    int(e.bounding_box_state() as i32);
    flag(e.bounding_box().is_void());
    if !e.bounding_box().is_void() {
        e.bounding_box().get()?.into_iter().for_each(real);
    }
    real(e.max_tolerance());
    flag(e.use_bnd_box());
    Ok(())
}

/// `tp_edge_command`: one BRepClass_Edge command (codes of the native file).
fn edge_command(
    r: &mut Reader<'_>,
    f: &TpFace,
    e: &mut BRepClassEdge,
    k: usize,
) -> Result<(), Failure> {
    match r.integer() {
        0 => {
            let found = BRepTool::curve_on_face(&f.edges[k], &f.face)?;
            e.set_geometry(found.pcurve, found.first, found.last);
        }
        1 => {
            let (x0, y0, x1, y1) = (r.real(), r.real(), r.real(), r.real());
            let mut bounds = BoundingBox2::default();
            bounds.update_bounds([x0, y0], [x1, y1]);
            e.set_bounding_box(&bounds);
        }
        2 => e.set_bounding_box(&BoundingBox2::default()),
        3 => e.set_bounding_box_unavailable(),
        4 => {
            let j = r.integer() as usize;
            e.set_edge(Edge::unchecked(f.edges[j].oriented(orientation(r))));
        }
        5 => e.set_face(Face::unchecked(f.face.oriented(orientation(r)))),
        6 => e.set_next_edge(f.edges[r.integer() as usize].clone()),
        7 => e.set_max_tolerance(r.real()),
        _ => e.set_use_bnd_box(r.flag()),
    }
    Ok(())
}

/// `face k mode n (command)...`: a BRepClass_Edge and its state after each command.
pub fn edge_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let k = r.integer() as usize;
        let mut e = BRepClassEdge::new();
        if r.flag() {
            e = BRepClassEdge::with_edge_face(f.edges[k].clone(), f.face.clone());
        }
        edge_data(&f, &e)?;
        for _ in 0..r.integer() {
            edge_command(&mut r, &f, &mut e, k)?;
            edge_data(&f, &e)?;
        }
        Ok(())
    });
    end();
}

/// `tp_explorer_edges`: orientation and state of each edge in wire order.
fn explorer_edges(f: &TpFace, x: &mut BRepClassFaceExplorer) -> Result<(), Failure> {
    x.init_wires();
    while x.more_wires() {
        flag(x.reject_wire(&Line2::default(), 1.0));
        x.init_edges();
        while x.more_edges() {
            let (e, o) = x.current_edge()?;
            int(o as i32);
            flag(x.reject_edge(&Line2::default(), 1.0));
            edge_data(f, &e)?;
            x.next_edge();
        }
        x.next_wire();
    }
    Ok(())
}

/// `face gaptol usebox`: the explorer's switches, ShouldUseBndBox, then its edges, and
/// its edges again after boxes are switched off and on.
pub fn explorer_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let mut x = BRepClassFaceExplorer::new(&f.face)?;
        let (gap, bounds) = (r.real(), r.flag());
        real(x.max_tolerance());
        flag(x.use_bnd_box());
        flag(x.reject(&Point2::default()));
        x.set_max_tolerance(gap);
        flag(x.should_use_bnd_box()?);
        x.set_use_bnd_box(bounds);
        flag(x.use_bnd_box());
        x.set_use_bnd_box(bounds);
        explorer_edges(&f, &mut x)?;
        x.set_use_bnd_box(false);
        flag(x.use_bnd_box());
        x.set_use_bnd_box(true);
        explorer_edges(&f, &mut x)
    });
    end();
}

fn line((found, l, par): (bool, Line2, f64)) {
    flag(found);
    pnt2d(l.location());
    xy(l.direction().xy());
    real(par);
}

/// `face u v n`: Segment, then n-1 OtherSegment from (u, v).
pub fn segment_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let mut x = BRepClassFaceExplorer::new(&f.face)?;
        let (p, n) = (r.pnt2d(), r.integer());
        line(x.segment(&p)?);
        for _ in 1..n {
            line(x.other_segment(&p)?);
        }
        Ok(())
    });
    end();
}

/// `face u v`: CheckPoint until it accepts the point (at most 4 calls).
pub fn checkpoint_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let mut x = BRepClassFaceExplorer::new(&f.face)?;
        let mut p = r.pnt2d();
        for _ in 0..4 {
            let accepted = x.check_point(&mut p)?;
            flag(accepted);
            pnt2d(p);
            if accepted {
                break;
            }
        }
        Ok(())
    });
    end();
}

/// `tp_explorer_edge`: the explorer edge of rank k in wire order.
fn explorer_edge(x: &mut BRepClassFaceExplorer, k: usize) -> Result<BRepClassEdge, Failure> {
    let mut i = 0;
    x.init_wires();
    while x.more_wires() {
        x.init_edges();
        while x.more_edges() {
            if i == k {
                return Ok(x.current_edge()?.0);
            }
            x.next_edge();
            i += 1;
        }
        x.next_wire();
    }
    Ok(BRepClassEdge::new())
}

fn local(intersector: &BRepClassIntersector, e: &BRepClassEdge, u: f64) {
    attempt(|| {
        let (t, n, c) = intersector.local_geometry(e, u)?;
        xy(t.xy());
        xy(n.xy());
        real(c);
        Ok(())
    });
}

fn box_mode(e: &mut BRepClassEdge, mode: i32) {
    match mode {
        1 => e.set_bounding_box_unavailable(),
        2 => {
            let c = e.curve().clone();
            e.set_geometry(c, e.first_parameter(), e.last_parameter());
        }
        3 => e.set_edge(e.edge().clone()),
        _ => {}
    }
}

/// `face k P D par tol gaptol usebox mode`: Perform on the explorer edge k, the results,
/// the box state, then LocalGeometry at the ends and middle.
pub fn intersect_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let mut x = BRepClassFaceExplorer::new(&f.face)?;
        let k = r.integer() as usize;
        let (p, d, par, tol) = (r.pnt2d(), r.dir2d(), r.real(), r.real());
        x.set_max_tolerance(r.real());
        x.set_use_bnd_box(r.flag());
        let mut e = explorer_edge(&mut x, k)?;
        box_mode(&mut e, r.integer());
        let mut intersector = BRepClassIntersector::new();
        attempt(|| {
            intersector.perform(&Line2::new(p, d), par, tol, &e)?;
            inter(&intersector)
        });
        int(e.bounding_box_state() as i32);
        // Without a pcurve LocalGeometry dereferences a null curve natively (a crash).
        let has_pcurve = !BRepTool::curve_on_face(e.edge(), e.face())?
            .pcurve
            .is_null();
        flag(has_pcurve);
        let (a, b) = (e.first_parameter(), e.last_parameter());
        if has_pcurve {
            local(&intersector, &e, a);
            local(&intersector, &e, 0.5 * (a + b));
            local(&intersector, &e, b);
        }
        Ok(())
    });
    end();
}

/// `face k1 k2 P D par tol`: one intersector on the explorer edges k1 then k2.
pub fn intersect2_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let mut x = BRepClassFaceExplorer::new(&f.face)?;
        let (k1, k2) = (r.integer() as usize, r.integer() as usize);
        let (p, d, par, tol) = (r.pnt2d(), r.dir2d(), r.real(), r.real());
        let (e1, e2) = (explorer_edge(&mut x, k1)?, explorer_edge(&mut x, k2)?);
        let mut intersector = BRepClassIntersector::new();
        intersector.perform(&Line2::new(p, d), par, tol, &e1)?;
        inter(&intersector)?;
        intersector.perform(&Line2::new(p, d), par, tol, &e2)?;
        inter(&intersector)
    });
    end();
}

/// The members of `tp_classification` a classifier exposes.
trait Classification {
    fn state(&self) -> State;
    fn rejected(&self) -> bool;
    fn no_wires(&self) -> bool;
    fn edge(&self) -> Result<&BRepClassEdge, Failure>;
    fn edge_parameter(&self) -> Result<f64, Failure>;
    fn position(&self) -> Position;
}

macro_rules! classification {
    ($class:ty) => {
        impl Classification for $class {
            fn state(&self) -> State {
                <$class>::state(self)
            }
            fn rejected(&self) -> bool {
                <$class>::rejected(self)
            }
            fn no_wires(&self) -> bool {
                <$class>::no_wires(self)
            }
            fn edge(&self) -> Result<&BRepClassEdge, Failure> {
                <$class>::edge(self)
            }
            fn edge_parameter(&self) -> Result<f64, Failure> {
                <$class>::edge_parameter(self)
            }
            fn position(&self) -> Position {
                <$class>::position(self)
            }
        }
    };
}
classification!(FClassifier);
classification!(BRepClassFaceClassifier);

/// `tp_classification`: state, rejected, no wires, edge, parameter and (edge set) position.
fn classification(f: &TpFace, c: &impl Classification) {
    int(c.state() as i32);
    flag(c.rejected());
    flag(c.no_wires());
    let mut edge = -3;
    attempt(|| {
        edge = tp_edge_index(&f.face, c.edge()?.edge());
        int(edge);
        Ok(())
    });
    attempt(|| {
        real(c.edge_parameter()?);
        Ok(())
    });
    if edge >= 0 || edge == -2 {
        int(c.position() as i32);
    }
}

/// `face u v tol usebox gaptol`: FClassifier over an explorer, then the FaceClassifier
/// over the face and over an explorer.
pub fn classify_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let (p, tol, bounds, gap) = (r.pnt2d(), r.real(), r.flag(), r.real());
        attempt(|| {
            let mut x = BRepClassFaceExplorer::new(&f.face)?;
            x.set_max_tolerance(gap);
            x.set_use_bnd_box(bounds);
            classification(&f, &FClassifier::with_point(&mut x, &p, tol)?);
            Ok(())
        });
        attempt(|| {
            let c = BRepClassFaceClassifier::with_face_point2d(&f.face, &p, tol, bounds, gap)?;
            classification(&f, &c);
            Ok(())
        });
        attempt(|| {
            let mut x = BRepClassFaceExplorer::new(&f.face)?;
            x.set_max_tolerance(gap);
            x.set_use_bnd_box(bounds);
            classification(
                &f,
                &BRepClassFaceClassifier::with_explorer(&mut x, &p, tol)?,
            );
            Ok(())
        });
        Ok(())
    });
    end();
}

/// `face x y z tol usebox gaptol`: BRepClass_FaceClassifier of a 3d point.
pub fn classify3d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let (p, tol, bounds, gap) = (r.pnt(), r.real(), r.flag(), r.real());
        let c = BRepClassFaceClassifier::with_face_point3d(&f.face, &p, tol, bounds, gap)?;
        classification(&f, &c);
        Ok(())
    });
    end();
}

fn passive_state(c: &FacePassiveClassifier) {
    int(c.closest_intersection());
    int(c.state() as i32);
    real(c.parameter());
    flag(c.is_head_or_end());
}

/// `tp_passive`: Reset, then Compare over the explorer edges.
fn passive(f: &TpFace, r: &mut Reader<'_>) -> Result<(), Failure> {
    let mut c = FClass2dOfFClassifier::new();
    passive_state(&c);
    flag(c.intersector().is_done());
    let mut x = BRepClassFaceExplorer::new(&f.face)?;
    let (p, d, par) = (r.pnt2d(), r.dir2d(), r.real());
    c.reset(Line2::new(p, d), par, r.real());
    passive_state(&c);
    x.init_wires();
    while x.more_wires() {
        x.init_edges();
        while x.more_edges() {
            let (e, o) = x.current_edge()?;
            attempt(|| {
                c.compare(&e, o)?;
                passive_state(&c);
                Ok(())
            });
            x.next_edge();
        }
        x.next_wire();
    }
    Ok(())
}

/// `face mode P D par tol`: FacePassiveClassifier (0) or FClass2dOfFClassifier (1); both
/// are the same TopClass instance in the port.
pub fn passive_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        r.flag();
        passive(&f, &mut r)
    });
    end();
}

/// No fields: the default classifiers, intersector and edge.
pub fn defaults_case(_values: &[f64]) {
    let a = BRepClassFaceClassifier::new();
    int(a.state() as i32);
    flag(a.rejected());
    flag(a.no_wires());
    attempt(|| {
        flag(a.edge()?.edge().is_null());
        Ok(())
    });
    attempt(|| {
        real(a.edge_parameter()?);
        Ok(())
    });
    let b = FClassifier::new();
    int(b.state() as i32);
    flag(b.rejected());
    flag(b.no_wires());
    attempt(|| {
        real(b.edge()?.max_tolerance());
        Ok(())
    });
    attempt(|| {
        real(b.edge_parameter()?);
        Ok(())
    });
    flag(BRepClassIntersector::new().is_done());
    let e = BRepClassEdge::new();
    real(e.max_tolerance());
    flag(e.use_bnd_box());
    int(e.bounding_box_state() as i32);
    end();
}
