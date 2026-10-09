// Rust translation of the static helpers AddNewEdge, SeqOfVertices, PlaneOfWire,
// WireContinuity, SearchRoot and SearchVertex
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_CompatibleWires.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The helpers of `BRepFill_CompatibleWires`: vertex sequences, the mean plane of a
//! wire, the continuity of a wire, and the searches in the vertex correspondence maps.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_lprop::continuity_with_tolerances;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity};
use rustycad_tkmath::gp::{Axis3, Direction3, Plane3, Point3, Vector3};
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use rustycad_tktopalgo::brep_lib::BRepLibFindSurface;

/// The vertex correspondence maps (`NCollection_DataMap<TopoDS_Shape, TopTools_ListOfShape>`).
pub(super) type VertexLists = DataMap<Shape, List<Shape>, ShapeMapHasher>;
/// The new edges of each cut edge (`NCollection_DataMap<TopoDS_Shape, TopTools_SequenceOfShape>`).
pub(super) type EdgeNewEdges = DataMap<Shape, Vec<Shape>, ShapeMapHasher>;

/// The item `i` (1-based) of a sequence; Standard_OutOfRange outside it (the native
/// sequences check their index only in debug builds).
pub(super) fn nth<'a, T>(items: &'a [T], i: usize, what: &str) -> Result<&'a T, Failure> {
    items.get(i.wrapping_sub(1)).ok_or_else(|| {
        let message = format!(
            "BRepFill_CompatibleWires: {what} {i} of {}; expected 1..=count",
            items.len()
        );
        Failure::with_type("Standard_OutOfRange", &message, "BRepFill_CompatibleWires")
    })
}

/// `AddNewEdge`: the newest edges of `edge`, followed recursively through the cuts.
pub(super) fn add_new_edge(edge: &Shape, new_edges: &EdgeNewEdges, list: &mut List<Shape>) {
    match new_edges.seek(edge) {
        Some(cuts) => {
            for cut in cuts {
                add_new_edge(cut, new_edges, list);
            }
        }
        None => {
            list.append(edge.clone());
        }
    }
}

/// `SeqOfVertices`: the distinct vertices of the wire in TopExp_Explorer order.
pub(super) fn seq_of_vertices(wire: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut sequence: Vec<Shape> = Vec::new();
    for vertex in Explorer::new(wire, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        if !sequence.iter().any(|known| known.is_same(&vertex)) {
            sequence.push(vertex);
        }
    }
    Ok(sequence)
}

/// Whether a wire is closed: its flag, or the same vertex at both ends.
pub(super) fn wire_closed(wire: &Shape) -> Result<bool, Failure> {
    if wire.closed()? {
        return Ok(true);
    }
    let (v1, v2) = topexp::wire_vertices(&topods::wire(wire)?);
    Ok(v1.is_same(&v2))
}

/// The center of a closed wire of one circle or ellipse edge.
fn conic_center(wire: &Wire) -> Result<Option<Point3>, Failure> {
    if !wire_closed(wire)? {
        return Ok(None);
    }
    let mut iterator = ShapeIterator::new(wire, true, true);
    let mut first: Option<Shape> = None;
    let mut count: usize = 0;
    while iterator.more() {
        count += 1;
        first.get_or_insert(iterator.value()?.clone());
        iterator.next();
    }
    let (1, Some(edge)) = (count, first) else {
        return Ok(None);
    };
    let (curve, _, _) = BRepTool::curve(&topods::edge(&edge)?)?;
    let adaptor = GeomAdaptorCurve::default();
    adaptor.load(&curve)?;
    let mut center: Option<Point3> = None;
    if adaptor.get_type()? == CurveType::Circle {
        center = Some(adaptor.circle()?.location());
    }
    if adaptor.get_type()? == CurveType::Ellipse {
        center = Some(adaptor.ellipse()?.location());
    }
    Ok(center)
}

/// `PlaneOfWire`: the plane of the wire through its barycenter (the center of a lone
/// circle or ellipse, else the center of mass of its edges), or, for a non-planar
/// wire, the plane of its two extreme inertia axes; `None` when two radii of gyration
/// are equal (`false` natively, the plane left unchanged).
pub(super) fn plane_of_wire(wire: &Wire) -> Result<Option<Plane3>, Failure> {
    let mut props = GProps::new();
    let bary: Point3 = match conic_center(wire)? {
        Some(center) => center,
        None => {
            BRepGProp::linear_properties(wire, &mut props, false, false)?;
            props.centre_of_mass()
        }
    };
    let finder = BRepLibFindSurface::new(wire, -1., true, false)?;
    if finder.found() {
        let mut surface: Handle<dyn GeomSurface> = finder.surface();
        let location = finder.location();
        if let (false, Some(geometry)) = (location.is_identity(), surface.get()) {
            surface = geometry
                .transformed(location.transformation())?
                .cast::<dyn GeomSurface>();
        }
        let Some(plane) = surface.cast::<GeomPlane>().get().map(|p| p.pln()) else {
            return Err(Failure::with_type(
                "Standard_NullObject",
                "BRepFill_CompatibleWires PlaneOfWire: the found surface is not a plane; expected Geom_Plane",
                "BRepFill_CompatibleWires",
            ));
        };
        let mut plane = plane;
        plane.set_location(bary);
        return Ok(Some(plane));
    }
    inertia_plane(&props, bary)
}

