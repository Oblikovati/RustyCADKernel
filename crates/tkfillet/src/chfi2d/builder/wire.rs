// Rust translation of ChFi2d_Builder::BuildNewWire, BuildNewEdge (both), UpDateHistory
// (both) and the shared part of RemoveFillet and RemoveChamfer (ChFi2d_Builder.cxx,
// ChFi2d_Builder_0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edits of the contour: the new wire with a fillet or a chamfer, the trimmed edges,
//! the history, and the contour restored when a fillet or a chamfer is removed.
use super::ChFi2dBuilder;
use crate::chfi2d::{ConstructionError, common_vertex, find_connected_edges};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Builder, Edge, Shape, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_lib::{BRepLibEdgeError, BRepLibMakeEdge, BRepLibMakeFace};

/// The degenerate statuses of the first and the last edge.
fn first_degenerated(status: ConstructionError) -> bool {
    matches!(
        status,
        ConstructionError::FirstEdgeDegenerated | ConstructionError::BothEdgesDegenerated
    )
}

fn last_degenerated(status: ConstructionError) -> bool {
    matches!(
        status,
        ConstructionError::LastEdgeDegenerated | ConstructionError::BothEdgesDegenerated
    )
}

impl ChFi2dBuilder {
    /// `BuildNewWire(OldE1, OldE2, E1, Fillet, E2)`: the contour with the old edges
    /// replaced by the trimmed ones (unless degenerated) and the fillet after the first
    /// one met; the closure of the first wire of the reference face is kept.
    pub(super) fn build_new_wire(
        &mut self,
        old: (&Edge, &Edge),
        e1: &Edge,
        fillet: &Edge,
        e2: &Edge,
    ) -> Result<(), Failure> {
        let mut ex = Explorer::new(&self.ref_face, ShapeEnum::Wire, ShapeEnum::Shape)?;
        let closed = if ex.more() {
            topods::wire(ex.current())?.closed()?
        } else {
            true
        };
        let (mut fillet_added, b) = (false, Builder);
        let mut new_wire = Wire::default();
        b.make_wire(&mut new_wire);
        ex.init(&self.new_face, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while ex.more() {
            let the_edge = topods::edge(ex.current())?;
            if !the_edge.is_same(old.0) && !the_edge.is_same(old.1) {
                b.add(&mut new_wire, &the_edge)?;
            } else {
                let is_first = *the_edge == **old.0;
                let degenerated = if is_first {
                    first_degenerated(self.status)
                } else {
                    last_degenerated(self.status)
                };
                if !degenerated {
                    b.add(&mut new_wire, if is_first { e1 } else { e2 })?;
                }
                if !fillet_added {
                    b.add(&mut new_wire, fillet)?;
                    fillet_added = true;
                }
            }
            ex.next()?;
        }
        new_wire.set_closed(closed)?;
        self.new_face = self.face_on_reference_plane(&new_wire)?;
        Ok(())
    }

    /// The face of the plane of the reference face bounded by the wire.
    fn face_on_reference_plane(
        &self,
        wire: &Wire,
    ) -> Result<rustycad_tkbrep::topods::Face, Failure> {
        let plane = BRepAdaptorSurface::with_face(&self.ref_face, true)?.plane()?;
        BRepLibMakeFace::from_plane_wire(plane, wire, true)?.face()
    }

    /// `BuildNewEdge(E1, OldExtr, NewExtr, IsDegenerated)`: the edge on the curve of `e1`
    /// from its kept vertex to the new extremity; `e1` itself and true when that is a
    /// point.
    pub(super) fn build_new_edge(
        &self,
        e1: &Edge,
        old_extr: &Vertex,
        new_extr: &Vertex,
    ) -> Result<(Edge, bool), Failure> {
        let (first_vertex, last_vertex) = topexp::vertices(e1, false);
        let pnew = BRepTool::pnt(new_extr)?;
        let (curve, _, _) = BRepTool::curve(e1)?;
        let mut make_edge = BRepLibMakeEdge::new();
        let kept = if first_vertex.is_same(old_extr) {
            make_edge.init_curve_vertices(&curve, new_extr, &last_vertex)?;
            last_vertex
        } else {
            make_edge.init_curve_vertices(&curve, &first_vertex, new_extr)?;
            first_vertex
        };
        let ponctual = pnew.distance(&BRepTool::pnt(&kept)?) < precision::confusion();
        if make_edge.error() == BRepLibEdgeError::LineThroughIdenticPoints || ponctual {
            return Ok((e1.clone(), true));
        }
        let mut an_edge = make_edge.edge()?;
        an_edge.set_orientation(e1.orientation());
        Ok((an_edge, false))
    }

    /// `UpDateHistory(E1, E2, TrimE1, TrimE2, NewEdge, Id)`: the new fillet (`Id` 1) or
    /// chamfer, and the basis edges bound to their trimmed edges unless degenerated.
    pub(super) fn update_history_new(
        &mut self,
        basis: (&Edge, &Edge),
        trim: (&Edge, &Edge),
        new_edge: &Edge,
        id: i32,
    ) {
        if id == 1 {
            self.fillets.push(new_edge.clone().into_shape());
        } else {
            self.chamfers.push(new_edge.clone().into_shape());
        }
        self.history.unbind(basis.0);
        if !first_degenerated(self.status) && !basis.0.is_same(trim.0) {
            self.history
                .bind(basis.0.clone().into_shape(), trim.0.clone().into_shape());
        }
        self.history.unbind(basis.1);
        if !last_degenerated(self.status) && !basis.1.is_same(trim.1) {
            self.history
                .bind(basis.1.clone().into_shape(), trim.1.clone().into_shape());
        }
    }

    /// `UpDateHistory(E1, E2, TrimE1, TrimE2)`: the basis edges bound to their new edges.
    fn update_history(&mut self, basis: (&Edge, &Edge), trim: (&Edge, &Edge)) {
        for (b, t) in [(basis.0, trim.0), (basis.1, trim.1)] {
            if self.history.is_bound(b) {
                self.history.unbind(b);
            }
            if !b.is_same(t) {
                self.history
                    .bind(b.clone().into_shape(), t.clone().into_shape());
            }
        }
    }

    /// The shared text of RemoveFillet and RemoveChamfer once the edge is found in its
    /// list: the edges at both ends of `added`, their basis edges and common vertex, the
    /// edges restored to it, and the contour rebuilt. The new edges take the orientation
    /// and location of the adjacent edge (`from_basis` false) or of its basis edge.
    pub(super) fn remove_added(
        &mut self,
        added: &Edge,
        from_basis: bool,
    ) -> Result<Vertex, Failure> {
        let (first_vertex, last_vertex) = topexp::vertices(added, false);
        let Some(e1) = self.other_edge_at(&first_vertex, added)? else {
            return Ok(Vertex::default());
        };
        let basis1 = self.basis_edge(&e1)?;
        let Some(e2) = self.other_edge_at(&last_vertex, added)? else {
            return Ok(Vertex::default());
        };
        let basis2 = self.basis_edge(&e2)?;
        let Some((common, c1, c2)) = self.connections((&basis1, &basis2), (&e1, &e2), added) else {
            self.status = ConstructionError::ConnexionError;
            return Ok(common_vertex(&basis1, &basis2).unwrap_or_default());
        };
        let new1 = restored_edge(&e1, &basis1, (&common, &c1), from_basis)?;
        let new2 = restored_edge(&e2, &basis2, (&common, &c2), from_basis)?;
        self.rebuild_without(added, (&e1, &new1), (&e2, &new2))?;
        self.update_history((&basis1, &basis2), (&new1, &new2));
        Ok(common)
    }

    /// The edge at the vertex other than `added` (`None` on a connexion error, which
    /// sets the status).
    fn other_edge_at(&mut self, v: &Vertex, added: &Edge) -> Result<Option<Edge>, Failure> {
        let (mut adj1, mut adj2) = (Edge::default(), Edge::default());
        self.status = find_connected_edges(&self.new_face, v, &mut adj1, &mut adj2)?;
        if self.status == ConstructionError::ConnexionError {
            return Ok(None);
        }
        Ok(Some(if adj1.is_same(added) { adj2 } else { adj1 }))
    }

    /// The common vertex of the basis edges and the vertices of the adjacent edges on the
    /// removed edge; `None` when one is missing.
    fn connections(
        &self,
        basis: (&Edge, &Edge),
        adj: (&Edge, &Edge),
        added: &Edge,
    ) -> Option<(Vertex, Vertex, Vertex)> {
        let common = common_vertex(basis.0, basis.1)?;
        let c1 = common_vertex(adj.0, added)?;
        let c2 = common_vertex(adj.1, added)?;
        Some((common, c1, c2))
    }

    /// Rebuild the new face: the contour without the removed edge, the adjacent edges
    /// replaced.
    fn rebuild_without(
        &mut self,
        added: &Edge,
        (e1, new1): (&Edge, &Edge),
        (e2, new2): (&Edge, &Edge),
    ) -> Result<(), Failure> {
        let b = Builder;
        let mut new_wire = Wire::default();
        b.make_wire(&mut new_wire);
        let mut ex = Explorer::new(&self.new_face, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while ex.more() {
            let the_edge: &Shape = ex.current();
            if !the_edge.is_same(e1) && !the_edge.is_same(e2) && !the_edge.is_same(added) {
                b.add(&mut new_wire, the_edge)?;
            } else if the_edge == &**e1 {
                b.add(&mut new_wire, new1)?;
            } else if the_edge == &**e2 {
                b.add(&mut new_wire, new2)?;
            }
            ex.next()?;
        }
        self.new_face = self.face_on_reference_plane(&new_wire)?;
        Ok(())
    }
}

/// The adjacent edge restored up to the common vertex: the basis edge when it supports
/// only this fillet (its other vertex is still there), else its curve from the common
/// vertex to the far vertex (a null edge when neither of its vertices is on the removed
/// edge).
fn restored_edge(
    e: &Edge,
    basis: &Edge,
    (common, connection): (&Vertex, &Vertex),
    from_basis: bool,
) -> Result<Edge, Failure> {
    let (first_vertex, last_vertex) = topexp::vertices(e, false);
    let (v1, v2) = topexp::vertices(basis, false);
    let v = if v1.is_same(common) { v2 } else { v1 };
    if first_vertex.is_same(&v) || last_vertex.is_same(&v) {
        // the edge supports only one fillet: the new edge is the basis edge
        return Ok(basis.clone());
    }
    // the edge supports one fillet on each end
    let (start, end) = if first_vertex.is_same(connection) {
        (common.clone(), last_vertex)
    } else if last_vertex.is_same(connection) {
        (first_vertex, common.clone())
    } else {
        return Ok(Edge::default());
    };
    let curve = BRepTool::curve_with_location(e)?.curve;
    let mut make_edge = BRepLibMakeEdge::new();
    make_edge.init_curve_vertices(&curve, &start, &end)?;
    let mut new_edge = make_edge.edge()?;
    let model = if from_basis { basis } else { e };
    new_edge.set_orientation(model.orientation());
    new_edge.set_location(model.location(), false)?;
    Ok(new_edge)
}
