//! Sub-iterator selection for generated EE vertices and source faces.
use super::{BOPAlgoPaveFiller, DataStructure, Failure, ShapeEnum, append_vf_record};
use crate::bop_tools::{BoxSet, PairSelector};
use rustycad_tkmath::{bnd::box3_to_bvh, bvh::BvhVec3d};

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

fn subset_boxes(ds: &DataStructure, indices: &[i32]) -> Result<BoxSet<BvhVec3d>, Failure> {
    let mut boxes = BoxSet::default();
    boxes.set_size(indices.len());
    for index in indices {
        boxes.add(*index, box3_to_bvh(ds.shape_info(*index)?.box_bounds())?);
    }
    boxes.build()?;
    Ok(boxes)
}

fn vertex_face_subset_pairs(
    ds: &DataStructure,
    vertices: &[i32],
    faces: &[i32],
) -> Result<Vec<(i32, i32)>, Failure> {
    if vertices.is_empty() || faces.is_empty() {
        return Ok(Vec::new());
    }
    let mut first = subset_boxes(ds, faces)?;
    let mut second = subset_boxes(ds, vertices)?;
    let mut selector = PairSelector::default();
    selector.set_bvh_sets(&mut first, &mut second);
    selector.select()?;
    let mut pairs = selector
        .pairs()
        .iter()
        .map(|pair| (pair.id1.min(pair.id2), pair.id1.max(pair.id2)))
        .collect::<Vec<_>>();
    pairs.sort();
    pairs.dedup();
    let mut output = Vec::new();
    for (first, second) in pairs {
        append_subset_pair(ds, first, second, &mut output)?;
    }
    Ok(output)
}

fn append_subset_pair(
    ds: &DataStructure,
    first: i32,
    second: i32,
    pairs: &mut Vec<(i32, i32)>,
) -> Result<(), Failure> {
    let first_type = ds.shape_info(first)?.shape_type();
    let (vertex, face) = if first_type == ShapeEnum::Vertex {
        (first, second)
    } else {
        (second, first)
    };
    if vertex != face && !ds.shape_info(face)?.has_sub_shape(vertex) {
        pairs.push((vertex, face));
    }
    Ok(())
}