/// The plane of a non-planar wire: normal along the axis of the largest radius of
/// gyration, X along the axis of the smallest one; `None` when two radii are equal.
fn inertia_plane(props: &GProps, bary: Point3) -> Result<Option<Plane3>, Failure> {
    let principal = props.principal_properties()?;
    let (r1, r2, r3) = principal.radius_of_gyration();
    let tol: f64 = precision::confusion();
    let rmax: f64 = r1.max(r2).max(r3);
    if ((rmax - r1).abs() < tol && (rmax - r2).abs() < tol)
        || ((rmax - r1).abs() < tol && (rmax - r3).abs() < tol)
        || ((rmax - r2).abs() < tol && (rmax - r3).abs() < tol)
    {
        return Ok(None);
    }
    let axes: [Vector3; 3] = [
        principal.first_axis_of_inertia(),
        principal.second_axis_of_inertia(),
        principal.third_axis_of_inertia(),
    ];
    let mut vec = Vector3::default();
    if r1 >= r2 && r1 >= r3 {
        vec = axes[0];
    } else if r2 >= r1 && r2 >= r3 {
        vec = axes[1];
    } else if r3 >= r1 && r3 >= r2 {
        vec = axes[2];
    }
    let n_dir = Direction3::from_vector(vec)?;
    if r3 <= r2 && r3 <= r1 {
        vec = axes[2];
    } else if r2 <= r1 && r2 <= r3 {
        vec = axes[1];
    } else if r1 <= r2 && r1 <= r3 {
        vec = axes[0];
    }
    let x_dir = Direction3::from_vector(vec)?;
    let frame = Axis3::new(bary, n_dir, x_dir)?;
    // `Geom_Plane GPlan(repere); P = GPlan.Pln();`
    Ok(Some(GeomPlane::new(frame).pln()))
}

/// `WireContinuity`: the lowest continuity at the vertices of a wire without
/// degenerated edge (CN otherwise); the closing vertex counts only when the ends meet.
pub(super) fn wire_continuity(wire: &Wire) -> Result<Continuity, Failure> {
    let mut cont_w = Continuity::CN;
    let mut edges: Vec<Edge> = Vec::new();
    let mut is_degenerated: bool = false;
    let mut explorer = WireExplorer::with_wire(wire)?;
    while explorer.more() {
        edges.push(explorer.current().clone());
        if BRepTool::degenerated(explorer.current())? {
            is_degenerated = true;
        }
        explorer.next()?;
    }
    if is_degenerated {
        return Ok(cont_w);
    }
    let nb_edges: usize = edges.len();
    let mut testconti: bool = true;
    for j in 1..=nb_edges {
        let (edge1, edge2) = if j == nb_edges {
            (&edges[nb_edges - 1], &edges[0])
        } else {
            (&edges[j - 1], &edges[j])
        };
        let (_, v1) = topexp::vertices(edge1, true);
        let (v2, _) = topexp::vertices(edge2, true);
        let u1: f64 = BRepTool::parameter(&v1, edge1)?;
        let u2: f64 = BRepTool::parameter(&v2, edge2)?;
        let curve1 = BRepAdaptorCurve::with_edge(edge1)?;
        let curve2 = BRepAdaptorCurve::with_edge(edge2)?;
        let eps: f64 = BRepTool::vertex_tolerance(&v2)? + BRepTool::vertex_tolerance(&v1)?;
        if j == nb_edges {
            testconti = curve1.value(u1)?.is_equal(&curve2.value(u2)?, eps);
        }
        if testconti {
            let cont =
                continuity_with_tolerances(&curve1, &curve2, u1, u2, eps, precision::angular())?;
            if cont <= cont_w {
                cont_w = cont;
            }
        }
    }
    Ok(cont_w)
}

/// `SearchRoot`: the key of the first list (in map order) holding `vertex`.
pub(super) fn search_root(vertex: &Shape, map: &VertexLists) -> Option<Vertex> {
    for (key, list) in map.items() {
        if list.iter().any(|v| v.is_same(vertex)) {
            return Some(Vertex::unchecked(key.clone()));
        }
    }
    None
}

/// `SearchVertex`: the first vertex of the wire (SeqOfVertices order) in the list.
pub(super) fn search_vertex(list: &List<Shape>, wire: &Shape) -> Result<Option<Vertex>, Failure> {
    for vi in seq_of_vertices(wire)? {
        if list.iter().any(|v| v.is_same(&vi)) {
            return Ok(Some(Vertex::unchecked(vi)));
        }
    }
    Ok(None)
}
