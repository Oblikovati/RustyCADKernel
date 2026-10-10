// Rust translation of the static helpers Oriente, UpdateEdgeOnPlane and BuildFace
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A face of the sweep from its surface and its four natural borders.
use super::helpers::number_of_poles;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_lib::IsPlanarSurface;
use rustycad_tkmath::gp::{Vector2, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeWire;
use rustycad_tktopalgo::brep_check::BRepCheckEdge;
use rustycad_tktopalgo::brep_lib::{
    BRepLibFaceError, BRepLibFindSurface, BRepLibMakeFace, BRepLibMakeWire,
};

/// The map of the edges of BuildFace to the edges BRepBuilderAPI_MakeWire put in the wire.
pub type EdgeEdgeMap = DataMap<Shape, Shape, ShapeMapHasher>;

/// `Oriente(S, E)`: the orientation of an edge of natural restriction of `s`: REVERSED
/// for a U iso, FORWARD for a V iso, reversed once more when it is not on the first
/// bound and once more when it runs against the parameter.
pub(super) fn oriente(s: &Handle<dyn GeomSurface>, e: &mut Edge) -> Result<(), Failure> {
    let surface = s.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepFill_Sweep Oriente: null surface; expected a surface",
            "BRepFill_Sweep",
        )
    })?;
    let (u_first, _, v_first, _) = surface.bounds();
    let found = BRepTool::curve_on_surface(e, s, &Location::default())?;
    let c = found.pcurve.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepFill_Sweep Oriente: no p-curve; expected a natural restriction",
            "BRepFill_Sweep",
        )
    })?;
    let (p, d) = c.d1((found.first + found.last) / 2.)?;
    let (u_ref, v_ref) = (Vector2::new(1., 0.), Vector2::new(0., 1.));
    let (isfirst, isopposite) = if d.is_parallel(v_ref, 0.1)? {
        e.set_orientation(Orientation::Reversed);
        (
            (p.x() - u_first).abs() < precision::confusion(),
            d.is_opposite(v_ref, 0.1)?,
        )
    } else {
        e.set_orientation(Orientation::Forward);
        (
            (p.y() - v_first).abs() < precision::confusion(),
            d.is_opposite(u_ref, 0.1)?,
        )
    };
    if !isfirst {
        e.reverse();
    }
    if isopposite {
        e.reverse();
    }
    Ok(())
}

/// `UpdateEdgeOnPlane(F, E, BB)`: the p-curve of the edge on the face stored on its
/// surface, the tolerance raised to the BRepCheck_Edge tolerance and the vertices
/// enlarged to 1.01 times it.
fn update_edge_on_plane(f: &Face, e: &Edge) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let c2d = BRepTool::curve_on_face(e, f)?.pcurve;
    let s = BRepTool::surface(f)?;
    let mut tol: f64 = BRepTool::edge_tolerance(e)?;
    builder.update_edge_pcurve(e, &c2d, &s, &Location::default(), tol, None)?;
    tol = tol.max(BRepCheckEdge::new(e)?.tolerance()?);
    builder.update_edge_tolerance(e, tol)?;
    tol *= 1.01;
    for v in [
        topexp::first_vertex(e, false),
        topexp::last_vertex(e, false),
    ] {
        if BRepTool::vertex_tolerance(&v)? < tol {
            builder.update_vertex_tolerance(&v, tol)?;
        }
    }
    Ok(())
}

/// The four borders of a face.
pub struct FaceBorders<'a> {
    pub e1: &'a Edge,
    pub e2: &'a Edge,
    pub e3: &'a Edge,
    pub e4: &'a Edge,
}

/// The plane of the face when the surface is planar (or, for a few poles, when its
/// borders lie on a plane); the border tolerances are restored after the search.
fn face_plane(
    s: &Handle<dyn GeomSurface>,
    b: &FaceBorders,
    tols: [f64; 4],
) -> Result<Option<Handle<GeomPlane>>, Failure> {
    let tol: f64 = tols[0].min(tols[1]).min(tols[2].min(tols[3]));
    // Exclude the cases with seam edges: they are not planar.
    if b.e1.is_same(b.e3) || b.e2.is_same(b.e4) {
        return Ok(None);
    }
    let is_planar = IsPlanarSurface::new(s, tol)?;
    if is_planar.is_planar() {
        return Ok(Some(Handle::new(GeomPlane::from_pln(is_planar.plan()?))));
    }
    let edges = [b.e1, b.e2, b.e3, b.e4];
    for e in edges {
        edge_tshape(e, "BRepFill_Sweep BuildFace")?.set_tolerance(precision::confusion());
    }
    let the_wire: Wire = BRepLibMakeWire::from_four_edges(b.e1, b.e2, b.e3, b.e4)?.wire()?;
    let mut plane: Option<Handle<GeomPlane>> = None;
    // The limitation for the CPU.
    if number_of_poles(&the_wire)? <= 100 {
        let finder = BRepLibFindSurface::new(&the_wire, -1., true, false)?;
        if finder.found() {
            plane = Some(finder.surface().cast::<GeomPlane>());
        }
    }
    let builder = BRepBuilder::default();
    for (e, t) in edges.iter().zip(tols) {
        builder.update_edge_tolerance(e, t)?;
    }
    Ok(plane)
}

