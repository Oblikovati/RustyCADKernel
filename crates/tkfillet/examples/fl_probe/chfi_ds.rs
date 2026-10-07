//! The ChFiDS probes, the Rust twin of tools/oracle/tkfillet_probe_chfids.cxx: the
//! guideline abscissa and evaluation of `ChFiDS_Spine` and the radius laws of
//! `ChFiDS_FilSpine`.
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::chfi_ds::{ElSpine, FilSpine, Spine, SpineObject, State};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3, Vector3, Xy};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

fn vec3(v: Vector3) {
    real(v.x());
    real(v.y());
    real(v.z());
}

/// `fl_segment`: the segment `a`-`b`, or `b`-`a` reversed.
fn segment(a: Point3, b: Point3, reversed: bool) -> Result<Edge, Failure> {
    if !reversed {
        return BRepBuilderAPIMakeEdge::from_points(a, b)?.edge();
    }
    let mut edge = BRepBuilderAPIMakeEdge::from_points(b, a)?.edge()?;
    edge.reverse();
    Ok(edge)
}

/// `fl_load`: the closed state, Load, the length and the tolerance.
fn load(spine: &mut Spine, closed: bool) {
    if closed {
        spine.set_first_status(State::Closed);
    }
    attempt(|| spine.load());
    attempt(|| spine.last_parameter().map(real));
    real(spine.get_tolesp());
}

/// `fl_spine_at`: Index, D2, D1, both Parameter flavours and Absc at `w`.
fn spine_at(spine: &mut Spine, w: f64) {
    attempt(|| spine.index_at(w, true).map(int));
    attempt(|| spine.index_at(w, false).map(int));
    attempt(|| {
        let (p, v1, v2) = spine.d2(w)?;
        pnt(p);
        vec3(v1);
        vec3(v2);
        Ok(())
    });
    attempt(|| {
        let (p, v1) = spine.d1(w)?;
        pnt(p);
        vec3(v1);
        Ok(())
    });
    attempt(|| spine.parameter(w, true).map(real));
    attempt(|| {
        let u = spine.parameter(w, false)?;
        real(u);
        real(spine.absc(u)?);
        Ok(())
    });
}

/// `fl_spine N P1..PN R closed tgtFirst tgtLast refMode ref M W1..WM`.
pub fn spine_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let points: Vec<Point3> = (0..n).map(|_| r.pnt()).collect();
    let reversed_bits = r.integer();
    let (closed, tangent_first, tangent_last) = (r.flag(), r.flag(), r.flag());
    let (reference_mode, reference) = (r.integer(), r.real());
    let ws = r.reals();
    let mut spine = Spine::new();
    for (i, pair) in points.windows(2).enumerate() {
        let edge = segment(pair[0], pair[1], (reversed_bits >> i) & 1 == 1).expect("probe segment");
        spine.set_edges(&edge);
    }
    load(&mut spine, closed);
    extend_and_refer(
        &mut spine,
        (tangent_first, tangent_last),
        reference_mode,
        reference,
    );
    for w in ws {
        spine_at(&mut spine, w);
    }
    end();
}

/// The tangent extensions and the reference of `fl_spine`.
fn extend_and_refer(spine: &mut Spine, tangents: (bool, bool), mode: i32, reference: f64) {
    if tangents.0 {
        attempt(|| spine.set_first_tgt(spine.first_parameter()));
    }
    if tangents.1 {
        attempt(|| {
            let last = spine.last_parameter()?;
            spine.set_last_tgt(last)
        });
    }
    if mode == 1 {
        attempt(|| spine.set_reference(reference));
    }
    if mode == 2 {
        attempt(|| spine.set_reference_on_edge(reference as i32));
    }
}

