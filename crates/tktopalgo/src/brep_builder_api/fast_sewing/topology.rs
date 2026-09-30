// Rust translation of the protected nested classes of BRepBuilderAPI_FastSewing
// (FS_Vertex, FS_Face, FS_Edge, NodeInspector), of FindVertexes, FindEdges,
// UpdateEdgeInfo and CreateNewEdge, and of the static IntersectionOfSets and Get2DCurve
// (BRepBuilderAPI_FastSewing.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertices, edges and faces of the fast sewing: shared corner points, shared
//! sides, and the topological shapes built from them.
use super::{BRepBuilderAPIFastSewing, FastSewingStatus};
use crate::brep_builder_api::sewing::failures::{null_object, out_of_range, program_error};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex, Wire};
use rustycad_tkernel::ncollection::{CellAction, CellFilter, CellInspector};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Point3, Xyz};

/// `FS_Vertex`: a corner point shared by faces and edges (its native `myID` is its
/// index in the vector, never read).
#[derive(Clone, Default)]
pub(super) struct FsVertex {
    pub my_pnt: Point3,
    pub my_topo_vert: Vertex,
    pub my_faces: Vec<i32>,
    pub my_edges: Vec<i32>,
}

impl FsVertex {
    /// `CreateTopologicalVertex(theToler)`.
    pub fn create_topological_vertex(&mut self, toler: f64) -> Result<(), Failure> {
        BRepBuilder::default().make_vertex_at(&mut self.my_topo_vert, self.my_pnt, toler)
    }
}

/// `FS_Face`: a source face, its four edges (V = Vf, U = Ul, V = Vl, U = Uf) and the
/// vertices that start them (its native `myID` is its index in the vector, never
/// read).
pub(super) struct FsFace {
    pub my_src_face: Face,
    pub my_wire: Wire,
    pub my_ret_face: Face,
    pub my_edges: [i32; 4],
    pub my_vertices: [i32; 4],
}

/// `FS_Edge`: a side shared by faces, between two vertices.
#[derive(Clone)]
pub(super) struct FsEdge {
    pub my_faces: Vec<i32>,
    pub my_id: i32,
    pub my_topo_edge: Edge,
    my_vertices: [i32; 2],
}

/// `NodeInspector`: the nearest vertex within the tolerance of a point. The points of
/// the vertices are lent to it for one inspection (natively a reference to the vector).
pub(super) struct NodeInspector {
    points: Vec<Point3>,
    my_point: Point3,
    my_sq_toler: f64,
    my_res_id: i32,
}

impl CellInspector for NodeInspector {
    type Target = i32;
    type Point = Xyz;
    const DIMENSION: usize = 3;
    fn coord(axis: usize, point: &Xyz) -> f64 {
        match axis {
            0 => point.x(),
            1 => point.y(),
            _ => point.z(),
        }
    }
    fn is_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
    /// `Inspect(theID)`: keeps the nearest vertex so far.
    fn inspect(&mut self, target: &i32) -> CellAction {
        if let Some(point) = usize::try_from(*target)
            .ok()
            .and_then(|i| self.points.get(i))
        {
            let sq_dist = point.square_distance(&self.my_point);
            if sq_dist < self.my_sq_toler {
                self.my_sq_toler = sq_dist;
                self.my_res_id = *target;
            }
        }
        CellAction::Keep
    }
}

/// The static `IntersectionOfSets(theSet1, theSet2)`: the smallest common value, -1
/// when none.
fn intersection_of_sets(set1: &[i32], set2: &[i32]) -> i32 {
    let mut ret_val = i32::MAX;
    for &val1 in set1 {
        if set2.contains(&val1) && val1 < ret_val {
            ret_val = val1;
        }
    }
    if ret_val == i32::MAX { -1 } else { ret_val }
}

