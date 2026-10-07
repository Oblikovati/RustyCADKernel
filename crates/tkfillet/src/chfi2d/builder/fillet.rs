// Rust translation of ChFi2d_Builder::AddFillet, ModifyFillet, RemoveFillet and
// ComputeFillet (ChFi2d_Builder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillets of the builder: added at a vertex, modified, removed.
use super::{ChFi2dBuilder, is_line_or_circle};
use crate::chfi2d::{ConstructionError, find_connected_edges};
use rustycad_tkbrep::topods::{self, Edge, Vertex};
use rustycad_tkernel::standard::Failure;

/// Whether a status lets the contour be edited (done, possibly with degenerated edges).
pub(super) fn editable(status: ConstructionError) -> bool {
    matches!(
        status,
        ConstructionError::IsDone
            | ConstructionError::FirstEdgeDegenerated
            | ConstructionError::LastEdgeDegenerated
            | ConstructionError::BothEdgesDegenerated
    )
}

/// The trimmed edges and the new edge (fillet or chamfer) of a computation.
#[derive(Default)]
pub(super) struct Computed {
    pub(super) trim1: Edge,
    pub(super) trim2: Edge,
    pub(super) edge: Edge,
}

impl ChFi2dBuilder {
    /// `AddFillet(V, Radius)`: the fillet of radius `radius` at the vertex between its
    /// two edges (lines or circles, not fillets nor chamfers); the null edge on failure
    /// (see the status). `builder.add_fillet(&vertex, 1.0)?`.
    pub fn add_fillet(&mut self, v: &Vertex, radius: f64) -> Result<Edge, Failure> {
        let (mut adj1, mut adj2) = (Edge::default(), Edge::default());
        self.status = find_connected_edges(&self.new_face, v, &mut adj1, &mut adj2)?;
        if self.status == ConstructionError::ConnexionError {
            return Ok(Edge::default());
        }
        if !self.authorized(&adj1, &adj2)? {
            self.status = ConstructionError::NotAuthorized;
            return Ok(Edge::default());
        }
        let computed = self.compute_fillet(v, (&adj1, &adj2), radius)?;
        self.apply(&adj1, &adj2, &computed, 1)
    }

    /// Neither edge a fillet or a chamfer, both lines or circles.
    pub(super) fn authorized(&self, e1: &Edge, e2: &Edge) -> Result<bool, Failure> {
        if self.any_fillet_or_chamfer(e1, e2) {
            return Ok(false);
        }
        Ok(is_line_or_circle(e1, &self.new_face)? && is_line_or_circle(e2, &self.new_face)?)
    }

    /// The edit once computed: the new contour, the history, the new fillet (`id` 1) or
    /// chamfer returned; the computed (null) edge when the status forbids it.
    pub(super) fn apply(
        &mut self,
        adj1: &Edge,
        adj2: &Edge,
        c: &Computed,
        id: i32,
    ) -> Result<Edge, Failure> {
        if !editable(self.status) {
            return Ok(c.edge.clone());
        }
        self.build_new_wire((adj1, adj2), &c.trim1, &c.edge, &c.trim2)?;
        let (basis1, basis2) = (self.basis_edge(adj1)?, self.basis_edge(adj2)?);
        self.update_history_new((&basis1, &basis2), (&c.trim1, &c.trim2), &c.edge, id);
        self.status = ConstructionError::IsDone;
        let list = if id == 1 {
            &self.fillets
        } else {
            &self.chamfers
        };
        list.last()
            .map_or_else(|| Ok(Edge::default()), topods::edge)
    }

    /// `ModifyFillet(Fillet, Radius)`: the fillet removed then added again with the new
    /// radius.
    pub fn modify_fillet(&mut self, fillet: &Edge, radius: f64) -> Result<Edge, Failure> {
        let vertex = self.remove_fillet(fillet)?;
        self.add_fillet(&vertex, radius)
    }

    /// `RemoveFillet(Fillet)`: the contour restored at the vertex of the fillet, which is
    /// returned (null when the edge is not a fillet of the builder).
    pub fn remove_fillet(&mut self, fillet: &Edge) -> Result<Vertex, Failure> {
        let Some(i) = self.fillets.iter().position(|f| f.is_same(fillet)) else {
            return Ok(Vertex::default());
        };
        self.fillets.remove(i);
        self.remove_added(fillet, false)
    }

    /// `ComputeFillet(V, E1, E2, Radius, TrimE1, TrimE2, Fillet)`: the fillet edge, and
    /// the edges trimmed to its ends (the status tells which ones degenerate).
    fn compute_fillet(
        &mut self,
        v: &Vertex,
        (e1, e2): (&Edge, &Edge),
        radius: f64,
    ) -> Result<Computed, Failure> {
        let mut c = Computed::default();
        let (edge, extr1, extr2) = self.build_fillet_edge(v, (e1, e2), radius)?;
        c.edge = edge;
        if self.status != ConstructionError::IsDone {
            return Ok(c);
        }
        self.trim_both(v, (e1, e2), (&extr1, &extr2), &mut c)?;
        Ok(c)
    }

    /// The edges trimmed to the new extremities, and the degenerate status.
    pub(super) fn trim_both(
        &mut self,
        v: &Vertex,
        (e1, e2): (&Edge, &Edge),
        (x1, x2): (&Vertex, &Vertex),
        c: &mut Computed,
    ) -> Result<(), Failure> {
        let (trim1, degen1) = self.build_new_edge(e1, v, x1)?;
        let (trim2, degen2) = self.build_new_edge(e2, v, x2)?;
        (c.trim1, c.trim2) = (trim1, trim2);
        self.status = match (degen1, degen2) {
            (true, true) => ConstructionError::BothEdgesDegenerated,
            (true, false) => ConstructionError::FirstEdgeDegenerated,
            (false, true) => ConstructionError::LastEdgeDegenerated,
            (false, false) => self.status,
        };
        Ok(())
    }
}