/// `fl_arc_spine C r a0 a1 reversed M W1..WM`.
pub fn arc_spine_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let center = r.pnt();
    let (radius, a0, a1, reversed) = (r.real(), r.real(), r.real(), r.flag());
    let ws = r.reals();
    let frame = Axis2::new(center, Direction3::new(0., 0., 1.).expect("Z")).expect("frame");
    let circle = Circle3::new(frame, radius).expect("probe circle");
    let mut edge = BRepBuilderAPIMakeEdge::from_circ_params(circle, a0, a1)
        .and_then(|maker| maker.edge())
        .expect("probe arc");
    if reversed {
        edge.reverse();
    }
    let mut spine = Spine::new();
    spine.set_edges(&edge);
    load(&mut spine, false);
    attempt(|| spine.current_elementary_spine(1).map(|_| ()));
    attempt(|| spine.get_type().map(|kind| int(kind as i32)));
    attempt(|| print_circle(spine.circle()?));
    for w in ws {
        spine_at(&mut spine, w);
    }
    end();
}

fn print_circle(circle: Circle3) -> Result<(), Failure> {
    let frame = circle.position();
    pnt(frame.location());
    dir(frame.direction());
    dir(frame.x_direction());
    real(circle.radius());
    Ok(())
}

/// `fl_radii`: IsConstant, then per edge IsConstant and Radius.
fn radii(spine: &FilSpine) {
    flag(spine.is_constant());
    attempt(|| spine.radius().map(real));
    for i in 1..=spine.nb_edges() {
        attempt(|| spine.is_constant_on_edge(i).map(flag));
        attempt(|| spine.radius_on_edge_index(i).map(real));
    }
}

/// `fl_fil_law N closed P (U R)*P E (I R)*E deb fin periodic M X1..XM`.
pub fn fil_law_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (n, closed) = (r.integer(), r.flag());
    let mut spine = FilSpine::new();
    for i in 0..n {
        let x = f64::from(i);
        let edge = segment(Point3::new(x, 0., 0.), Point3::new(x + 1., 0., 0.), false);
        spine.set_edges(&edge.expect("probe segment"));
    }
    load(&mut spine, closed);
    set_radii(&mut spine, &mut r);
    let (deb, fin, periodic) = (r.real(), r.real(), r.flag());
    let xs = r.reals();
    radii(&spine);
    let el_spine = el_spine(deb, fin, periodic);
    attempt(|| sample_law(&mut spine, &el_spine, &xs));
    attempt(|| spine.reset(false));
    radii(&spine);
    end();
}

/// The pairs and edge radii of `fl_fil_law`.
fn set_radii(spine: &mut FilSpine, r: &mut Reader<'_>) {
    for _ in 0..r.integer() {
        let (u, radius) = (r.real(), r.real());
        attempt(|| spine.set_radius_at(Xy::new(u, radius), 0));
    }
    for _ in 0..r.integer() {
        let (index, radius) = (r.integer(), r.real());
        attempt(|| {
            let edge = spine.edges(index)?.clone();
            spine.set_radius_on_edge(radius, &edge)
        });
    }
}

/// The elementary spine of `fl_fil_law`: a line along X on `[deb, fin]`.
fn el_spine(deb: f64, fin: f64, periodic: bool) -> Handle<ElSpine> {
    let line = GeomLine::from_point(Point3::default(), Direction3::new(1., 0., 0.).expect("X"));
    let curve: Handle<dyn GeomCurve> = Handle::new(line).cast();
    let el_spine = ElSpine::new();
    el_spine.set_curve(&curve).expect("probe line");
    el_spine.set_first_parameter(deb);
    el_spine.set_last_parameter(fin);
    el_spine.set_periodic(periodic);
    Handle::new(el_spine)
}

/// AppendElSpine, the law of the elementary spine sampled at `xs`, then the maximal radius.
fn sample_law(spine: &mut FilSpine, el_spine: &Handle<ElSpine>, xs: &[f64]) -> Result<(), Failure> {
    spine.append_el_spine(el_spine)?;
    let law = spine.law(el_spine)?;
    flag(law.is_null());
    if let Some(composite) = law.get() {
        for &x in xs {
            attempt(|| composite.value(x).map(real));
        }
    }
    real(spine.max_rad_from_seq_and_laws()?);
    Ok(())
}