/// The static `Get2DCurve(theIndex, Uf, Ul, Vf, Vl, theIsReverse)`: the p-curve of a side
/// of the parametric rectangle, reversed on request; Standard_OutOfRange outside
/// `[0, 3]`.
fn get_2d_curve(
    index: usize,
    (uf, ul, vf, vl): (f64, f64, f64, f64),
    reverse: bool,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (origin, direction, (u1, u2)) = match (index, reverse) {
        (0, false) => (Point2::new(0.0, vf), Direction2::X, (uf, ul)),
        (1, false) => (Point2::new(ul, 0.0), Direction2::Y, (vf, vl)),
        (2, false) => (Point2::new(0.0, vl), Direction2::X, (uf, ul)),
        (3, false) => (Point2::new(uf, 0.0), Direction2::Y, (vf, vl)),
        (0, true) => (Point2::new(uf + ul, vf), Direction2::NEG_X, (uf, ul)),
        (1, true) => (Point2::new(ul, vf + vl), Direction2::NEG_Y, (vf, vl)),
        (2, true) => (Point2::new(uf + ul, vl), Direction2::NEG_X, (uf, ul)),
        (3, true) => (Point2::new(uf, vf + vl), Direction2::NEG_Y, (vf, vl)),
        _ => {
            return Err(out_of_range(
                "BRepBuilderAPI_FastSewing.cxx, Get2DCurve()",
                index as i32,
                3,
            ));
        }
    };
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_lin2d(Line2::new(origin, direction))).cast();
    Ok(Handle::new(Geom2dTrimmedCurve::new(&line, u1, u2, true, true)?).cast())
}

impl FsFace {
    /// `FS_Face()` with its source face.
    pub fn new(face: Face) -> Self {
        Self {
            my_src_face: face,
            my_wire: Wire::default(),
            my_ret_face: Face::default(),
            my_edges: [-1; 4],
            my_vertices: [-1; 4],
        }
    }

    /// `CreateTopologicalWire(theEdgeVec, theToler)`: the four edges in order, each
    /// with its p-curve on the face's surface (reversed when the 3d curve runs the
    /// other way), a closed wire.
    pub fn create_topological_wire(&mut self, edges: &[FsEdge], toler: f64) -> Result<(), Failure> {
        let (surface, location) = BRepTool::surface_with_location(&self.my_src_face)?;
        let s = surface
            .get()
            .ok_or_else(|| null_object("FS_Face::CreateTopologicalWire: null surface"))?;
        let bounds = s.bounds();
        let b = BRepBuilder::default();
        b.make_wire(&mut self.my_wire);
        for index in 0..4 {
            let id = self.my_edges[index];
            let fs_edge = usize::try_from(id)
                .ok()
                .and_then(|i| edges.get(i))
                .ok_or_else(|| program_error("BRepBuilderAPI_FastSewing::FS_Face::CreateTopologicalWire: Wire is not closed."))?;
            let mut ori = if index < 2 {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            let mut top_e = fs_edge.my_topo_edge.clone();
            if !fs_edge.is_degenerated() {
                let (c3d, first, last) = BRepTool::curve(&top_e)?;
                let c = c3d
                    .get()
                    .ok_or_else(|| null_object("FS_Face::CreateTopologicalWire: null 3d curve"))?;
                let mut c2d = get_2d_curve(index, bounds, false)?;
                let p_ref = c.value(first)?;
                let (p2df, p2dl) = match c2d.get() {
                    Some(c2) => (c2.value(first)?, c2.value(last)?),
                    None => {
                        return Err(null_object("FS_Face::CreateTopologicalWire: null p-curve"));
                    }
                };
                let p3df = s
                    .value(p2df.x(), p2df.y())?
                    .transformed(location.transformation());
                let p3dl = s
                    .value(p2dl.x(), p2dl.y())?
                    .transformed(location.transformation());
                if p3dl.square_distance(&p_ref) < p3df.square_distance(&p_ref) {
                    c2d = get_2d_curve(index, bounds, true)?;
                    ori = top_abs::reverse(ori);
                }
                b.update_edge_pcurve(&top_e, &c2d, &surface, &location, toler, None)?;
            }
            top_e.set_orientation(ori);
            b.add(&mut self.my_wire, &top_e)?;
        }
        self.my_wire.set_closed(true)
    }

    /// `CreateTopologicalFace()`: an empty copy of the source face bounded by the wire,
    /// with a natural restriction.
    pub fn create_topological_face(&mut self) -> Result<(), Failure> {
        if self.my_wire.is_null() {
            return Err(program_error(
                "BRepBuilderAPI_FastSewing::FS_Face::CreateTopologicalFace: Cannot create wire.",
            ));
        }
        let b = BRepBuilder::default();
        self.my_ret_face = Face::unchecked(self.my_src_face.empty_copied()?);
        b.add(&mut self.my_ret_face, &self.my_wire)?;
        b.natural_restriction(&self.my_ret_face, true)
    }
}

impl FsEdge {
    /// `FS_Edge(theIDVert1, theIDVert2)`.
    fn new(vertex1: i32, vertex2: i32) -> Self {
        Self {
            my_faces: Vec::new(),
            my_id: -1,
            my_topo_edge: Edge::default(),
            my_vertices: [vertex1, vertex2],
        }
    }

