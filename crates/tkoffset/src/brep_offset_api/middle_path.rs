// Rust translation of ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_MiddlePath.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffsetAPI_MiddlePath`: the middle path of a pipe-like shape, the wire through
//! the centres of mass of its sections from a start wire (or face) to an end one.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::{self, WireExplorer};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{Array1, HArray1, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomLine, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_api::Interpolate;
use rustycad_tkgeombase::gc::{MakeCircle, MakeLine2d};
use rustycad_tkgeombase::gce::MakeLin;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3, Vector3};
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use rustycad_tktopalgo::brep_extrema::{BRepExtremaDistShapeShape, BRepExtremaSupportType};
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeEdge, BRepLibMakeFace, BRepLibMakeWire};
use std::f64::consts::PI;

/// A path: the edges of the shape from a start vertex, with vertices for the punctual
/// segments (`NCollection_Sequence<TopoDS_Shape>`).
type Path = Vec<Shape>;

/// `Seq(i)` of a 1-based sequence: Standard_OutOfRange outside 1..=Length().
fn at<T>(seq: &[T], i: i32) -> Result<&T, Failure> {
    usize::try_from(i - 1)
        .ok()
        .and_then(|k| seq.get(k))
        .ok_or_else(|| Failure::with_message("Standard_OutOfRange", "NCollection_Sequence::Value"))
}

/// `Seq.ChangeValue(i)`: Standard_OutOfRange outside 1..=Length().
fn at_mut<T>(seq: &mut [T], i: i32) -> Result<&mut T, Failure> {
    usize::try_from(i - 1)
        .ok()
        .and_then(|k| seq.get_mut(k))
        .ok_or_else(|| {
            Failure::with_message("Standard_OutOfRange", "NCollection_Sequence::ChangeValue")
        })
}

/// `Seq.InsertBefore(i, value)`: Standard_OutOfRange outside 1..=Length().
fn insert_before(seq: &mut Path, i: i32, value: Shape) -> Result<(), Failure> {
    let k = usize::try_from(i - 1)
        .ok()
        .filter(|k| *k < seq.len())
        .ok_or_else(|| {
            Failure::with_message("Standard_OutOfRange", "NCollection_Sequence::InsertBefore")
        })?;
    seq.insert(k, value);
    Ok(())
}

/// Standard_RangeError of an array with bad bounds.
fn array_failure(e: impl std::fmt::Debug) -> Failure {
    Failure::with_message(
        "Standard_RangeError",
        &format!("BRepOffsetAPI_MiddlePath: NCollection_Array1 {e:?}"),
    )
}

/// `Seq.Length()`.
fn length<T>(seq: &[T]) -> i32 {
    i32::try_from(seq.len()).unwrap_or(i32::MAX)
}

/// The object of a live handle; Standard_NullObject otherwise.
fn live<'a, T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &'a Handle<T>,
    what: &str,
) -> Result<&'a T, Failure> {
    handle.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            &format!("BRepOffsetAPI_MiddlePath: null {what}"),
        )
    })
}

/// The 3d curve of an edge, its basis curve when trimmed, and its range.
fn untrimmed_curve(an_edge: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let (mut a_curve, fpar, lpar) = BRepTool::curve(an_edge)?;
    if live(&a_curve, "curve")?.is_instance_name("Geom_TrimmedCurve") {
        a_curve = live(&a_curve.clone().cast::<GeomTrimmedCurve>(), "curve")?.basis_curve();
    }
    Ok((a_curve, fpar, lpar))
}

