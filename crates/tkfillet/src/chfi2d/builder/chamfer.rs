// Rust translation of ChFi2d_Builder::AddChamfer (both), ModifyChamfer (both),
// RemoveChamfer and ComputeChamfer (both) (ChFi2d_Builder_0.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chamfers of the builder: by two distances or by a distance and an angle.
use super::ChFi2dBuilder;
use super::fillet::Computed;
use crate::chfi2d::{ConstructionError, common_vertex, find_connected_edges};
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::Failure;

/// The size of a chamfer: two distances, or a distance on the first edge and an angle.
#[derive(Clone, Copy, Debug)]
pub(super) enum ChamferSize {
    Distances(f64, f64),
    DistanceAngle(f64, f64),
}

impl ChFi2dBuilder {
    /// `AddChamfer(E1, E2, D1, D2)`: the chamfer at the common vertex of the edges, at
    /// the distances from it; the null edge on failure (see the status).
    pub fn add_chamfer(&mut self, e1: &Edge, e2: &Edge, d1: f64, d2: f64) -> Result<Edge, Failure> {
        let Some(common) = common_vertex(e1, e2) else {
            return Ok(Edge::default());
        };
        if !self.authorized(e1, e2)? {
            self.status = ConstructionError::NotAuthorized;
            return Ok(Edge::default());
        }
        // EE1 and EE2 are copies of E1 and E2 with the good orientation on <newFace>
        let (mut ee1, mut ee2) = (Edge::default(), Edge::default());
        self.status = find_connected_edges(&self.new_face, &common, &mut ee1, &mut ee2)?;
        if ee1.is_same(e2) {
            let orient = ee1.orientation();
            ee1 = ee2;
            ee2 = e2.clone();
            ee2.set_orientation(orient);
        }
        let computed =
            self.compute_chamfer(&common, (&ee1, &ee2), ChamferSize::Distances(d1, d2))?;
        self.apply(&ee1, &ee2, &computed, 2)
    }

    /// `AddChamfer(E, V, D, Ang)`: the chamfer at the vertex, at the distance on the edge
    /// `e` and at the angle from it.
    pub fn add_chamfer_angle(
        &mut self,
        e: &Edge,
        v: &Vertex,
        d: f64,
        ang: f64,
    ) -> Result<Edge, Failure> {
        let (mut adj1, mut adj2) = (Edge::default(), Edge::default());
        self.status = find_connected_edges(&self.new_face, v, &mut adj1, &mut adj2)?;
        if self.status == ConstructionError::ConnexionError {
            return Ok(Edge::default());
        }
        // adjEdge1 is a copy of E with the good orientation on <newFace>
        if adj2.is_same(e) {
            let orient = adj2.orientation();
            adj2 = adj1;
            adj1 = e.clone();
            adj1.set_orientation(orient);
        }
        if !self.authorized(&adj1, &adj2)? {
            self.status = ConstructionError::NotAuthorized;
            return Ok(Edge::default());
        }
        let computed =
            self.compute_chamfer(v, (&adj1, &adj2), ChamferSize::DistanceAngle(d, ang))?;
        self.apply(&adj1, &adj2, &computed, 2)
    }

    /// `ComputeChamfer(V, E1, E2, D1, D2, ...)` and `ComputeChamfer(V, E1, D, Ang, E2,
    /// ...)`: the chamfer edge, and the edges trimmed to its ends.
    fn compute_chamfer(
        &mut self,
        v: &Vertex,
        (e1, e2): (&Edge, &Edge),
        size: ChamferSize,
    ) -> Result<Computed, Failure> {
        let mut c = Computed::default();
        let (edge, extr1, extr2) = match size {
            ChamferSize::Distances(d1, d2) => self.build_chamfer_edge(v, (e1, e2), (d1, d2))?,
            ChamferSize::DistanceAngle(d, ang) => {
                self.build_chamfer_edge_angle(v, (e1, e2), (d, ang))?
            }
        };
        c.edge = edge;
        if self.status != ConstructionError::IsDone {
            return Ok(c);
        }
        self.trim_both(v, (e1, e2), (&extr1, &extr2), &mut c)?;
        Ok(c)
    }

    /// `ModifyChamfer(Chamfer, E1, E2, D1, D2)`: the chamfer removed then added again at
    /// the new distances (the first edge is not used, as natively).
    pub fn modify_chamfer(
        &mut self,
        chamfer: &Edge,
        _e1: &Edge,
        e2: &Edge,
        d1: f64,
        d2: f64,
    ) -> Result<Edge, Failure> {
        let vertex = self.remove_chamfer(chamfer)?;
        let (mut adj1, mut adj2) = (Edge::default(), Edge::default());
        self.status = find_connected_edges(&self.new_face, &vertex, &mut adj1, &mut adj2)?;
        if self.status == ConstructionError::ConnexionError {
            return Ok(Edge::default());
        }
        // adjEdge1 and adjEdge2 are copies of E1 and E2 with the good orientation
        if adj1.is_same(e2) {
            let orient = adj1.orientation();
            adj1 = adj2;
            adj2 = e2.clone();
            adj2.set_orientation(orient);
        }
        self.add_chamfer(&adj1, &adj2, d1, d2)
    }

    /// `ModifyChamfer(Chamfer, E, D, Ang)`: the chamfer removed then added again at the
    /// new distance and angle.
    pub fn modify_chamfer_angle(
        &mut self,
        chamfer: &Edge,
        e: &Edge,
        d: f64,
        ang: f64,
    ) -> Result<Edge, Failure> {
        let vertex = self.remove_chamfer(chamfer)?;
        let (mut adj1, mut adj2) = (Edge::default(), Edge::default());
        self.status = find_connected_edges(&self.new_face, &vertex, &mut adj1, &mut adj2)?;
        if self.status == ConstructionError::ConnexionError {
            return Ok(Edge::default());
        }
        let on = if adj1.is_same(e) { adj1 } else { adj2 };
        self.add_chamfer_angle(&on, &vertex, d, ang)
    }

    /// `RemoveChamfer(Chamfer)`: the contour restored at the vertex of the chamfer, which
    /// is returned (null when the edge is not a chamfer of the builder).
    pub fn remove_chamfer(&mut self, chamfer: &Edge) -> Result<Vertex, Failure> {
        let Some(i) = self.chamfers.iter().position(|c| c.is_same(chamfer)) else {
            return Ok(Vertex::default());
        };
        self.chamfers.remove(i);
        self.remove_added(chamfer, true)
    }
}
