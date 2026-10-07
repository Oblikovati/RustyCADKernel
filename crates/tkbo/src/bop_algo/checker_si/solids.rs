//! `BOPAlgo_CheckerSI_1.cxx`: the interferences of vertices, edges, faces and solids with
//! the solids of the checked shape (`PerformVZ`, `PerformEZ`, `PerformFZ`, `PerformZZ`).
use super::BOPAlgoCheckerSI;
use crate::bop_ds::{DataStructure, InterfEZ, InterfFZ, InterfVZ, InterfZZ, Pair};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

impl BOPAlgoCheckerSI {
    /// Records the vertices classified inside a solid of the shape, each vertex (or its
    /// same-domain vertex) once per solid, skipping the vertices of the solid itself
    /// (`PerformVZ`).
    pub(super) fn perform_vz(&mut self) -> Result<(), Failure> {
        let pairs = self.candidate_pairs(ShapeEnum::Vertex)?;
        if pairs.is_empty() {
            return Ok(());
        }
        let context = self.filler.context_handle();
        let context = context.get().ok_or_else(super::missing_structure)?;
        let structure = self.structure_mut()?;
        structure.interf_vz().set_increment(pairs.len());
        for task in vertex_solid_tasks(structure, &pairs)? {
            if vertex_state(structure, task.classified, task.solid, context)? == State::In {
                structure
                    .interf_vz()
                    .append(InterfVZ::default())
                    .set_indices(task.vertex, task.solid);
                structure.add_interference(task.vertex, task.solid);
            }
        }
        Ok(())
    }

    /// The edges that interfere with a sub-shape of a solid, or the reverse (`PerformEZ`).
    pub(super) fn perform_ez(&mut self) -> Result<(), Failure> {
        self.perform_sz(ShapeEnum::Edge)
    }

    /// The faces that interfere with a sub-shape of a solid, or the reverse (`PerformFZ`).
    pub(super) fn perform_fz(&mut self) -> Result<(), Failure> {
        self.perform_sz(ShapeEnum::Face)
    }

    /// The solid pairs where every sub-shape of one interferes with the other
    /// (`BOPAlgo_SolidSolid` tasks of `PerformZZ`).
    pub(super) fn perform_zz(&mut self) -> Result<(), Failure> {
        let pairs = self.candidate_pairs(ShapeEnum::Solid)?;
        if pairs.is_empty() {
            return Ok(());
        }
        let structure = self.structure_mut()?;
        let mut found = Vec::new();
        for &(first, second) in &pairs {
            if structure.has_interference_shape_subshapes_mode(second, first, false)?
                || structure.has_interference_shape_subshapes_mode(first, second, false)?
            {
                found.push((first, second));
            }
        }
        structure.interf_zz().set_increment(pairs.len());
        for (first, second) in found {
            structure
                .interf_zz()
                .append(InterfZZ::default())
                .set_indices(first, second);
            structure.add_interference(first, second);
        }
        Ok(())
    }

    /// The edge or face/solid pairs where the shape interferes with a sub-shape of the solid,
    /// or the solid with a sub-shape of the shape (`BOPAlgo_ShapeSolid` tasks of `PerformSZ`).
    fn perform_sz(&mut self, kind: ShapeEnum) -> Result<(), Failure> {
        let pairs = self.candidate_pairs(kind)?;
        if pairs.is_empty() {
            return Ok(());
        }
        let structure = self.structure_mut()?;
        let mut found = Vec::new();
        for &(shape, solid) in &pairs {
            if structure.has_interference_shape_subshapes(shape, solid)?
                || structure.has_interference_shape_subshapes(solid, shape)?
            {
                found.push((shape, solid));
            }
        }
        if kind == ShapeEnum::Edge {
            structure.interf_ez().set_increment(pairs.len());
        } else {
            structure.interf_fz().set_increment(pairs.len());
        }
        for (shape, solid) in found {
            record_shape_solid(structure, kind, shape, solid);
            structure.add_interference(shape, solid);
        }
        Ok(())
    }

    /// The broad-phase pairs of `kind` with a solid, in iterator order.
    fn candidate_pairs(&mut self, kind: ShapeEnum) -> Result<Vec<(i32, i32)>, Failure> {
        let iterator = self
            .filler
            .iterator_mut()
            .ok_or_else(super::missing_structure)?;
        iterator.initialize(kind, ShapeEnum::Solid);
        let mut pairs = Vec::new();
        while iterator.more() {
            pairs.push(iterator.value()?);
            iterator.next();
        }
        Ok(pairs)
    }
}

/// One vertex/solid classification (`BOPAlgo_VertexSolid`): the record names the source
/// vertex, the classified point is its same-domain vertex.
struct VertexSolidTask {
    vertex: i32,
    classified: i32,
    solid: i32,
}

/// The vertex/solid classifications to run, skipping a vertex of the solid and a repeated
/// classification of the same vertex and solid.
fn vertex_solid_tasks(
    structure: &DataStructure,
    pairs: &[(i32, i32)],
) -> Result<Vec<VertexSolidTask>, Failure> {
    let mut fence: Map<Pair> = Map::new(1);
    let mut tasks = Vec::new();
    for &(vertex, solid) in pairs {
        if structure.has_interference_shape_subshapes(vertex, solid)? {
            continue;
        }
        let mut classified = vertex;
        structure.has_shape_sd_into(vertex, &mut classified);
        if fence.add(Pair::new(classified, solid)) {
            tasks.push(VertexSolidTask {
                vertex,
                classified,
                solid,
            });
        }
    }
    Ok(tasks)
}

/// The state of a vertex point against a solid, with the vertex tolerance
/// (`BOPAlgo_VertexSolid::Perform`).
fn vertex_state(
    structure: &DataStructure,
    vertex: i32,
    solid: i32,
    context: &Context,
) -> Result<State, Failure> {
    let vertex = topods::vertex(structure.shape(vertex)?)?;
    let solid = topods::solid(structure.shape(solid)?)?;
    let classifier = context.solid_classifier(&solid)?;
    let mut classifier = classifier.write();
    classifier.perform(
        &BRepTool::pnt(&vertex)?,
        BRepTool::vertex_tolerance(&vertex)?,
    )?;
    Ok(classifier.state())
}

/// Appends the edge/solid or face/solid interference record.
fn record_shape_solid(structure: &mut DataStructure, kind: ShapeEnum, shape: i32, solid: i32) {
    if kind == ShapeEnum::Edge {
        structure
            .interf_ez()
            .append(InterfEZ::default())
            .set_indices(shape, solid);
    } else {
        structure
            .interf_fz()
            .append(InterfFZ::default())
            .set_indices(shape, solid);
    }
}
