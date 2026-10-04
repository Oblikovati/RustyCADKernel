//! Sub-iterator selection for generated EE vertices and source faces.
use super::{BOPAlgoPaveFiller, DataStructure, Failure, ShapeEnum, append_vf_record};
use crate::bop_ds::BOPDSSubIterator;
use rustycad_tkernel::ncollection::List;

impl BOPAlgoPaveFiller {
    pub(super) fn treat_vertices_ee(&mut self) -> Result<(), Failure> {
        let ds = self.vf_structure()?;
        let vertices = generated_ee_vertices(ds);
        if vertices.is_empty() {
            return Ok(());
        }
        let faces = (0..ds.nb_source_shapes())
            .filter(|index| {
                ds.shape_info(*index)
                    .is_ok_and(|info| info.shape_type() == ShapeEnum::Face)
            })
            .collect::<Vec<_>>();
        let pairs = vertex_face_subset_pairs(ds, &vertices, &faces)?;
        for (vertex, face) in pairs {
            self.accept_ee_vertex_on_face(vertex, face)?;
        }
        Ok(())
    }

    fn accept_ee_vertex_on_face(&mut self, vertex: i32, face: i32) -> Result<(), Failure> {
        if self
            .vf_structure()?
            .change_face_info(face)?
            .vertices_on()
            .contains(vertex)
        {
            return Ok(());
        }
        let Some(hit) = self.compute_vertex_face(vertex, face, false)? else {
            return Ok(());
        };
        let ds = self.vf_structure()?;
        append_vf_record(ds, vertex, face, hit);
        ds.change_face_info(face)?.change_vertices_in().add(vertex);
        Ok(())
    }
}

fn generated_ee_vertices(ds: &mut DataStructure) -> Vec<i32> {
    let mut vertices = Vec::new();
    for record in ds.interf_ee().iter() {
        if let Some(index) = record.get_index_new()
            && !vertices.contains(&index)
        {
            vertices.push(index);
        }
    }
    vertices
}

fn vertex_face_subset_pairs(
    ds: &DataStructure,
    vertices: &[i32],
    faces: &[i32],
) -> Result<Vec<(i32, i32)>, Failure> {
    let first: List<i32> = faces.iter().copied().collect();
    let second: List<i32> = vertices.iter().copied().collect();
    let mut iterator = BOPDSSubIterator::default();
    iterator.set_ds(Some(ds));
    iterator.set_subset1(&first);
    iterator.set_subset2(&second);
    iterator.prepare()?;
    iterator.initialize();
    let mut pairs = Vec::new();
    while iterator.more() {
        pairs.push(iterator.value()?);
        iterator.next();
    }
    Ok(pairs)
}
