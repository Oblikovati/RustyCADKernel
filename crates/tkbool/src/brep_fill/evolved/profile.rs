// Rust translation of BRepFill_Evolved::PrepareProfile, PrepareSpine and the static
// CutEdgeProf and CutEdge of ModelingAlgorithms/TKBool/BRepFill/BRepFill_Evolved.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The preparation of the profile (projected on yOz and cut at its extrema of distance
//! to Oz and at Oz, split into wires on one side of Oz, planar and vertical parts apart)
//! and of the spine (its edges cut at the extrema of curvature and inflexions).
use super::statics::{
    bubble_reals, confusion, distance_to_oz, edge_vertices, is_planar, is_vertical,
};
use super::{BRepFillEvolved, ShapeOrigins};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomGeometry, GeomPlane, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom2d_api::ExtremaCurveCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{self, Axis3, Point3};
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeEdge, BRepLibMakeVertex};
use rustycad_tktopalgo::mat2d::Mat2dCutCurve;

type ProfileProjection = (Handle<dyn GeomCurve>, Handle<dyn Geom2dCurve>);

/// The projection of the edge curve on the plane yOz: the 3d projected curve and its
/// p-curve on the plane.
fn project_on_yoz(e: &Edge, plane: &Handle<GeomPlane>) -> Result<ProfileProjection, Failure> {
    let found = BRepTool::curve_with_location(e)?;
    let ct = Handle::new(GeomTrimmedCurve::new(
        &found.curve,
        found.first,
        found.last,
        true,
        true,
    )?);
    ct.transform(found.location.transformation())?;
    let normal = plane
        .get()
        .ok_or_else(|| Failure::new("CutEdgeProf: null plane"))?
        .pln()
        .axis()
        .direction();
    let c = geom_proj_lib::project_on_plane(&ct.cast(), plane, normal, false)?;
    let surface: Handle<dyn GeomSurface> = plane.cast();
    let c2d = geom_proj_lib::curve2d(&c, &surface)?;
    Ok((c, c2d))
}

/// The parameters on `c2d` of its extrema of distance to the line and of its
/// intersections with it.
fn cut_parameters(
    line: &Handle<dyn Geom2dCurve>,
    c2d: &Handle<dyn Geom2dCurve>,
) -> Result<Vec<f64>, Failure> {
    let mut seq: Vec<f64> = Vec::new();
    let (f, l) = (c2d.first_parameter(), c2d.last_parameter());
    let ac2d = Geom2dAdaptorCurve::with_curve(c2d)?;
    let mut b = BoundingBox2::default();
    Add2dCurve::add_adaptor(&ac2d, confusion(), &mut b)?;
    let [_, u1, _, u2] = b.get()?;
    // no sense if C2 is normal to Line or really is a point (NIZHNY-EAP 2000-02-02)
    if u1 != u2 {
        let extrema = ExtremaCurveCurve::new(line, c2d, u1 - 1., u2 + 1., f, l)?;
        for i in 1..=extrema.nb_extrema() {
            seq.push(extrema.parameters(i)?.1);
        }
    }
    let a_line = Geom2dAdaptorCurve::with_curve(line)?;
    let intersector = GInter::between(&a_line, &ac2d, 0., precision::intersection())?;
    for i in 1..=intersector.nb_points()? {
        seq.push(intersector.point(i)?.param_on_second());
    }
    Ok(seq)
}

/// The projected vertex of a profile vertex, made once.
fn moved_vertex(
    vr: &Vertex,
    p: Point3,
    map_ver_ref_moved: &mut ShapeOrigins,
) -> Result<Vertex, Failure> {
    if let Some(v) = map_ver_ref_moved.seek(vr.as_ref()) {
        return topods::vertex(v);
    }
    let mut v = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut v, p, BRepTool::vertex_tolerance(vr)?)?;
    map_ver_ref_moved.bind(vr.as_ref().clone(), v.as_ref().clone());
    Ok(v)
}

/// The cut piece appended (FORWARD) or prepended with the orientation of `e`.
fn push_piece(
    cuts: &mut List<Shape>,
    c: &Handle<dyn GeomCurve>,
    v1: &Vertex,
    v2: &Vertex,
    e: &Edge,
) -> Result<(), Failure> {
    let mut ee = BRepLibMakeEdge::from_curve_vertices(c, v1, v2)?
        .edge()?
        .into_shape();
    ee.set_orientation(e.as_ref().orientation());
    if ee.orientation() == Orientation::Forward {
        cuts.append(ee);
    } else {
        cuts.prepend(ee);
    }
    Ok(())
}

