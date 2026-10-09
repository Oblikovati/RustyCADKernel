// Rust translation of the static MakeEdgeDegenerated and InsertEDegenerated
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_AdvancedEvolved.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The degenerated edges plugging the singular vertices of a face rebuilt from split
//! edges: between consecutive edges whose ends differ in UV at the same vertex.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Direction2, Point2, Xy};

/// One degenerated edge on the face along the line through `start` in `direction`.
fn degenerated_edge(
    the_face: &Face,
    (start, direction): (Point2, Direction2),
    (a_vf, a_vl): (&Vertex, &Vertex),
    a_l_par: f64,
) -> Result<Edge, Failure> {
    let a_bb = BRepBuilder::default();
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(start, direction)).cast();
    let mut an_edegen = Edge::default();
    a_bb.make_edge(&mut an_edegen)?;
    a_bb.update_edge_pcurve_on_face(&an_edegen, &line, the_face, precision::confusion())?;
    an_edegen.set_orientation(Orientation::Forward);
    a_bb.add(&mut an_edegen, a_vf)?;
    a_bb.add(&mut an_edegen, a_vl)?;
    a_bb.degenerated(&an_edegen, true)?;
    a_bb.range(&an_edegen, 0.0, a_l_par, false)?;
    Ok(an_edegen)
}

/// `MakeEdgeDegenerated(theV, theFace, thePf, thePl, theLEdges)`: two degenerated
/// edges (both senses) at the vertex between the UV points, appended; false when the
/// points coincide within twice the vertex tolerance in UV.
pub(super) fn make_edge_degenerated(
    the_v: &Vertex,
    the_face: &Face,
    (the_pf, the_pl): (Point2, Point2),
    the_l_edges: &mut List<Shape>,
) -> Result<bool, Failure> {
    let an_as = BRepAdaptorSurface::with_face(the_face, false)?;
    let a_tol: f64 = 2.0 * BRepTool::vertex_tolerance(the_v)?;
    let (a_tol_u, a_tol_v) = (an_as.u_resolution(a_tol)?, an_as.v_resolution(a_tol)?);
    if (the_pf.x() - the_pl.x()).abs() < a_tol_u && (the_pf.y() - the_pl.y()).abs() < a_tol_v {
        return Ok(false);
    }
    let a_vf: Vertex = topods::vertex(&the_v.oriented(Orientation::Forward))?;
    let a_vl: Vertex = topods::vertex(&the_v.oriented(Orientation::Reversed))?;
    let a_v: Xy = the_pl.xy() - the_pf.xy();
    let a_l_par: f64 = a_v.modulus();
    let e1 = degenerated_edge(
        the_face,
        (the_pf, Direction2::from_xy(a_v)?),
        (&a_vf, &a_vl),
        a_l_par,
    )?;
    let e2 = degenerated_edge(
        the_face,
        (the_pl, Direction2::from_xy(a_v.reversed())?),
        (&a_vf, &a_vl),
        a_l_par,
    )?;
    the_l_edges.append(e1.into_shape());
    the_l_edges.append(e2.into_shape());
    Ok(true)
}

/// The UV point of a vertex on an edge of the face.
fn uv_at(edge: &Edge, vertex: &Vertex, face: &Face) -> Result<Point2, Failure> {
    let pc = BRepTool::curve_on_face(edge, face)?;
    let a_f: f64 = BRepTool::parameter(vertex, edge)?;
    pc.pcurve.get().ok_or_else(no_pcurve)?.value(a_f)
}

/// The vertices of the degenerated edges of the wire.
fn degenerated_vertices(a_wir: &Wire) -> Result<MapOfShape, Failure> {
    let mut a_map_vof_de = MapOfShape::default();
    let mut an_exp_de = Explorer::new(a_wir.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while an_exp_de.more() {
        let an_e = topods::edge(an_exp_de.current())?;
        if BRepTool::degenerated(&an_e)? {
            // Both vertices of a degenerated edge are the same.
            a_map_vof_de.add(topexp::vertices(&an_e, false).0.into_shape());
        }
        an_exp_de.next()?;
    }
    Ok(a_map_vof_de)
}

/// The context of the insertion: the face, the vertex/edges map of the wire and the
/// vertices already plugged.
struct Insertion<'a> {
    face: &'a Face,
    map_ve: IndexedDataMapOfShapeListOfShape,
    map_vof_de: MapOfShape,
}