/// `IsLinear(anEdge, aLine)`: the edge is a line, or a Bezier or B-spline curve of two
/// poles (whose line raises StdFail_NotDone when the poles coincide).
fn is_linear(an_edge: &Edge) -> Result<bool, Failure> {
    let (a_curve, _, _) = untrimmed_curve(an_edge)?;
    let curve = live(&a_curve, "curve")?;
    if curve.is_kind_name("Geom_Line") {
        live(&a_curve.clone().cast::<GeomLine>(), "line")?.lin();
        return Ok(true);
    } else if curve.is_kind_name("Geom_BezierCurve") {
        let the_bezier = a_curve.clone().cast::<GeomBezierCurve>();
        let the_bezier = live(&the_bezier, "Bezier curve")?;
        if the_bezier.nb_poles() == 2 {
            MakeLin::from_points(the_bezier.pole(1)?, the_bezier.pole(2)?)?.value()?;
            return Ok(true);
        }
    } else if curve.is_kind_name("Geom_BSplineCurve") {
        let the_bspline = a_curve.clone().cast::<GeomBSplineCurve>();
        let the_bspline = live(&the_bspline, "B-spline curve")?;
        if the_bspline.nb_poles() == 2 {
            MakeLin::from_points(the_bspline.pole(1)?, the_bspline.pole(2)?)?.value()?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// `TypeOfEdge(anEdge)`: a line for a linear edge, else the adaptor type.
fn type_of_edge(an_edge: &Edge) -> Result<CurveType, Failure> {
    if is_linear(an_edge)? {
        return Ok(CurveType::Line);
    }
    BRepAdaptorCurve::with_edge(an_edge)?.get_type()
}

/// `TangentOfEdge(aShape, OnFirst)`: the tangent of the oriented edge at its first or
/// last end.
fn tangent_of_edge(a_shape: &Shape, on_first: bool) -> Result<Vector3, Failure> {
    let an_edge = topods::edge(a_shape)?;
    let an_or = an_edge.orientation();
    let (a_curve, fpar, lpar) = BRepTool::curve(&an_edge)?;
    let the_par = if on_first {
        if an_or == Orientation::Forward { fpar } else { lpar }
    } else if an_or == Orientation::Forward {
        lpar
    } else {
        fpar
    };
    let (_, mut the_tangent) = live(&a_curve, "curve")?.d1(the_par)?;
    if an_or == Orientation::Reversed {
        the_tangent.reverse();
    }
    Ok(the_tangent)
}

/// `IsValidEdge(theEdge, theFace)`: the edge touches the edges of the face only at its
/// own vertices.
fn is_valid_edge(the_edge: &Edge, the_face: &Face) -> Result<bool, Failure> {
    let (v1, v2) = topexp::vertices(the_edge, false);
    let tol = precision::confusion();
    for an_edge in topexp::Explorer::new(the_face.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
        let dist_mini =
            BRepExtremaDistShapeShape::with_shapes(the_edge.as_ref(), &an_edge, Default::default())?;
        if dist_mini.value()? <= tol {
            for i in 1..=dist_mini.nb_solution() {
                let the_type = dist_mini.support_type_shape2(i)?;
                if the_type == BRepExtremaSupportType::IsOnEdge {
                    return Ok(false);
                }
                // theType is "IsVertex"
                let a_vertex = dist_mini.support_on_shape2(i)?;
                if !(a_vertex.is_same(v1.as_ref()) || a_vertex.is_same(v2.as_ref())) {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}

/// `GetUnifiedWire(theWire, theUnifier)`: the wire rebuilt from the unified edges.
fn get_unified_wire(
    the_wire: &Wire,
    the_unifier: &ShapeUpgradeUnifySameDomain,
) -> Result<Wire, Failure> {
    let mut a_w_maker = BRepLibMakeWire::new();
    let mut wexp = WireExplorer::with_wire(the_wire)?;
    let mut a_generated_edges = MapOfShape::default();
    let history = the_unifier.history();
    while wexp.more() {
        let an_edge = wexp.current().clone().into_shape();
        let a_ls = live(&history, "history")?.modified(&an_edge)?;
        if !a_ls.is_empty() {
            for a_shape in a_ls {
                // wire shouldn't contain duplicated generated edges
                if a_generated_edges.add(a_shape.clone()) {
                    a_w_maker.add_edge(&topods::edge(&a_shape)?)?;
                }
            }
        } else {
            // no change, put original edge
            a_w_maker.add_edge(&topods::edge(&an_edge)?)?;
        }
        wexp.next()?;
    }
    a_w_maker.wire()
}

/// The 2d line through two points (GC_MakeLine2d), as a curve.
fn line_2d(p1: rustycad_tkmath::gp::Point2, p2: rustycad_tkmath::gp::Point2) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(MakeLine2d::from_points(p1, p2)?.value()?.cast())
}

/// `BRepLib_MakeEdge(theLine, theSurf, V1, V2, 0., len)` then `BRepLib::BuildCurve3d`.
fn edge_on_face(
    the_line: &Handle<dyn Geom2dCurve>,
    the_face: &Face,
    v1: &Vertex,
    v2: &Vertex,
    len: f64,
) -> Result<Edge, Failure> {
    let the_surf = BRepTool::surface(the_face)?;
    let new_edge =
        BRepLibMakeEdge::from_pcurve_vertices_params(the_line, &the_surf, v1, v2, 0., len)?.edge()?;
    brep_lib::build_curve3d(&new_edge, 1.0e-5, Continuity::C1, 14, 0)?;
    Ok(new_edge)
}

/// `BRepOffsetAPI_MiddlePath`: the middle path of a pipe-like shape, from a start wire
/// or face to an end wire or face (the same one for a ring).
///
/// ```
/// use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIMiddlePath;
/// use rustycad_tkbrep::topods::Shape;
/// let maker = BRepOffsetAPIMiddlePath::new(&Shape::default(), &Shape::default(), &Shape::default());
/// assert!(maker.is_err());
/// ```
pub struct BRepOffsetAPIMiddlePath {
    base: BRepBuilderAPIMakeShapeBase,
    my_initial_shape: Shape,
    my_start_wire: Wire,
    my_end_wire: Wire,
    my_closed_section: bool,
    my_closed_ring: bool,
    my_start_wire_edges: MapOfShape,
    my_end_wire_edges: MapOfShape,
    my_paths: Vec<Path>,
}

impl BRepBuilderAPIMakeShape for BRepOffsetAPIMiddlePath {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `Build(theRange)`: the paths along the shape from each start vertex, the
    /// successive sections between them up to the end wire, and the wire through the
    /// centres of mass of the sections (lines, arcs, or interpolated curves).
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        self.perform_build()
    }
}

impl BRepOffsetAPIMiddlePath {
    /// `BRepOffsetAPI_MiddlePath(aShape, StartShape, EndShape)`: the shape unified
    /// (ShapeUpgrade_UnifySameDomain) and the outer wires of the start and end faces (or
    /// the wires) rebuilt from the unified edges;
    /// `BRepOffsetAPIMiddlePath::new(&shape, &start_face, &end_face)?`.
    pub fn new(a_shape: &Shape, start_shape: &Shape, end_shape: &Shape) -> Result<Self, Failure> {
        let unifier = ShapeUpgradeUnifySameDomain::new(a_shape, true, true, false);
        unifier.build()?;
        let my_initial_shape = unifier.shape();

        let a_start_wire = if start_shape.shape_type()? == ShapeEnum::Face {
            brep_tools::outer_wire(&topods::face(start_shape)?)?
        } else {
            topods::wire(start_shape)?
        };
        let an_end_wire = if end_shape.shape_type()? == ShapeEnum::Face {
            brep_tools::outer_wire(&topods::face(end_shape)?)?
        } else {
            topods::wire(end_shape)?
        };

        let my_start_wire = get_unified_wire(&a_start_wire, &unifier)?;
        let my_end_wire = get_unified_wire(&an_end_wire, &unifier)?;

        let my_closed_section = my_start_wire.closed()?;
        let my_closed_ring = my_start_wire.is_same(my_end_wire.as_ref());
        Ok(Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_initial_shape,
            my_start_wire,
            my_end_wire,
            my_closed_section,
            my_closed_ring,
            my_start_wire_edges: MapOfShape::default(),
            my_end_wire_edges: MapOfShape::default(),
            my_paths: Vec::new(),
        })
    }

    /// `myClosedRing`: the start and end wires are the same; `maker.is_closed_ring()`.
    pub fn is_closed_ring(&self) -> bool {
        self.my_closed_ring
    }

    fn perform_build(&mut self) -> Result<(), Failure> {
        let mut start_vertices: Vec<Shape> = Vec::new();
        let mut end_vertices = MapOfShape::default();
        let mut end_edges = MapOfShape::default();
        let mut sections_edges: Vec<Vec<Shape>> = Vec::new();

        let mut wexp = WireExplorer::with_wire(&self.my_start_wire)?;
        let mut edge_seq: Vec<Shape> = Vec::new();
        while wexp.more() {
            start_vertices.push(wexp.current_vertex().clone().into_shape());
            edge_seq.push(wexp.current().clone().into_shape());
            wexp.next()?;
        }
        if !self.my_closed_section {
            start_vertices.push(wexp.current_vertex().clone().into_shape());
        }
        sections_edges.push(edge_seq.clone());

        wexp.init(&self.my_end_wire)?;
        while wexp.more() {
            end_vertices.add(wexp.current_vertex().clone().into_shape());
            end_edges.add(wexp.current().clone().into_shape());
            wexp.next()?;
        }
        if !self.my_closed_section {
            end_vertices.add(wexp.current_vertex().clone().into_shape());
        }

        for e in ShapeIterator::new(self.my_start_wire.as_ref(), true, true) {
            self.my_start_wire_edges.add(e);
        }
        for e in ShapeIterator::new(self.my_end_wire.as_ref(), true, true) {
            self.my_end_wire_edges.add(e);
        }

        let mut ve_map = IndexedDataMapOfShapeListOfShape::default();
        let mut ef_map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(&self.my_initial_shape, ShapeEnum::Vertex, ShapeEnum::Edge, &mut ve_map)?;
        topexp::map_shapes_and_ancestors(&self.my_initial_shape, ShapeEnum::Edge, ShapeEnum::Face, &mut ef_map)?;

        let mut cur_vertices = MapOfShape::default();

        // Initialization of <myPaths>
        for start_vertex in &start_vertices {
            let mut edges: Path = Vec::new();
            let le = ve_map.find_from_key(start_vertex)?;
            for item in le.iter() {
                let mut an_edge = topods::edge(item)?;
                if !self.my_start_wire_edges.contains(an_edge.as_ref()) {
                    let (v1, v2) = topexp::vertices(&an_edge, true);
                    if v1.is_same(start_vertex) {
                        cur_vertices.add(v2.clone().into_shape());
                    } else {
                        an_edge.reverse();
                        cur_vertices.add(v1.clone().into_shape());
                    }
                    edges.push(an_edge.clone().into_shape());
                    break;
                }
            }
            if !edges.is_empty() {
                self.my_paths.push(edges);
            } else {
                return Ok(());
            }
        }

        // Filling of "myPaths"
        let mut next_vertices: Vec<Shape> = Vec::new();
        loop {
            for i in 1..=length(&self.my_paths) {
                let path = at(&self.my_paths, i)?;
                let the_shape = path.last().cloned().unwrap_or_default();
                let (the_edge, the_vertex) = if the_shape.shape_type()? == ShapeEnum::Edge {
                    let the_edge = topods::edge(&the_shape)?;
                    let the_vertex = topexp::last_vertex(&the_edge, true);
                    (the_edge, the_vertex)
                } else {
                    // last segment of path was punctual
                    (topods::edge(at(path, length(path) - 1)?)?, topods::vertex(&the_shape)?)
                };

                if end_vertices.contains(the_vertex.as_ref()) {
                    continue;
                }
                let le = ve_map.find_from_key(the_vertex.as_ref())?;
                let mut next_edge_candidates = MapOfShape::default();
                for item in le.iter() {
                    let mut an_edge = topods::edge(item)?;
                    if an_edge.is_same(the_edge.as_ref()) {
                        continue;
                    }
                    let (v1, v2) = topexp::vertices(&an_edge, true);
                    let next_vertex = if v1.is_same(the_vertex.as_ref()) {
                        v2
                    } else {
                        an_edge.reverse();
                        v1
                    };
                    if !cur_vertices.contains(next_vertex.as_ref()) {
                        next_edge_candidates.add(an_edge.clone().into_shape());
                    }
                }
                if !next_edge_candidates.is_empty() {
                    let path = at_mut(&mut self.my_paths, i)?;
                    if next_edge_candidates.extent() > 1 {
                        path.push(the_vertex.clone().into_shape()); // punctual segment of path
                    } else {
                        let an_edge = topods::edge(next_edge_candidates.iter().next().ok_or_else(
                            || Failure::with_message("Standard_NoSuchObject", "NCollection_Map::Iterator::Key"),
                        )?)?;
                        path.push(an_edge.clone().into_shape());
                        let next_vertex = topexp::last_vertex(&an_edge, true);
                        next_vertices.push(next_vertex.clone().into_shape());
                    }
                }
            }
            if next_vertices.is_empty() {
                break;
            }
            for v in next_vertices.drain(..) {
                cur_vertices.add(v);
            }
        }

        // Building of set of sections
        let nb_e = length(&edge_seq);
        let nb_paths = length(&self.my_paths);
        let mut nb_ver = length(&self.my_paths);
        if self.my_closed_section {
            nb_ver += 1;
        }
        let jj = |j: i32| if j <= nb_paths { j } else { 1 };
        let mut i: i32 = 1;
        loop {
            for e in edge_seq.iter_mut() {
                e.nullify();
            }

            let mut to_insert_vertex = false;

            let mut j: i32 = 2;
            while j <= nb_ver {
                if !at(&edge_seq, j - 1)?.is_null() {
                    j += 1;
                    continue;
                }

                // for the end of initial shape
                if length(at(&self.my_paths, j - 1)?) < i {
                    let a_e1 = topods::edge(at(at(&self.my_paths, j - 1)?, i - 1)?)?;
                    let last_ver = topexp::last_vertex(&a_e1, true);
                    at_mut(&mut self.my_paths, j - 1)?.push(last_ver.clone().into_shape());
                }
                if length(at(&self.my_paths, jj(j))?) < i {
                    let a_e2 = topods::edge(at(at(&self.my_paths, jj(j))?, i - 1)?)?;
                    let last_ver = topexp::last_vertex(&a_e2, true);
                    at_mut(&mut self.my_paths, jj(j))?.push(last_ver.clone().into_shape());
                }

                if to_insert_vertex {
                    let s1 = at(at(&self.my_paths, j - 1)?, i)?.clone();
                    if s1.shape_type()? == ShapeEnum::Edge {
                        let fver = topexp::first_vertex(&topods::edge(&s1)?, true);
                        insert_before(at_mut(&mut self.my_paths, j - 1)?, i, fver.clone().into_shape())?;
                    }
                    let s2 = at(at(&self.my_paths, jj(j))?, i)?.clone();
                    if s2.shape_type()? == ShapeEnum::Edge {
                        let fver = topexp::first_vertex(&topods::edge(&s2)?, true);
                        insert_before(at_mut(&mut self.my_paths, jj(j))?, i, fver.clone().into_shape())?;
                    }
                    to_insert_vertex = false;
                }

                let s1 = at(at(&self.my_paths, j - 1)?, i)?.clone();
                let s2 = at(at(&self.my_paths, jj(j))?, i)?.clone();
                let mut e1 = Edge::default();
                let mut e2 = Edge::default();
                if s1.shape_type()? == ShapeEnum::Edge {
                    e1 = topods::edge(&s1)?;
                }
                if s2.shape_type()? == ShapeEnum::Edge {
                    e2 = topods::edge(&s2)?;
                }
                let e12 = topods::edge(at(at(&sections_edges, i)?, j - 1)?)?;
                // Find the face on which (E1 or E2) and E12 lie
                let e1_or_e2 = if e1.is_null() { e2.clone().into_shape() } else { e1.clone().into_shape() };
                if e1_or_e2.is_null() {
                    // both E1 and E2 are vertices =>
                    // => proper edge is the edge of previous section between them
                    *at_mut(&mut edge_seq, j - 1)? = e12.clone().into_shape();
                    j += 1;
                    continue;
                }
                let mut the_face = Face::default();
                {
                    let lf = ef_map.find_from_key(&e1_or_e2)?;
                    let lf2 = ef_map.find_from_key(e12.as_ref())?;
                    'faces: for a_face in lf.iter() {
                        for a_face2 in lf2.iter() {
                            if a_face.is_same(a_face2) {
                                the_face = topods::face(a_face)?;
                                break 'faces;
                            }
                        }
                    }
                }

                let prev_vertex = if e1.is_null() {
                    topods::vertex(&s1)?
                } else {
                    topexp::last_vertex(&e1, true)
                };
                let cur_vertex = if e2.is_null() {
                    topods::vertex(&s2)?
                } else {
                    topexp::last_vertex(&e2, true)
                };

                let mut proper_edge = Edge::default();
                {
                    let le = ve_map.find_from_key(prev_vertex.as_ref())?;
                    let mut edges_of_the_face = IndexedMapOfShape::default();
                    topexp::map_shapes_of_type(the_face.as_ref(), ShapeEnum::Edge, &mut edges_of_the_face)?;
                    for item in le.iter() {
                        let an_edge = topods::edge(item)?;
                        let (v1, v2) = topexp::vertices(&an_edge, false);
                        if ((v1.is_same(prev_vertex.as_ref()) && v2.is_same(cur_vertex.as_ref()))
                            || (v1.is_same(cur_vertex.as_ref()) && v2.is_same(prev_vertex.as_ref())))
                            && edges_of_the_face.contains(an_edge.as_ref()) // this condition is for a section of two edges
                            && !an_edge.is_same(e1.as_ref())
                        // the last condition is for torus-like shape
                        {
                            proper_edge = an_edge;
                            break;
                        }
                    }
                }

                if s1.shape_type()? == ShapeEnum::Vertex && s2.shape_type()? == ShapeEnum::Vertex {
                    *at_mut(&mut edge_seq, j - 1)? = proper_edge.clone().into_shape();
                    j += 1;
                    continue;
                }

                let prev_prev_ver = if e1.is_null() {
                    prev_vertex.clone()
                } else {
                    topexp::first_vertex(&e1, true)
                };
                let prev_cur_ver = if e2.is_null() {
                    cur_vertex.clone()
                } else {
                    topexp::first_vertex(&e2, true)
                };
                if proper_edge.is_null() {
                    // no connection between these two vertices
                    // Find the face on which E1, E2 and E12 lie
                    let mut list_one_face = List::default();
                    list_one_face.append(the_face.clone().into_shape());

                    if e1.is_null() || e2.is_null() {
                        if e1.is_null() {
                            e1 = topods::edge(at(at(&self.my_paths, j - 1)?, i - 1)?)?;
                        }
                        if e2.is_null() {
                            e2 = topods::edge(at(at(&self.my_paths, jj(j))?, i - 1)?)?;
                        }
                        let pc1 = BRepTool::curve_on_face(&e1, &the_face)?;
                        let pc2 = BRepTool::curve_on_face(&e2, &the_face)?;
                        let last_par1 = if e1.orientation() == Orientation::Forward { pc1.last } else { pc1.first };
                        let last_par2 = if e2.orientation() == Orientation::Forward { pc2.last } else { pc2.first };
                        let first_pnt2d = live(&pc1.pcurve, "pcurve")?.value(last_par1)?;
                        let last_pnt2d = live(&pc2.pcurve, "pcurve")?.value(last_par2)?;
                        let the_line = line_2d(first_pnt2d, last_pnt2d)?;
                        let len_ne = first_pnt2d.distance(last_pnt2d);
                        let new_edge = edge_on_face(&the_line, &the_face, &prev_vertex, &cur_vertex, len_ne)?;
                        *at_mut(&mut edge_seq, j - 1)? = new_edge.clone().into_shape();
                        ef_map.add(new_edge.clone().into_shape(), list_one_face);
                    } else {
                        // E1 is edge
                        // Extract points 2d
                        let pc1 = BRepTool::curve_on_face(&e1, &the_face)?;
                        let pc2 = BRepTool::curve_on_face(&e2, &the_face)?;
                        let (first_par1, last_par1) = if e1.orientation() == Orientation::Forward {
                            (pc1.first, pc1.last)
                        } else {
                            (pc1.last, pc1.first)
                        };
                        let (first_par2, last_par2) = if e2.orientation() == Orientation::Forward {
                            (pc2.first, pc2.last)
                        } else {
                            (pc2.last, pc2.first)
                        };
                        let pcurve1 = live(&pc1.pcurve, "pcurve")?;
                        let pcurve2 = live(&pc2.pcurve, "pcurve")?;
                        let first_pnt2d = pcurve1.value(last_par1)?;
                        let last_pnt2d = pcurve2.value(last_par2)?;
                        let the_line = line_2d(first_pnt2d, last_pnt2d)?;
                        let len_ne = first_pnt2d.distance(last_pnt2d);
                        let new_edge = edge_on_face(&the_line, &the_face, &prev_vertex, &cur_vertex, len_ne)?;
                        let prev_first_pnt2d = pcurve1.value(first_par1)?;
                        let prev_last_pnt2d = pcurve2.value(first_par2)?;
                        let line1 = line_2d(prev_first_pnt2d, last_pnt2d)?;
                        let line2 = line_2d(first_pnt2d, prev_last_pnt2d)?;
                        let len_ne1 = prev_first_pnt2d.distance(last_pnt2d);
                        let new_edge1 = edge_on_face(&line1, &the_face, &prev_prev_ver, &cur_vertex, len_ne1)?;
                        let len_ne2 = first_pnt2d.distance(prev_last_pnt2d);
                        let new_edge2 = edge_on_face(&line2, &the_face, &prev_vertex, &prev_cur_ver, len_ne2)?;
                        let good_ne = is_valid_edge(&new_edge, &the_face)?;
                        let good_ne1 = is_valid_edge(&new_edge1, &the_face)?;

                        let type_e1 = type_of_edge(&e1)?;
                        let type_e2 = type_of_edge(&e2)?;

                        let mut choose_edge = 0;

                        if !good_ne || type_e1 != type_e2 {
                            if type_e1 == type_e2 {
                                // !good_ne
                                choose_edge = if good_ne1 { 1 } else { 2 };
                            } else if type_e1 == CurveType::Line {
                                // types are different
                                choose_edge = 1;
                            } else if type_e2 == CurveType::Line {
                                choose_edge = 2;
                            }
                            // else: to be developed later...
                        }

                        if choose_edge == 0 {
                            *at_mut(&mut edge_seq, j - 1)? = new_edge.clone().into_shape();
                            ef_map.add(new_edge.clone().into_shape(), list_one_face);
                        } else if choose_edge == 1 {
                            *at_mut(&mut edge_seq, j - 1)? = new_edge1.clone().into_shape();
                            ef_map.add(new_edge1.clone().into_shape(), list_one_face);
                            for k in 1..j - 1 {
                                at_mut(&mut edge_seq, k)?.nullify();
                            }
                            for k in 1..=j - 1 {
                                let a_last_edge = topods::edge(at(at(&self.my_paths, k)?, i)?)?;
                                let vertex_as_edge = topexp::first_vertex(&a_last_edge, true);
                                insert_before(at_mut(&mut self.my_paths, k)?, i, vertex_as_edge.clone().into_shape())?;
                            }
                            j = 1; // start from beginning
                        } else if choose_edge == 2 {
                            *at_mut(&mut edge_seq, j - 1)? = new_edge2.clone().into_shape();
                            ef_map.add(new_edge2.clone().into_shape(), list_one_face);
                            to_insert_vertex = true;
                        }
                    }
                } else {
                    // connecting edge exists
                    *at_mut(&mut edge_seq, j - 1)? = proper_edge.clone().into_shape();
                }
                j += 1;
            }
            sections_edges.push(edge_seq.clone());

            // check for exit from for(;;)
            let nb_end_edges = edge_seq.iter().filter(|e| end_edges.contains(e)).count();
            if i32::try_from(nb_end_edges).unwrap_or(i32::MAX) == nb_e {
                break;
            }

            i += 1;
        }

        // final phase: building of middle path
        let nb_sec_faces = length(&sections_edges);
        let mut sec_faces: Vec<Shape> = Vec::new();
        for section in &sections_edges {
            let mut mw = BRepLibMakeWire::new();
            for j in 1..=nb_e {
                mw.add_edge(&topods::edge(at(section, j)?)?)?;
            }
            if !self.my_closed_section {
                let (v1, v2) = topexp::wire_vertices(&mw.wire()?);
                let an_edge = BRepLibMakeEdge::from_vertices(&v2, &v1)?.edge()?;
                mw.add_edge(&an_edge)?;
            }
            let a_wire = mw.wire()?;
            let mf = BRepLibMakeFace::from_wire(&a_wire, true)?; // Only plane
            if mf.is_done() {
                sec_faces.push(mf.face()?.clone().into_shape());
            } else {
                sec_faces.push(a_wire.clone().into_shape());
            }
        }

        let mut centers: Vec<Point3> = Vec::new();
        for sec_face in &sec_faces {
            let mut properties = GProps::new();
            if sec_face.shape_type()? == ShapeEnum::Face {
                BRepGProp::surface_properties(sec_face, &mut properties, false, false)?;
            } else {
                // wire
                BRepGProp::linear_properties(sec_face, &mut properties, false, false)?;
            }
            centers.push(properties.centre_of_mass());
        }

        let mut mid_edges: Vec<Shape> = vec![Shape::default(); usize::try_from(nb_sec_faces - 1).unwrap_or(0)];
        let lin_tol = 1.0e-5;
        let ang_tol = 1.0e-7;
        for i in 1..nb_sec_faces {
            let mut type_of_mid_edge = CurveType::OtherCurve;
            for j in 1..=length(&self.my_paths) {
                let a_shape = at(at(&self.my_paths, j)?, i)?;
                if a_shape.shape_type()? == ShapeEnum::Vertex {
                    type_of_mid_edge = CurveType::OtherCurve;
                    break;
                }
                let a_type = type_of_edge(&topods::edge(a_shape)?)?;
                if j == 1 {
                    type_of_mid_edge = a_type;
                } else if a_type != type_of_mid_edge {
                    type_of_mid_edge = CurveType::OtherCurve;
                    break;
                }
            }
            if type_of_mid_edge == CurveType::Line {
                *at_mut(&mut mid_edges, i)? =
                    BRepLibMakeEdge::from_points(*at(&centers, i)?, *at(&centers, i + 1)?)?.edge()?.clone().into_shape();
            } else if type_of_mid_edge == CurveType::Circle {
                *at_mut(&mut mid_edges, i)? = self.mid_arc(i, &centers, lin_tol, ang_tol)?;
            }
        }

        // Build missed edges
        let mut i: i32 = 1;
        while i < nb_sec_faces {
            if at(&mid_edges, i)?.is_null() {
                let mut j = i + 1;
                while j < nb_sec_faces {
                    if !at(&mid_edges, j)?.is_null() {
                        break;
                    }
                    j += 1;
                }
                // from i to j-1 all edges are null
                let mut the_points: Array1<'static, Point3> = Array1::with_bounds(1, j - i + 1).map_err(array_failure)?;
                let mut the_tangents: Array1<'static, Vector3> = Array1::with_bounds(1, j - i + 1).map_err(array_failure)?;
                let mut the_flags: Array1<'static, bool> = Array1::with_bounds(1, j - i + 1).map_err(array_failure)?;
                for k in i..=j {
                    the_points[k - i + 1] = *at(&centers, k)?;
                }
                for k in i..=j {
                    let mut pnt_seq: Vec<Point3> = Vec::new();
                    for indp in 1..=length(&self.my_paths) {
                        let path = at(&self.my_paths, indp)?;
                        let mut a_tangent = if k == i {
                            if at(path, k)?.shape_type()? == ShapeEnum::Vertex {
                                continue;
                            }
                            tangent_of_edge(at(path, k)?, true)? // at begin
                        } else if k == j {
                            if at(path, k - 1)?.shape_type()? == ShapeEnum::Vertex {
                                continue;
                            }
                            tangent_of_edge(at(path, k - 1)?, false)? // at end
                        } else {
                            if at(path, k - 1)?.shape_type()? == ShapeEnum::Vertex
                                || at(path, k)?.shape_type()? == ShapeEnum::Vertex
                            {
                                continue;
                            }
                            let tangent1 = tangent_of_edge(at(path, k - 1)?, false)?; // at end
                            let tangent2 = tangent_of_edge(at(path, k)?, true)?; // at begin
                            tangent1 + tangent2
                        };
                        a_tangent.normalize()?;
                        pnt_seq.push(Point3::from(*a_tangent.xyz()));
                    }
                    let mut pnt_array: Array1<'static, Point3> = Array1::with_bounds(1, length(&pnt_seq)).map_err(array_failure)?;
                    for (ip, p) in (1..).zip(pnt_seq.iter()) {
                        pnt_array[ip] = *p;
                    }
                    let inertia = geom_lib::inertia(&pnt_array)?;
                    the_tangents[k - i + 1] = Vector3::from(inertia.bary.xyz());
                }
                for f in 1..=j - i + 1 {
                    the_flags[f] = true;
                }

                let mut interpol = Interpolate::new(&Handle::new(HArray1::from_owned(the_points)), false, lin_tol)?;
                interpol.load(&the_tangents, &Handle::new(HArray1::from_owned(the_flags)), true)?;
                interpol.perform()?;
                // The native code reports "Interpolation failed" on the standard output and
                // then raises StdFail_NotDone from Curve().
                let inter_curve: Handle<dyn GeomCurve> = interpol.curve()?.cast();
                *at_mut(&mut mid_edges, i)? = BRepLibMakeEdge::from_curve(&inter_curve)?.edge()?.clone().into_shape();
                i = j;
            }
            i += 1;
        }

        let mut make_final_wire = BRepLibMakeWire::new();
        for mid_edge in &mid_edges {
            if !mid_edge.is_null() {
                make_final_wire.add_edge(&topods::edge(mid_edge)?)?;
            }
        }

        self.base.my_shape = make_final_wire.wire()?.clone().into_shape();
        self.base.set_done();
        Ok(())
    }

    /// The circular middle edge `i` when the path edges are similar arcs (coaxial, with
    /// the same end directions): the arc of the same angle through the two centres; a null
    /// shape otherwise (the edge is then interpolated).
    fn mid_arc(
        &self,
        i: i32,
        centers: &[Point3],
        lin_tol: f64,
        ang_tol: f64,
    ) -> Result<Shape, Failure> {
        let mut the_axis = Axis1::default();
        let mut the_dir1 = Direction3::default();
        let mut the_dir2 = Direction3::default();
        let mut the_angle = 0.;
        let mut the_tangent = Vector3::default();
        let mut similar_arcs = true;
        for j in 1..=length(&self.my_paths) {
            let an_edge = topods::edge(at(at(&self.my_paths, j)?, i)?)?;
            let (a_curve, fpar, lpar) = untrimmed_curve(&an_edge)?;
            let curve = live(&a_curve, "curve")?;
            let pnt1 = curve.value(fpar)?;
            let pnt2 = curve.value(lpar)?;
            let a_circle = a_curve.clone().cast::<GeomCircle>();
            let a_circ = live(&a_circle, "circle")?.circ()?;
            if j == 1 {
                the_axis = a_circ.axis();
                the_dir1 = Direction3::from_vector(Vector3::between(a_circ.location(), pnt1))?;
                the_dir2 = Direction3::from_vector(Vector3::between(a_circ.location(), pnt2))?;
                the_angle = lpar - fpar;
                let the_param = if an_edge.orientation() == Orientation::Forward { fpar } else { lpar };
                the_tangent = curve.d1(the_param)?.1;
                if an_edge.orientation() == Orientation::Reversed {
                    the_tangent.reverse();
                }
            } else {
                let an_axis = a_circ.axis();
                let a_lin = rustycad_tkmath::gp::Line3::from(an_axis);
                if !a_lin.contains(the_axis.location(), lin_tol) || !an_axis.is_parallel(the_axis, ang_tol) {
                    similar_arcs = false;
                    break;
                }
                let a_dir1 = Direction3::from_vector(Vector3::between(a_circ.location(), pnt1))?;
                let a_dir2 = Direction3::from_vector(Vector3::between(a_circ.location(), pnt2))?;
                if !((a_dir1.is_equal(&the_dir1, ang_tol) && a_dir2.is_equal(&the_dir2, ang_tol))
                    || (a_dir1.is_equal(&the_dir2, ang_tol) && a_dir2.is_equal(&the_dir1, ang_tol)))
                {
                    similar_arcs = false;
                    break;
                }
            }
        }
        if !similar_arcs {
            return Ok(Shape::default());
        }
        let axis_loc = the_axis.location().xyz();
        let axis_dir = the_axis.direction().xyz();
        let parameter = (at(centers, i)?.xyz() - axis_loc).dot(axis_dir);
        let the_center_of_circ = Point3::from(axis_loc + axis_dir * parameter);

        let vec1 = Vector3::between(the_center_of_circ, *at(centers, i)?);
        let vec2 = Vector3::between(the_center_of_circ, *at(centers, i + 1)?);

        let mut an_angle = vec1.angle_with_ref(vec2, Vector3::from(the_axis.direction()))?;
        if an_angle < 0. {
            an_angle += 2. * PI;
        }
        if (an_angle - the_angle).abs() > ang_tol {
            the_axis.reverse();
        }
        let x_dir = Direction3::from_vector(vec1)?;
        let mut the_ax2 = Axis2::with_x_direction(the_center_of_circ, the_axis.direction(), x_dir)?;
        let mut the_circle = MakeCircle::new(the_ax2, vec1.magnitude())?.value()?;
        let (_, a_tangent) = live(&the_circle, "circle")?.d1(0.)?;
        if a_tangent.dot(the_tangent) < 0. {
            the_axis.reverse();
            the_ax2 = Axis2::with_x_direction(the_center_of_circ, the_axis.direction(), x_dir)?;
            the_circle = MakeCircle::new(the_ax2, vec1.magnitude())?.value()?;
        }
        let circle: Handle<dyn GeomCurve> = the_circle.cast();
        let a_me = BRepLibMakeEdge::from_curve_params(&circle, 0., the_angle)?;
        Ok(if a_me.is_done() {
            a_me.edge()?.clone().into_shape()
        } else {
            Shape::default()
        })
    }
}