/// `CutEdgeProf(E, Plane, Line, Cuts, MapVerRefMoved)` (static): the edge projected on
/// the plane and cut at its extrema of distance to the line and at its intersections
/// with it (the projected end vertices shared through `map_ver_ref_moved`).
fn cut_edge_prof(
    e: &Edge,
    plane: &Handle<GeomPlane>,
    line: &Handle<dyn Geom2dCurve>,
    cuts: &mut List<Shape>,
    map_ver_ref_moved: &mut ShapeOrigins,
) -> Result<(), Failure> {
    cuts.clear();
    let (c, c2d) = project_on_yoz(e, plane)?;
    let (f, l) = (c2d.first_parameter(), c2d.last_parameter());
    let mut seq = cut_parameters(line, &c2d)?;
    let (vrf, vrl) = topexp::vertices(e, false);
    let mut vf = moved_vertex(&vrf, c.value(f)?, map_ver_ref_moved)?;
    let vl = moved_vertex(&vrl, c.value(l)?, map_ver_ref_moved)?;
    bubble_reals(&mut seq);
    let mut cur_param = f;
    for param in seq {
        if (param - cur_param).abs() > confusion() && (param - l).abs() > confusion() {
            let vv = BRepLibMakeVertex::new(c.value(param)?)?.vertex()?;
            push_piece(cuts, &c, &vf, &vv, e)?;
            (cur_param, vf) = (param, vv);
        }
    }
    push_piece(cuts, &c, &vf, &vl, e)
}

/// The profile wires under construction (`WP` natively) with the current one.
struct ProfileWires {
    wires: List<Shape>,
    current: Shape,
}

impl ProfileWires {
    /// A list with one new empty wire.
    fn new() -> Self {
        let mut pw = Self {
            wires: List::default(),
            current: Shape::default(),
        };
        pw.start_wire();
        pw
    }

    /// A new empty wire, current and appended.
    fn start_wire(&mut self) {
        let mut w = Wire::default();
        BRepBuilder::default().make_wire(&mut w);
        self.current = w.into_shape();
        self.wires.append(self.current.clone());
    }

    /// `e` added to the current wire.
    fn add(&mut self, e: &Shape) -> Result<(), Failure> {
        BRepBuilder::default().add(&mut self.current, e)
    }
}

/// The parts of a wire with planar or vertical edges: a new wire at each change of
/// kind (vertical, planar, other); the wire itself when it has no such edge.
fn split_by_kind(cur_w: &Shape, work_prof: &mut List<Shape>) -> Result<(), Failure> {
    let edges: Vec<Edge> = Explorer::new(cur_w, ShapeEnum::Edge, ShapeEnum::Shape)?
        .map(|e| topods::edge(&e))
        .collect::<Result<_, _>>()?;
    let mut kinds = Vec::with_capacity(edges.len());
    for ee in &edges {
        kinds.push(if is_vertical(ee)? {
            3
        } else if is_planar(ee)? {
            2
        } else {
            1
        });
    }
    if !kinds.iter().any(|k| *k != 1) {
        work_prof.append(cur_w.clone());
        return Ok(());
    }
    let bb = BRepBuilder::default();
    let (mut status, mut nw) = (0, Shape::default());
    for (ee, kind) in edges.iter().zip(kinds) {
        if kind != status {
            let mut w = Wire::default();
            bb.make_wire(&mut w);
            nw = w.into_shape();
            work_prof.append(nw.clone());
            status = kind;
        }
        bb.add(&mut nw, ee.as_ref())?;
    }
    Ok(())
}