/// Adds a border to the wire unless it is a degenerated edge of a planar face, and
/// records the edge the wire maker put in its place.
fn add_border(
    maker: &mut BRepBuilderAPIMakeWire,
    e: &Edge,
    is_plan: bool,
    track: bool,
    eemap: &mut EdgeEdgeMap,
) -> Result<(), Failure> {
    if is_plan && BRepTool::degenerated(e)? {
        return Ok(());
    }
    maker.add_edge(e)?;
    if !track || BRepTool::degenerated(e)? {
        return Ok(());
    }
    let ww: Wire = maker.wire()?;
    // The last edge added to the wire.
    let mut new_edge = Shape::default();
    let mut iter = ShapeIterator::new(&ww, true, true);
    while iter.more() {
        new_edge = iter.value()?.clone();
        iter.next();
    }
    if !e.is_same(&new_edge) {
        eemap.bind(e.as_ref().clone(), new_edge);
    }
    Ok(())
}

/// The wire of the borders oriented on the surface (E3 and E4 are the reversed E1
/// and E2 when they are the same edges).
fn border_wire(
    s: &Handle<dyn GeomSurface>,
    b: &FaceBorders,
    is_plan: bool,
    eemap: &mut EdgeEdgeMap,
) -> Result<Wire, Failure> {
    let mut maker = BRepBuilderAPIMakeWire::new();
    let mut e1 = b.e1.clone();
    oriente(s, &mut e1)?;
    add_border(&mut maker, &e1, is_plan, false, eemap)?;
    let mut e2 = b.e2.clone();
    oriente(s, &mut e2)?;
    add_border(&mut maker, &e2, is_plan, true, eemap)?;
    let e3 = opposite_border(s, b.e3, b.e1, &e1)?;
    add_border(&mut maker, &e3, is_plan, true, eemap)?;
    let e4 = opposite_border(s, b.e4, b.e2, &e2)?;
    add_border(&mut maker, &e4, is_plan, true, eemap)?;
    maker.wire()
}

/// The border opposite to `same`: its oriented copy reversed when it is the same edge,
/// else the border oriented on the surface.
fn opposite_border(
    s: &Handle<dyn GeomSurface>,
    border: &Edge,
    same: &Edge,
    oriented: &Edge,
) -> Result<Edge, Failure> {
    if border.is_same(same) {
        let mut e = oriented.clone();
        e.reverse();
        return Ok(e);
    }
    let mut e = border.clone();
    oriente(s, &mut e)?;
    Ok(e)
}

/// `BuildFace(S, E1, E2, E3, E4, EEmap, ExchUV, UReverse, F)`: the face on the surface
/// bounded by its natural borders (on a plane when the surface is planar, the
/// degenerated borders left out), reversed for each of `exch_uv` and `u_reverse`.
pub(super) fn build_face(
    s: &Handle<dyn GeomSurface>,
    b: &FaceBorders,
    eemap: &mut EdgeEdgeMap,
    exch_uv: bool,
    u_reverse: bool,
    f: &mut Face,
) -> Result<(), Failure> {
    let tols: [f64; 4] = [
        BRepTool::edge_tolerance(b.e1)?,
        BRepTool::edge_tolerance(b.e2)?,
        BRepTool::edge_tolerance(b.e3)?,
        BRepTool::edge_tolerance(b.e4)?,
    ];
    let the_plane = face_plane(s, b, tols)?;
    let ww: Wire = border_wire(s, b, the_plane.is_some(), eemap)?;
    match &the_plane {
        Some(plane) => planar_face(s, plane, &ww, b, tols, f)?,
        None => {
            // The standard case: the face on the surface.
            let builder = BRepBuilder::default();
            builder.make_face_on_surface(f, s, precision::confusion())?;
            builder.add(f, &ww)?;
        }
    }
    // Reorientation.
    if exch_uv {
        f.reverse();
    }
    if u_reverse {
        f.reverse();
    }
    Ok(())
}

/// The planar face: the plane oriented like the surface at its middle, the border
/// p-curves on the surface removed and replaced by those on the plane (the face is
/// left unchanged when BRepLib_MakeFace fails, as natively).
fn planar_face(
    s: &Handle<dyn GeomSurface>,
    plane: &Handle<GeomPlane>,
    ww: &Wire,
    b: &FaceBorders,
    tols: [f64; 4],
    f: &mut Face,
) -> Result<(), Failure> {
    let surface = s.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepFill_Sweep BuildFace: null surface; expected a surface",
            "BRepFill_Sweep",
        )
    })?;
    let geometry = plane.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepFill_Sweep BuildFace: null plane; expected a plane",
            "BRepFill_Sweep",
        )
    })?;
    let (u1, u2, v1, v2) = surface.bounds();
    let d1 = surface.d1((u1 + u2) / 2., (v1 + v2) / 2.)?;
    let ns: Vector3 = d1.d1u.crossed(d1.d1v);
    let np = Vector3::from(geometry.pln().axis().direction());
    if ns.dot(np) < 0. {
        geometry.u_reverse()?;
    }
    let mk_f =
        BRepLibMakeFace::from_surface_wire(&plane.clone().cast::<dyn GeomSurface>(), ww, true)?;
    if mk_f.error() != BRepLibFaceError::FaceDone {
        return Ok(());
    }
    let builder = BRepBuilder::default();
    let null_c2d: Handle<dyn Geom2dCurve> = Handle::default();
    let edges = [b.e1, b.e2, b.e3, b.e4];
    for (e, t) in edges.iter().zip(tols) {
        builder.update_edge_pcurve(e, &null_c2d, s, &Location::default(), t, None)?;
    }
    *f = mk_f.face()?;
    for e in edges {
        update_edge_on_plane(f, e)?;
    }
    Ok(())
}
