//! `BOPAlgo_PaveFiller_11.cxx`: warnings about self-interferences acquired by an argument.
use super::{BOPAlgoPaveFiller, missing_structure};
use crate::bop_algo::AlertAcquiredSelfIntersection;
use crate::bop_ds::{CommonBlockHandle, CommonBlockHandleIdentity, DataStructure, IndexRange};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::{IndexedDataMap, IndexedMap, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

/// Sub-shapes and the argument shapes connected through them (`aMCSI`).
type Connections = IndexedDataMap<Shape, IndexedMap<Shape, ShapeMapHasher>, ShapeMapHasher>;

impl BOPAlgoPaveFiller {
    /// Warns when one argument acquires a self-interference: one common block holding several
    /// of its edges, or several of its shapes sharing a new vertex or edge
    /// (`CheckSelfInterference`); `filler.check_self_interference()?`.
    pub fn check_self_interference(&mut self) -> Result<(), Failure> {
        if self.arguments.extent() == 1 {
            // Self-interference check mode.
            return Ok(());
        }
        let ranges = self
            .data_structure
            .as_ref()
            .ok_or_else(missing_structure)?
            .nb_ranges();
        for range in 0..ranges {
            let range = *self
                .data_structure
                .as_ref()
                .ok_or_else(missing_structure)?
                .range(range)?;
            let mut connections: Connections = IndexedDataMap::with_hasher(ShapeMapHasher, 1);
            let mut fence: Map<CommonBlockHandle, CommonBlockHandleIdentity> =
                Map::with_hasher(CommonBlockHandleIdentity, 1);
            for index in range.first()..=range.last() {
                self.collect_shape_connections(index, &range, &mut connections, &mut fence)?;
            }
            for position in 1..=connections.extent() {
                let shapes = connections.find_from_index(position)?;
                if shapes.extent() > 1 {
                    let shapes: Vec<Shape> = shapes.iter().cloned().collect();
                    self.warn_acquired_self_intersection(&shapes)?;
                }
            }
        }
        Ok(())
    }

    fn collect_shape_connections(
        &mut self,
        index: i32,
        range: &IndexRange,
        connections: &mut Connections,
        fence: &mut Map<CommonBlockHandle, CommonBlockHandleIdentity>,
    ) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let info = structure.shape_info(index)?;
        if !info.has_reference() {
            return Ok(());
        }
        let shape = info.shape().clone();
        match info.shape_type() {
            ShapeEnum::Edge if !info.has_flag() => {
                let warnings =
                    edge_connections(structure, index, range, &shape, connections, fence)?;
                for edges in warnings {
                    self.warn_acquired_self_intersection(&edges)?;
                }
            }
            ShapeEnum::Face => face_connections(structure, index, &shape, connections)?,
            _ => {}
        }
        Ok(())
    }

    fn warn_acquired_self_intersection(&mut self, shapes: &[Shape]) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        let mut compound = compound.into_shape();
        for shape in shapes {
            builder.add(&mut compound, shape)?;
        }
        let alert: AlertHandle = Arc::new(AlertAcquiredSelfIntersection::new(&compound));
        self.options.add_warning(&alert);
        Ok(())
    }
}

/// Connects the new vertices of a split edge to it and returns the edge groups of common
/// blocks holding several edges of the argument.
fn edge_connections(
    structure: &DataStructure,
    index: i32,
    range: &IndexRange,
    shape: &Shape,
    connections: &mut Connections,
    fence: &mut Map<CommonBlockHandle, CommonBlockHandleIdentity>,
) -> Result<Vec<Vec<Shape>>, Failure> {
    let mut own: Map<i32> = Map::new(1);
    for vertex in structure.shape_info(index)?.sub_shapes() {
        own.add(structure.same_domain_index(*vertex));
    }
    let blocks = structure.pave_blocks(index)?;
    let analyze_vertices = blocks.len() > 1;
    let mut warnings = Vec::new();
    for block in blocks {
        if analyze_vertices {
            let (first, last) = block.lock().expect("pave-block lock").indices();
            for vertex in [first, last] {
                if !range.contains(vertex) && !own.contains(&vertex) {
                    connect(connections, structure.shape(vertex)?, shape);
                }
            }
        }
        let Some(common) = structure.common_block(block) else {
            continue;
        };
        if !fence.add(Arc::clone(&common)) {
            continue;
        }
        let members = common
            .lock()
            .expect("common-block lock")
            .pave_blocks()
            .to_vec();
        let mut edges = Vec::new();
        for member in members {
            let original = member.lock().expect("pave-block lock").original_edge();
            if range.contains(original) {
                edges.push(structure.shape(original)?.clone());
            }
        }
        if edges.len() > 1 {
            // One common block holds several edges of one argument.
            warnings.push(edges);
        }
    }
    Ok(warnings)
}

/// Connects the In/Section vertices and edges of a face to it.
fn face_connections(
    structure: &DataStructure,
    index: i32,
    shape: &Shape,
    connections: &mut Connections,
) -> Result<(), Failure> {
    let info = structure.face_info(index)?;
    for vertices in [info.vertices_in(), info.vertices_section()] {
        for vertex in vertices.iter().copied() {
            connect(connections, structure.shape(vertex)?, shape);
        }
    }
    for blocks in [info.pave_blocks_in(), info.pave_blocks_section()] {
        for block in blocks.iter() {
            let edge = block.lock().expect("pave-block lock").edge();
            if edge < 0 {
                // Native: "Face information is not up to date" assertion, then skipped.
                continue;
            }
            connect(connections, structure.shape(edge)?, shape);
        }
    }
    Ok(())
}

fn connect(connections: &mut Connections, sub_shape: &Shape, shape: &Shape) {
    if let Some(shapes) = connections.change_seek(sub_shape) {
        shapes.add(shape.clone());
        return;
    }
    let mut shapes: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::with_hasher(ShapeMapHasher, 1);
    shapes.add(shape.clone());
    connections.add(sub_shape.clone(), shapes);
}
