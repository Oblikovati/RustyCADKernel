// Rust translation of Ensure (three overloads), Needs and the private collectDirty
// members of ModelingData/TKBRep/BRepGraph/BRepGraph_CacheMesh.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Finding the stale meshes of a slot and asking its driver to refill them.
use super::entries::Slot;
use super::{CacheMesh, CacheMeshDriver, DirtySet, SlotId};
use crate::brep_graph::child_explorer::{ChildExplorer, TraversalMode};
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId, NodeId, NodeKind};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::FlatMap;
use rustycad_tkernel::standard::{Failure, Handle};

// The driver and the dirty set `Ensure` hands it.
type PreparedEnsure = (Handle<dyn CacheMeshDriver>, DirtySet);

// What `Ensure` collects before it calls the driver.
enum DirtyScope<'n> {
    // The whole graph.
    Graph,
    // The subtrees of the nodes.
    Nodes(&'n [NodeId]),
}

impl CacheMesh {
    /// `Ensure(theGraph, theSlot, theRange)`: refills the stale faces and free edges of
    /// the whole graph with the slot's driver. False without a driver; true when
    /// nothing is stale; else the driver's answer. A changed driver recipe clears the
    /// slot first.
    pub fn ensure(
        &self,
        graph: &mut BRepGraph,
        slot: SlotId,
        range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.ensure_scope(graph, slot, DirtyScope::Graph, range)
    }

    /// `Ensure(theGraph, theRoot, theSlot, theRange)`: the subtree of one node.
    pub fn ensure_root(
        &self,
        graph: &mut BRepGraph,
        root: NodeId,
        slot: SlotId,
        range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.ensure_scope(graph, slot, DirtyScope::Nodes(&[root]), range)
    }

    /// `Ensure(theGraph, theNodes, theSlot, theRange)`: the subtrees of the nodes.
    pub fn ensure_nodes(
        &self,
        graph: &mut BRepGraph,
        nodes: &[NodeId],
        slot: SlotId,
        range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.ensure_scope(graph, slot, DirtyScope::Nodes(nodes), range)
    }

    // The three `Ensure` overloads.
    fn ensure_scope(
        &self,
        graph: &mut BRepGraph,
        slot: SlotId,
        scope: DirtyScope<'_>,
        range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let Some((driver, dirty)) = self.prepare_ensure(graph, slot, scope)? else {
            return Ok(false);
        };
        let Some(driver) = driver.get() else {
            return Ok(false);
        };
        if dirty.is_empty() {
            return Ok(true);
        }
        driver.fill(graph, slot, &dirty, range)
    }

    // The locked part of `Ensure`: the slot (created), a recipe change clearing it,
    // and the dirty set; `None` without a driver.
    fn prepare_ensure(
        &self,
        graph: &BRepGraph,
        slot: SlotId,
        scope: DirtyScope<'_>,
    ) -> Result<Option<PreparedEnsure>, Failure> {
        let mut slots = self.slots.write();
        let found = slots.change_slot(slot);
        let Some(driver) = found.driver.get() else {
            return Ok(None);
        };
        let recipe = driver.recipe_hash();
        if found.recipe_hash != recipe {
            found.recipe_hash = recipe;
            found.clear();
        }
        let dirty = self.collect(graph, found, scope)?;
        Ok(Some((found.driver.clone(), dirty)))
    }

    /// `Needs(theGraph, theRoot, theSlot)`: a missing slot, a changed driver recipe, or
    /// stale meshes below the node (without calling the driver).
    pub fn needs(&self, graph: &BRepGraph, root: NodeId, slot: SlotId) -> Result<bool, Failure> {
        let slots = self.slots.read();
        let Some(found) = slots.find_slot(slot) else {
            return Ok(true);
        };
        let recipe_changed = found
            .driver
            .get()
            .is_some_and(|driver| found.recipe_hash != driver.recipe_hash());
        if recipe_changed {
            return Ok(true);
        }
        Ok(!self
            .collect(graph, found, DirtyScope::Nodes(&[root]))?
            .is_empty())
    }

    fn collect(
        &self,
        graph: &BRepGraph,
        slot: &Slot,
        scope: DirtyScope<'_>,
    ) -> Result<DirtySet, Failure> {
        match scope {
            DirtyScope::Graph => self.collect_dirty(graph, slot),
            DirtyScope::Nodes(nodes) => self.collect_dirty_nodes(graph, nodes, slot),
        }
    }

    /// `collectDirty(theGraph, theSlot)` (private): every active face without a fresh
    /// entry, every active free edge without one, then every other active face with a
    /// stale polygon on triangulation among its coedges.
    fn collect_dirty(&self, graph: &BRepGraph, slot: &Slot) -> Result<DirtySet, Failure> {
        let mut dirty = DirtySet::default();
        let faces: Vec<FaceId> = (0..graph.topo().faces().nb())
            .map(FaceId::new)
            .filter(|face| !face.is_removed(graph))
            .collect();
        for &face in &faces {
            if !self.has_fresh_face(graph, slot, face) {
                dirty.faces.push(face);
            }
        }
        for edge in (0..graph.topo().edges().nb()).map(EdgeId::new) {
            if is_free_edge(graph, edge) && !self.has_fresh_edge(graph, slot, edge) {
                dirty.free_edges.push(edge);
            }
        }
        self.add_stale_co_edge_faces(graph, slot, &faces, &mut dirty)?;
        Ok(dirty)
    }