impl Insertion<'_> {
    /// The degenerated edge at the vertex between two consecutive edges (or, at the
    /// apex of a cone, between one of them and a third edge of the vertex).
    fn between(
        &mut self,
        an_e1: &Edge,
        an_e2: &Edge,
        a_vert_curr: &Vertex,
        the_l_edges: &mut List<Shape>,
    ) -> Result<(), Failure> {
        if self.map_vof_de.contains(a_vert_curr.as_ref()) {
            // The necessary degenerated edge has already been created.
            return Ok(());
        }
        let a_pf: Point2 = uv_at(an_e1, a_vert_curr, self.face)?;
        let a_pl: Point2 = uv_at(an_e2, a_vert_curr, self.face)?;
        if make_edge_degenerated(a_vert_curr, self.face, (a_pf, a_pl), the_l_edges)? {
            self.map_vof_de.add(a_vert_curr.as_ref().clone());
            return Ok(());
        }
        let an_e_list: Vec<Shape> = match self.map_ve.seek(a_vert_curr.as_ref()) {
            Some(list) if list.len() <= 2 => return Ok(()),
            Some(list) => list.iter().cloned().collect(),
            // A null list natively (a crash).
            None => return Err(no_pcurve()),
        };
        self.apex(
            &an_e_list,
            (an_e1, an_e2),
            a_vert_curr,
            (a_pf, a_pl),
            the_l_edges,
        )
    }

    /// The case of a cone apex: fine in 2D, but several edges share the vertex in 3D;
    /// the apex is plugged by a degenerated edge from one of the edges to a third one.
    fn apex(
        &mut self,
        an_e_list: &[Shape],
        (an_e1, an_e2): (&Edge, &Edge),
        a_vert_curr: &Vertex,
        (a_pf, a_pl): (Point2, Point2),
        the_l_edges: &mut List<Shape>,
    ) -> Result<(), Failure> {
        for an_edge in an_e_list {
            if BRepTool::degenerated(&topods::edge(an_edge)?)? {
                return Ok(());
            }
        }
        // Look for the pair of anE1 and anE2.
        for a_point in [a_pf, a_pl] {
            for an_edge in an_e_list {
                if an_edge.is_same(an_e1) || an_edge.is_same(an_e2) {
                    continue;
                }
                let a_p: Point2 = uv_at(&topods::edge(an_edge)?, a_vert_curr, self.face)?;
                if make_edge_degenerated(a_vert_curr, self.face, (a_point, a_p), the_l_edges)? {
                    self.map_vof_de.add(a_vert_curr.as_ref().clone());
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// The degenerated edge between the last and the first edges at their common
    /// vertex (a vertex of more than two edges is not joined).
    fn close(
        &mut self,
        a_first_edge: &Edge,
        a_last_edge: &Edge,
        the_l_edges: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let (v0, v1) = topexp::vertices(a_first_edge, false);
        if !v0.is_null() && v0.is_same(&v1) {
            // A not closed edge covered by its single vertex.
            return Ok(());
        }
        let (v2, v3) = topexp::vertices(a_last_edge, false);
        if !v2.is_null() && v2.is_same(&v3) {
            return Ok(());
        }
        for a_vf in [&v0, &v1] {
            for a_vl in [&v2, &v3] {
                if !a_vf.is_same(a_vl) {
                    continue;
                }
                if self.map_ve.seek(a_vf.as_ref()).is_some_and(|l| l.len() > 2) {
                    // Non-manifold topology, or the two ends of a seam that must not
                    // be joined.
                    continue;
                }
                let a_pf: Point2 = uv_at(a_first_edge, a_vf, self.face)?;
                let a_pl: Point2 = uv_at(a_last_edge, a_vl, self.face)?;
                make_edge_degenerated(a_vf, self.face, (a_pf, a_pl), the_l_edges)?;
            }
        }
        Ok(())
    }
}

/// `InsertEDegenerated(theFace, theLEdges)`: the degenerated edges plugging the
/// singular vertices met along the wire of the edges, appended.
pub(super) fn insert_e_degenerated(
    the_face: &Face,
    the_l_edges: &mut List<Shape>,
) -> Result<(), Failure> {
    let a_bb = BRepBuilder::default();
    let mut a_wir = Wire::default();
    a_bb.make_wire(&mut a_wir);
    for an_e in the_l_edges.iter() {
        topods::edge(an_e)?;
        a_bb.add(&mut a_wir, an_e)?;
    }
    let mut map_ve = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_unique_ancestors(
        a_wir.as_ref(),
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        &mut map_ve,
        false,
    )?;
    let mut an_exp = WireExplorer::with_wire_on_face(&a_wir, the_face)?;
    let mut an_e1: Edge = an_exp.current().clone();
    if an_e1.is_null() {
        // Only INTERNAL/EXTERNAL edges in the wire.
        return Ok(());
    }
    let a_first_edge: Edge = an_e1.clone();
    let mut a_last_edge = Edge::default();
    an_exp.next()?;
    let map_vof_de = degenerated_vertices(&a_wir)?;
    let mut insertion = Insertion {
        face: the_face,
        map_ve,
        map_vof_de,
    };
    while an_exp.more() {
        let an_e2: Edge = an_exp.current().clone();
        a_last_edge = an_e2.clone();
        let a_vert_curr: Vertex = an_exp.current_vertex().clone();
        insertion.between(&an_e1, &an_e2, &a_vert_curr, the_l_edges)?;
        an_e1 = an_e2;
        an_exp.next()?;
    }
    if a_last_edge.is_null() {
        return Ok(());
    }
    insertion.close(&a_first_edge, &a_last_edge, the_l_edges)
}

/// Standard_NullObject of an edge without p-curve on the face or a vertex without
/// edges (natively a crash).
fn no_pcurve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_AdvancedEvolved: edge without p-curve or vertex without edge; expected one",
        "BRepFill_AdvancedEvolved",
    )
}