impl BRepFillEvolved {
    /// The pieces of one profile edge added to the wires: a new wire after a piece
    /// ending on Oz (after the edge for its last piece); whether a new wire is due.
    fn add_profile_pieces(
        e: &Edge,
        cuts: &mut List<Shape>,
        wp: &mut ProfileWires,
        map_prof: &mut ShapeOrigins,
    ) -> Result<bool, Failure> {
        let mut new_wire = false;
        while let Some(first) = cuts.first().cloned() {
            let ne = topods::edge(&first)?;
            map_prof.bind(ne.as_ref().clone(), e.as_ref().clone());
            let (v1, v2) = edge_vertices(&ne);
            for v in [&v1, &v2] {
                if !map_prof.is_bound(v.as_ref()) {
                    map_prof.bind(v.as_ref().clone(), e.as_ref().clone());
                }
            }
            wp.add(ne.as_ref())?;
            cuts.remove_first()?;
            if distance_to_oz(&v2)? < confusion() && distance_to_oz(&v1)? > confusion() {
                if cuts.is_empty() {
                    new_wire = true;
                } else {
                    wp.start_wire();
                }
            }
        }
        Ok(new_wire)
    }

    /// `PrepareProfile(WorkProf, MapProf)` (private): the profile projected on yOz and
    /// cut, split into wires ending on Oz, then into planar, vertical and other parts;
    /// `map_prof` binds the new edges and vertices to their origin.
    pub(super) fn prepare_profile(
        &self,
        work_prof: &mut List<Shape>,
        map_prof: &mut ShapeOrigins,
    ) -> Result<(), Failure> {
        let plane = Handle::new(GeomPlane::new(Axis3::from_axis2(gp::yoz())));
        let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(gp::oy2d())).cast();
        let mut map_ver_ref_moved = ShapeOrigins::default();
        let mut wp = ProfileWires::new();
        let mut exp = WireExplorer::with_wire(&self.my_profile)?;
        while exp.more() {
            let e = exp.current().clone();
            let mut cuts = List::default();
            cut_edge_prof(&e, &plane, &line, &mut cuts, &mut map_ver_ref_moved)?;
            let new_wire = if cuts.is_empty() {
                wp.add(e.as_ref())?;
                map_prof.bind(e.as_ref().clone(), e.as_ref().clone());
                false
            } else {
                Self::add_profile_pieces(&e, &mut cuts, &mut wp, map_prof)?
            };
            exp.next()?;
            if exp.more() && new_wire {
                wp.start_wire();
            }
        }
        for cur_w in wp.wires.iter() {
            split_by_kind(cur_w, work_prof)?;
        }
        for (key, value) in map_ver_ref_moved.items() {
            map_prof.bind(value.clone(), key.clone());
        }
        Ok(())
    }

    /// The edges of one wire of the spine cut, added to `nw` and bound to their origin.
    fn prepare_spine_wire(
        &self,
        w: &Shape,
        nw: &mut Shape,
        map_spine: &mut ShapeOrigins,
    ) -> Result<(), Failure> {
        let bb = BRepBuilder::default();
        for s in ShapeIterator::new(w, true, true) {
            let e = topods::edge(&s)?;
            let (v1, v2) = edge_vertices(&e);
            map_spine.bind(v1.as_ref().clone(), v1.as_ref().clone());
            map_spine.bind(v2.as_ref().clone(), v2.as_ref().clone());
            let mut cuts = List::default();
            cut_edge(&e, &self.my_spine, &mut cuts)?;
            if cuts.is_empty() {
                bb.add(nw, e.as_ref())?;
                map_spine.bind(e.as_ref().clone(), e.as_ref().clone());
                continue;
            }
            for ne in cuts.iter() {
                bb.add(nw, ne)?;
                map_spine.bind(ne.clone(), e.as_ref().clone());
                let (v1, v2) = edge_vertices(&topods::edge(ne)?);
                for v in [v1, v2] {
                    if !map_spine.is_bound(v.as_ref()) {
                        map_spine.bind(v.as_ref().clone(), e.as_ref().clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// `PrepareSpine(WorkSpine, MapSpine)` (private): a face on the spine surface with
    /// the wires of its cut edges (closed flags kept), with 3d curves.
    pub(super) fn prepare_spine(
        &self,
        work_spine: &mut Face,
        map_spine: &mut ShapeOrigins,
    ) -> Result<(), Failure> {
        let bb = BRepBuilder::default();
        let (s, l) = BRepTool::surface_with_location(&self.my_spine)?;
        let tol_f = BRepTool::face_tolerance(&self.my_spine)?;
        bb.make_face_on_located_surface(work_spine, &s, &l, tol_f)?;
        let mut ws = work_spine.clone().into_shape();
        for w in ShapeIterator::new(self.my_spine.as_ref(), true, true) {
            let mut nw = Wire::default();
            bb.make_wire(&mut nw);
            let mut nw = nw.into_shape();
            let is_closed = w.closed()?;
            self.prepare_spine_wire(&w, &mut nw, map_spine)?;
            nw.set_closed(is_closed)?;
            bb.add(&mut ws, &nw)?;
        }
        *work_spine = topods::face(&ws)?;
        brep_lib::build_curves3d_default(work_spine.as_ref())?;
        Ok(())
    }
}

/// An edge copy of `e` (FORWARD) between `v1` and `v2` over `range`.
fn piece_of(e: &Edge, v1: &Shape, v2: &Shape, range: (f64, f64)) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut ne = e.as_ref().empty_copied()?;
    ne.set_orientation(Orientation::Forward);
    b.add(&mut ne, v1)?;
    b.add(&mut ne, v2)?;
    b.range(&topods::edge(&ne)?, range.0, range.1, false)?;
    Ok(ne.oriented(e.as_ref().orientation()))
}

/// The vertex at a 2d point of the plane z = 0.
fn vertex_at_uv(c: &Handle<Geom2dTrimmedCurve>, u: f64) -> Result<Shape, Failure> {
    let p = c
        .get()
        .ok_or_else(|| Failure::new("CutEdge: null curve"))?
        .value(u)?;
    Ok(BRepLibMakeVertex::new(Point3::new(p.x(), p.y(), 0.))?
        .vertex()?
        .into_shape())
}

/// The closed circle cut in three at the thirds of its range.
fn cut_closed_circle(
    e: &Edge,
    ct2d: &Handle<Geom2dTrimmedCurve>,
    (f, l): (f64, f64),
    cuts: &mut List<Shape>,
) -> Result<(), Failure> {
    let (v1, v2) = topexp::vertices(e, false);
    let (m1, m2) = ((2. * f + l) / 3., (f + 2. * l) / 3.);
    let (vl1, vl2) = (vertex_at_uv(ct2d, m1)?, vertex_at_uv(ct2d, m2)?);
    cuts.append(piece_of(
        e,
        v1.as_ref(),
        &vl1.oriented(Orientation::Reversed),
        (f, m1),
    )?);
    cuts.append(piece_of(
        e,
        &vl1.oriented(Orientation::Forward),
        &vl2.oriented(Orientation::Reversed),
        (m1, m2),
    )?);
    cuts.append(piece_of(
        e,
        &vl2.oriented(Orientation::Forward),
        v2.as_ref(),
        (m2, l),
    )?);
    Ok(())
}

/// `CutEdge(E, F, Cuts)` (static): the edge cut at the extrema of curvature and the
/// inflexions of its p-curve (a closed circle in three); empty when not cut.
fn cut_edge(e: &Edge, f: &Face, cuts: &mut List<Shape>) -> Result<(), Failure> {
    cuts.clear();
    let (v1, v2) = topexp::vertices(e, false);
    let pc = BRepTool::curve_on_face(e, f)?;
    let ct2d = Handle::new(Geom2dTrimmedCurve::new(
        &pc.pcurve, pc.first, pc.last, true, true,
    )?);
    let basis = ct2d
        .get()
        .ok_or_else(|| Failure::new("CutEdge: null p-curve"))?
        .basis_curve();
    if basis.is_kind_name("Geom2d_Circle") && BRepTool::is_closed(e.as_ref())? {
        return cut_closed_circle(e, &ct2d, (pc.first, pc.last), cuts);
    }
    let mut cuter = Mat2dCutCurve::new();
    cuter.perform(&ct2d.cast())?;
    if cuter.un_modified() {
        return Ok(());
    }
    let mut vf = v1.as_ref().clone();
    let n = cuter.nb_curves()?;
    for k in 1..=n {
        let cc = cuter.value(k)?;
        let ccr = cc
            .get()
            .ok_or_else(|| Failure::new("CutEdge: null cut curve"))?;
        let vl = if k == n {
            v2.as_ref().clone()
        } else {
            vertex_at_uv(&cc, ccr.last_parameter())?
        };
        let range = (ccr.first_parameter(), ccr.last_parameter());
        cuts.append(piece_of(
            e,
            &vf.oriented(Orientation::Forward),
            &vl.oriented(Orientation::Reversed),
            range,
        )?);
        vf = vl;
    }
    Ok(())
}