    /// `collectDirty(theGraph, theNodes, theSlot)` (private): the faces and free edges
    /// of the nodes and their subtrees, each once, then the faces (in set order) with a
    /// stale polygon on triangulation.
    fn collect_dirty_nodes(
        &self,
        graph: &BRepGraph,
        nodes: &[NodeId],
        slot: &Slot,
    ) -> Result<DirtySet, Failure> {
        let mut scan = NodeScan {
            cache: self,
            graph,
            slot,
            faces: FlatMap::default(),
            free_edges: FlatMap::default(),
            dirty: DirtySet::default(),
        };
        for &node in nodes.iter().filter(|node| node.is_valid()) {
            scan.add_node(node)?;
        }
        let faces: Vec<FaceId> = scan.faces.iter().copied().collect();
        let mut dirty = scan.dirty;
        self.add_stale_co_edge_faces(graph, slot, &faces, &mut dirty)?;
        Ok(dirty)
    }

    // The coedge pass of `collectDirty`: a face not dirty yet becomes dirty when one of
    // its coedges has a present entry whose polygons on triangulation are stale.
    fn add_stale_co_edge_faces(
        &self,
        graph: &BRepGraph,
        slot: &Slot,
        faces: &[FaceId],
        dirty: &mut DirtySet,
    ) -> Result<(), Failure> {
        for &face in faces {
            if dirty.faces.contains(&face) {
                continue;
            }
            if self.has_stale_co_edge(graph, slot, face)? {
                dirty.faces.push(face);
            }
        }
        Ok(())
    }

    fn has_stale_co_edge(
        &self,
        graph: &BRepGraph,
        slot: &Slot,
        face: FaceId,
    ) -> Result<bool, Failure> {
        let mut explorer = ChildExplorer::with_target_mode(
            graph,
            face,
            NodeKind::CO_EDGE,
            TraversalMode::Recursive,
        )?;
        while explorer.more() {
            let co_edge = CoEdgeId::from_node_id(explorer.current().def_id);
            let stale = slot
                .co_edge_raw(co_edge)
                .is_some_and(|raw| !self.is_co_edge_polygon_on_tri_fresh(graph, raw, slot));
            if stale {
                return Ok(true);
            }
            explorer.next()?;
        }
        Ok(false)
    }

    fn has_fresh_face(&self, graph: &BRepGraph, slot: &Slot, face: FaceId) -> bool {
        let entry = slot
            .face_meshes
            .get(face.index as usize)
            .filter(|_| face.is_valid());
        entry.is_some_and(|entry| self.is_face_entry_fresh(graph, entry, slot))
    }

    fn has_fresh_edge(&self, graph: &BRepGraph, slot: &Slot, edge: EdgeId) -> bool {
        let entry = slot
            .edge_meshes
            .get(edge.index as usize)
            .filter(|_| edge.is_valid());
        entry.is_some_and(|entry| self.is_edge_entry_fresh(graph, entry, slot))
    }
}

// An active edge used by no face.
fn is_free_edge(graph: &BRepGraph, edge: EdgeId) -> bool {
    !edge.is_removed(graph) && !graph.topo().edges().faces_of(edge).more()
}

// The node walk of `collectDirty(theGraph, theNodes, theSlot)` (its `addFace` and
// `addFreeEdge` lambdas and their sets).
struct NodeScan<'s> {
    cache: &'s CacheMesh,
    graph: &'s BRepGraph,
    slot: &'s Slot,
    faces: FlatMap<FaceId>,
    free_edges: FlatMap<EdgeId>,
    dirty: DirtySet,
}

impl NodeScan<'_> {
    // One requested node: itself when a face or an edge, then the faces and the edges
    // of its subtree.
    fn add_node(&mut self, node: NodeId) -> Result<(), Failure> {
        if node.kind == NodeKind::FACE {
            self.add_face(FaceId::from_node_id(node));
        } else if node.kind == NodeKind::EDGE {
            self.add_free_edge(EdgeId::from_node_id(node));
        }
        let mut faces = ChildExplorer::with_target(self.graph, node, NodeKind::FACE)?;
        while faces.more() {
            self.add_face(FaceId::from_node_id(faces.current().def_id));
            faces.next()?;
        }
        let mut edges = ChildExplorer::with_target(self.graph, node, NodeKind::EDGE)?;
        while edges.more() {
            self.add_free_edge(EdgeId::from_node_id(edges.current().def_id));
            edges.next()?;
        }
        Ok(())
    }

    // `addFace`: an active face seen for the first time, dirty without a fresh entry.
    fn add_face(&mut self, face: FaceId) {
        if face.is_removed(self.graph) || !self.faces.add(face) {
            return;
        }
        if !self.cache.has_fresh_face(self.graph, self.slot, face) {
            self.dirty.faces.push(face);
        }
    }

    // `addFreeEdge`: an active free edge seen for the first time, dirty without a
    // fresh entry.
    fn add_free_edge(&mut self, edge: EdgeId) {
        if !is_free_edge(self.graph, edge) || !self.free_edges.add(edge) {
            return;
        }
        if !self.cache.has_fresh_edge(self.graph, self.slot, edge) {
            self.dirty.free_edges.push(edge);
        }
    }
}
