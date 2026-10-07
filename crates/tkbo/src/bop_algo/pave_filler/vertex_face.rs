//! `BOPAlgo_PaveFiller_4.cxx`: vertex/face intersection and the treatment of EE vertices.
use super::{BOPAlgoPaveFiller, Glue, missing_structure};
use crate::bop_ds::{BOPDSSubIterator, InterfVF, Pair};
use rustycad_tkbrep::topods::{Face, Vertex};
use rustycad_tkernel::ncollection::{DataMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// One prepared `BOPAlgo_VertexFace` solver and its computed result.
struct VertexFaceSolver {
    vertex: i32,
    face: i32,
    flag: i32,
    uv: (f64, f64),
    new_tolerance: f64,
    failed: bool,
}

impl BOPAlgoPaveFiller {
    /// Intersects vertices with faces, recording VF interferences and the faces' In vertices,
    /// then treats the vertices created by EE intersections (`PerformVF`); `filler.perform_vf()?`.
    pub fn perform_vf(&mut self) -> Result<(), Failure> {
        let pairs = self.iterator_pairs(ShapeEnum::Vertex, ShapeEnum::Face)?;
        if self.glue == Glue::Full {
            // No vertex/face intersection in this mode: only the face records are initialized.
            let structure = self.structure_mut()?;
            for (vertex, face) in pairs {
                if !structure.is_sub_shape(vertex, face)? {
                    structure.change_face_info(face)?;
                }
            }
            return Ok(());
        }
        if pairs.is_empty() {
            return self.treat_vertices_ee();
        }
        let mut grouped: DataMap<Pair, Map<i32>> = DataMap::default();
        let mut solvers = Vec::new();
        for (vertex, face) in pairs {
            if let Some(solver) = self.vf_candidate(vertex, face, &mut grouped)? {
                solvers.push(solver);
            }
        }
        for solver in &mut solvers {
            self.compute_vf_solver(solver)?;
        }
        for solver in &solvers {
            self.apply_vf_solver(solver, &grouped)?;
        }
        self.treat_vertices_ee()
    }

    /// Classifies every vertex created by an EE intersection against the source faces it is
    /// not On, adding VF interferences and In vertices (`TreatVerticesEE`).
    /// `filler.treat_vertices_ee()?`.
    pub fn treat_vertices_ee(&mut self) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut fence: Map<i32> = Map::new(100);
        let mut vertices: List<i32> = List::default();
        for record in structure.interf_ee_records().iter() {
            if let Some(vertex) = record.get_index_new()
                && fence.add(vertex)
            {
                vertices.append(vertex);
            }
        }
        if vertices.is_empty() {
            return Ok(());
        }
        let mut faces: List<i32> = List::default();
        for index in 0..structure.nb_source_shapes() {
            if structure.shape_info(index)?.shape_type() == ShapeEnum::Face {
                faces.append(index);
            }
        }
        if faces.is_empty() {
            return Ok(());
        }
        let pairs = subset_pairs(structure, &faces, &vertices)?;
        for (vertex, face) in pairs {
            self.treat_ee_vertex_on_face(vertex, face)?;
        }
        Ok(())
    }

    fn vf_candidate(
        &mut self,
        vertex: i32,
        face: i32,
        grouped: &mut DataMap<Pair, Map<i32>>,
    ) -> Result<Option<VertexFaceSolver>, Failure> {
        let structure = self.structure_mut()?;
        if structure.is_sub_shape(vertex, face)? || structure.has_interference_between(vertex, face)
        {
            return Ok(None);
        }
        structure.change_face_info(face)?;
        if structure.has_interference_shape_subshapes(vertex, face)? {
            return Ok(None);
        }
        let mut representative = vertex;
        structure.has_shape_sd_into(vertex, &mut representative);
        let key = Pair::new(representative, face);
        if let Some(group) = grouped.change_seek(&key) {
            group.add(vertex);
            return Ok(None);
        }
        let mut group: Map<i32> = Map::new(1);
        group.add(vertex);
        grouped.bind(key, group);
        Ok(Some(VertexFaceSolver {
            vertex: representative,
            face,
            flag: -1,
            uv: (-1., -1.),
            new_tolerance: -1.,
            failed: false,
        }))
    }

    fn compute_vf_solver(&self, solver: &mut VertexFaceSolver) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let vertex = Vertex::unchecked(structure.shape(solver.vertex)?.clone());
        let face = Face::unchecked(structure.shape(solver.face)?.clone());
        let context = self.context.get().ok_or_else(missing_structure)?;
        let (u, v) = (&mut solver.uv.0, &mut solver.uv.1);
        // An exception of ComputeVF becomes the solver's error; its flag stays -1.
        match context.compute_vf(
            &vertex,
            &face,
            u,
            v,
            &mut solver.new_tolerance,
            self.options.fuzzy_value(),
        ) {
            Ok(flag) => solver.flag = flag,
            Err(_) => solver.failed = true,
        }
        Ok(())
    }

    fn apply_vf_solver(
        &mut self,
        solver: &VertexFaceSolver,
        grouped: &DataMap<Pair, Map<i32>>,
    ) -> Result<(), Failure> {
        if solver.flag != 0 {
            if solver.failed {
                let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
                let vertex = structure.shape(solver.vertex)?.clone();
                let face = structure.shape(solver.face)?.clone();
                self.add_intersection_failed_warning(&vertex, &face)?;
            }
            return Ok(());
        }
        let originals: Vec<i32> = grouped
            .find(&Pair::new(solver.vertex, solver.face))?
            .iter()
            .copied()
            .collect();
        let mut updated = solver.vertex;
        for original in originals {
            let structure = self.structure_mut()?;
            let position = structure.interf_vf_records().len();
            let record = structure.interf_vf().append(InterfVF::default());
            record.set_indices(original, solver.face);
            record.set_uv(solver.uv.0, solver.uv.1);
            structure.add_interference(original, solver.face);
            updated = self.update_vertex(original, solver.new_tolerance)?;
            let structure = self.structure_mut()?;
            if structure.is_new_shape(updated) {
                structure.interf_vf()[position].set_index_new(updated);
            }
        }
        self.structure_mut()?
            .change_face_info(solver.face)?
            .change_vertices_in()
            .add(updated);
        Ok(())
    }

    fn treat_ee_vertex_on_face(&mut self, vertex: i32, face: i32) -> Result<(), Failure> {
        let fuzzy = self.options.fuzzy_value();
        let context = self.context.clone();
        let structure = self.structure_mut()?;
        if structure
            .change_face_info(face)?
            .vertices_on()
            .contains(vertex)
        {
            return Ok(());
        }
        let shape = Vertex::unchecked(structure.shape(vertex)?.clone());
        let face_shape = Face::unchecked(structure.shape(face)?.clone());
        let (mut u, mut v, mut dummy) = (0., 0., 0.);
        let context = context.get().ok_or_else(missing_structure)?;
        if context.compute_vf(&shape, &face_shape, &mut u, &mut v, &mut dummy, fuzzy)? != 0 {
            return Ok(());
        }
        let record = structure.interf_vf().append(InterfVF::default());
        record.set_indices(vertex, face);
        record.set_uv(u, v);
        structure.add_interference(vertex, face);
        structure
            .change_face_info(face)?
            .change_vertices_in()
            .add(vertex);
        Ok(())
    }
}

fn subset_pairs(
    structure: &crate::bop_ds::DataStructure,
    faces: &List<i32>,
    vertices: &List<i32>,
) -> Result<Vec<(i32, i32)>, Failure> {
    let mut iterator = BOPDSSubIterator::new();
    iterator.set_ds(Some(structure));
    iterator.set_subset1(faces);
    iterator.set_subset2(vertices);
    iterator.prepare()?;
    iterator.initialize();
    let mut pairs = Vec::new();
    while iterator.more() {
        pairs.push(iterator.value()?);
        iterator.next();
    }
    Ok(pairs)
}