    /// `IsDegenerated()`: both ends are the same vertex.
    pub fn is_degenerated(&self) -> bool {
        self.my_vertices[0] == self.my_vertices[1]
    }

    /// `CreateTopologicalEdge(theVertexVec, theFaceVec, theTol)`: on the first face of
    /// the edge, a degenerated edge with its p-curve, or an edge on the isoline of its
    /// side (in the surface's own coordinates, as natively).
    pub fn create_topological_edge(
        &self,
        vertices: &[FsVertex],
        faces: &[FsFace],
        tol: f64,
    ) -> Result<Edge, Failure> {
        let vertex = |i: i32| {
            vertices
                .get(i as usize)
                .map(|v| v.my_topo_vert.clone())
                .ok_or_else(|| program_error("FS_Edge: vertex index"))
        };
        let v1 = vertex(self.my_vertices[0])?.oriented(Orientation::Forward);
        let v2 = vertex(self.my_vertices[1])?.oriented(Orientation::Reversed);
        let face = self
            .my_faces
            .first()
            .and_then(|&f| faces.get(f as usize))
            .ok_or_else(|| out_of_range("FS_Edge::CreateTopologicalEdge: myFaces", 1, 0))?;
        let (surface, location) = BRepTool::surface_with_location(&face.my_src_face)?;
        let s = surface
            .get()
            .ok_or_else(|| null_object("FS_Edge::CreateTopologicalEdge: null surface"))?;
        let bounds = s.bounds();
        let edge_id = face
            .my_edges
            .iter()
            .position(|&e| e == self.my_id)
            .ok_or_else(|| {
                program_error(
                    "BRepBuilderAPI_FastSewing::FS_Edge::CreateTopologicalEdge: Single edge.",
                )
            })?;
        let b = BRepBuilder::default();
        let mut edge = Edge::default();
        if self.is_degenerated() {
            let c2d = get_2d_curve(edge_id, bounds, false)?;
            let c2 = c2d
                .get()
                .ok_or_else(|| null_object("FS_Edge: null p-curve"))?;
            let (f_par, l_par) = (c2.first_parameter(), c2.last_parameter());
            b.make_edge(&mut edge)?;
            b.update_edge_pcurve(&edge, &c2d, &surface, &location, tol, None)?;
            b.add(&mut edge, &v1)?;
            b.add(&mut edge, &v2)?;
            b.range(&edge, f_par, l_par, false)?;
            b.degenerated(&edge, true)?;
            return Ok(edge);
        }
        let (uf, ul, vf, vl) = bounds;
        let c3d: Handle<dyn GeomCurve> = match edge_id {
            0 => s.v_iso(vf)?,
            1 => s.u_iso(ul)?,
            2 => s.v_iso(vl)?,
            _ => s.u_iso(uf)?,
        };
        let c = c3d
            .get()
            .ok_or_else(|| null_object("FS_Edge: null isoline"))?;
        b.make_edge_with_curve(&mut edge, &c3d, tol)?;
        b.add(&mut edge, &v1)?;
        b.add(&mut edge, &v2)?;
        b.range(&edge, c.first_parameter(), c.last_parameter(), false)?;
        Ok(edge)
    }
}

impl BRepBuilderAPIFastSewing {
    /// `FindVertexes(theSurfID, theCells)`: each corner of the surface takes the nearest
    /// vertex within the tolerance, or a new vertex filed in the cells; `points` holds
    /// the points of the vertices (the vector the native inspector reads).
    pub(super) fn find_vertexes(
        &mut self,
        surf_id: usize,
        cells: &mut CellFilter<NodeInspector>,
        points: &mut Vec<Point3>,
    ) -> Result<(), Failure> {
        let surface = BRepTool::surface(&self.my_face_vec[surf_id].my_src_face)?;
        let s = surface
            .get()
            .ok_or_else(|| null_object("FindVertexes: null surface"))?;
        let (uf, ul, vf, vl) = s.bounds();
        let corners = [
            s.value(uf, vf)?,
            s.value(ul, vf)?,
            s.value(ul, vl)?,
            s.value(uf, vl)?,
        ];
        for (i, corner) in corners.into_iter().enumerate() {
            let mut inspector = NodeInspector {
                points: std::mem::take(points),
                my_point: corner,
                my_sq_toler: self.my_tolerance * self.my_tolerance,
                my_res_id: -1,
            };
            let mut bounds = BoundingBox3::default();
            bounds.add_point(corner);
            bounds.enlarge(self.my_tolerance);
            let (min, max) = (bounds.corner_min()?.xyz(), bounds.corner_max()?.xyz());
            cells.inspect_range(&min, &max, &mut inspector);
            *points = inspector.points;
            if inspector.my_res_id >= 0 {
                self.my_face_vec[surf_id].my_vertices[i] = inspector.my_res_id;
                self.my_vertex_vec[inspector.my_res_id as usize]
                    .my_faces
                    .push(surf_id as i32);
                continue;
            }
            let id = self.my_vertex_vec.len() as i32;
            self.my_vertex_vec.push(FsVertex {
                my_pnt: corner,
                my_faces: vec![surf_id as i32],
                ..FsVertex::default()
            });
            points.push(corner);
            self.my_face_vec[surf_id].my_vertices[i] = id;
            cells.add_range(&id, &min, &max);
        }
        Ok(())
    }

    /// `FindEdges(theSurfID)`: each side of the face is a new edge, unless an earlier
    /// face shares its two vertices (the edge is then updated).
    pub(super) fn find_edges(&mut self, surf_id: usize) -> Result<(), Failure> {
        const FIRST_IND: [usize; 4] = [0, 1, 3, 0];
        const LAST_IND: [usize; 4] = [1, 2, 2, 3];
        for i in 0..4 {
            let first = self.my_face_vec[surf_id].my_vertices[FIRST_IND[i]];
            let last = self.my_face_vec[surf_id].my_vertices[LAST_IND[i]];
            if first == last {
                // Edge is degenerated.
                self.create_new_edge(first, last, surf_id, i);
                continue;
            }
            let int_res = intersection_of_sets(
                &self.my_vertex_vec[first as usize].my_faces,
                &self.my_vertex_vec[last as usize].my_faces,
            );
            if int_res < 0 || int_res >= surf_id as i32 {
                self.create_new_edge(first, last, surf_id, i);
            } else {
                self.update_edge_info(first, last, surf_id, i)?;
            }
        }
        Ok(())
    }

    /// `UpdateEdgeInfo(theIDPrevVertex, theIDCurrVertex, theFaceID, theIDCurvOnFace)`:
    /// the edge of the two vertices gets the face; Standard_ProgramError when there is
    /// none.
    fn update_edge_info(
        &mut self,
        prev: i32,
        curr: i32,
        face_id: usize,
        curv: usize,
    ) -> Result<(), Failure> {
        let edge_id = intersection_of_sets(
            &self.my_vertex_vec[prev as usize].my_edges,
            &self.my_vertex_vec[curr as usize].my_edges,
        );
        if edge_id < 0 {
            return Err(program_error(
                "BRepBuilderAPI_FastSewing::UpdateEdgeInfo: Update not existing edge.",
            ));
        }
        let edge = &mut self.my_edge_vec[edge_id as usize];
        edge.my_faces.push(face_id as i32);
        self.my_face_vec[face_id].my_edges[curv] = edge.my_id;
        Ok(())
    }

    /// `CreateNewEdge(theIDPrevVertex, theIDCurrVertex, theFaceID, theIDCurvOnFace)`:
    /// a new edge of the face, recorded on its vertices (FS_Degenerated when they are
    /// the same).
    fn create_new_edge(&mut self, prev: i32, curr: i32, face_id: usize, curv: usize) {
        let mut edge = FsEdge::new(prev, curr);
        edge.my_id = self.my_edge_vec.len() as i32;
        edge.my_faces.push(face_id as i32);
        self.my_face_vec[face_id].my_edges[curv] = edge.my_id;
        self.my_vertex_vec[prev as usize].my_edges.push(edge.my_id);
        if prev == curr {
            // The edge is degenerated.
            self.set_status(FastSewingStatus::Degenerated);
        } else {
            self.my_vertex_vec[curr as usize].my_edges.push(edge.my_id);
        }
        self.my_edge_vec.push(edge);
    }
}
