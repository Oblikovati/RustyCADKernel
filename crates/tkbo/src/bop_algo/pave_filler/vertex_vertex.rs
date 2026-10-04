use super::BOPAlgoPaveFiller;
use crate::bop_ds::{DataStructure, InterfVV, Pair, ShapeInfo};
use crate::bop_tools::{compute_vertices, make_vertex};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Shape, Vertex},
};
use rustycad_tkernel::{ncollection::List, precision, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;

impl BOPAlgoPaveFiller {
    /// Performs native vertex/vertex candidate intersection and same-domain grouping.
    /// `filler.perform_vv()?` records coincident candidates and updates DS vertices.
    pub fn perform_vv(&mut self) -> Result<(), Failure> {
        let pairs = self.vertex_pairs()?;
        if pairs.is_empty() {
            return Ok(());
        }
        let groups = self.vertex_groups(&pairs)?;
        for group in groups {
            self.make_same_domain_vertices(&group)?;
        }
        self.initialize_vertex_pave_blocks()?;
        Ok(())
    }

    pub(super) fn initialize_vertex_pave_blocks(&mut self) -> Result<(), Failure> {
        let structure = self.data_structure.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformVV requires initialized data structure")
        })?;
        let vertices: Vec<i32> = structure
            .shape_sd_items()
            .map(|(source, _)| *source)
            .collect();
        for vertex in vertices {
            structure.init_pave_blocks_for_vertex(vertex)?;
        }
        Ok(())
    }

    fn vertex_pairs(&mut self) -> Result<Vec<Pair>, Failure> {
        let iterator = self.iterator.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformVV requires initialized iterator")
        })?;
        iterator.initialize(ShapeEnum::Vertex, ShapeEnum::Vertex);
        let mut pairs = Vec::new();
        while iterator.more() {
            let (first, second) = iterator.value()?;
            pairs.push(Pair::new(first, second));
            iterator.next();
        }
        Ok(pairs)
    }

    fn vertex_groups(&self, pairs: &[Pair]) -> Result<Vec<Vec<i32>>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformVV requires initialized data structure")
        })?;
        let mut connections = Vec::new();
        for pair in pairs {
            let (source_first, source_second) = pair.indices();
            if structure.has_interference_between(source_first, source_second) {
                connections.push((source_first, source_second));
                continue;
            }
            let (first, second) = (source_first, source_second);
            let (first, second) = resolved_vertices(structure, first, second);
            let first_shape = Vertex::unchecked(structure.shape(first)?.clone());
            let second_shape = Vertex::unchecked(structure.shape(second)?.clone());
            if compute_vertices(&first_shape, &second_shape, self.options.fuzzy_value())? == 0 {
                connections.push((source_first, source_second));
            }
        }
        Ok(connected_groups(&connections))
    }

    pub(super) fn make_same_domain_vertices(&mut self, group: &[i32]) -> Result<(), Failure> {
        let structure = self.data_structure.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformVV requires initialized data structure")
        })?;
        let shapes = vertex_shapes(structure, group)?;
        let mut merged = Vertex::default();
        make_vertex(&shapes, &mut merged)?;
        let representative = update_or_append_vertex(structure, group, &merged)?;
        update_vertex_bounds(structure, representative)?;
        bind_group_interferences(structure, group, representative);
        Ok(())
    }
}

fn resolved_vertices(structure: &DataStructure, first: i32, second: i32) -> (i32, i32) {
    (
        structure.shape_sd(first).unwrap_or(first),
        structure.shape_sd(second).unwrap_or(second),
    )
}

fn connected_groups(pairs: &[(i32, i32)]) -> Vec<Vec<i32>> {
    let mut groups: Vec<Vec<i32>> = Vec::new();
    for (first, second) in pairs {
        merge_connection(&mut groups, *first, *second);
    }
    groups
}

fn merge_connection(groups: &mut Vec<Vec<i32>>, first: i32, second: i32) {
    let first_group = groups.iter().position(|group| group.contains(&first));
    let second_group = groups.iter().position(|group| group.contains(&second));
    match (first_group, second_group) {
        (None, None) => groups.push(vec![first, second]),
        (Some(index), None) => append_unique(&mut groups[index], second),
        (None, Some(index)) => append_unique(&mut groups[index], first),
        (Some(first_index), Some(second_index)) if first_index != second_index => {
            let (keep_index, remove_index) = if first_index < second_index {
                (first_index, second_index)
            } else {
                (second_index, first_index)
            };
            let second_group = groups.remove(remove_index);
            for index in second_group {
                append_unique(&mut groups[keep_index], index);
            }
        }
        _ => {}
    }
}

fn append_unique(group: &mut Vec<i32>, index: i32) {
    if !group.contains(&index) {
        group.push(index);
    }
}

fn vertex_shapes(structure: &DataStructure, group: &[i32]) -> Result<List<Shape>, Failure> {
    let mut shapes = List::default();
    for index in group {
        if let Some(representative) = structure.shape_sd(*index) {
            shapes.append(structure.shape(representative)?.clone());
        }
        shapes.append(structure.shape(*index)?.clone());
    }
    Ok(shapes)
}

fn update_or_append_vertex(
    structure: &mut DataStructure,
    group: &[i32],
    merged: &Vertex,
) -> Result<i32, Failure> {
    if let Some(index) = group.iter().find_map(|source| structure.shape_sd(*source)) {
        let existing = Vertex::unchecked(structure.shape(index)?.clone());
        BRepBuilder::default().set_vertex_point_and_tolerance(
            &existing,
            BRepTool::pnt(merged)?,
            BRepTool::vertex_tolerance(merged)?,
        )?;
        return Ok(index);
    }
    append_merged_vertex(structure, merged)
}

fn update_vertex_bounds(structure: &mut DataStructure, index: i32) -> Result<(), Failure> {
    let vertex = Vertex::unchecked(structure.shape(index)?.clone());
    let point = BRepTool::pnt(&vertex)?;
    let gap = BRepTool::vertex_tolerance(&vertex)? + precision::confusion();
    let bounds = structure.change_shape_info(index)?.change_box();
    bounds.add_point(point);
    bounds.set_gap(gap);
    Ok(())
}

fn append_merged_vertex(structure: &mut DataStructure, vertex: &Vertex) -> Result<i32, Failure> {
    let mut info = ShapeInfo::default();
    info.set_shape_type(ShapeEnum::Vertex);
    info.set_shape(&vertex.clone().into_shape());
    let point = BRepTool::pnt(vertex)?;
    let gap = BRepTool::vertex_tolerance(vertex)? + precision::confusion();
    let mut bounds = BoundingBox3::default();
    bounds.add_point(point);
    bounds.set_gap(gap);
    info.set_box(&bounds);
    Ok(structure.append_shape_info(&info))
}

fn bind_group_interferences(structure: &mut DataStructure, group: &[i32], new_index: i32) {
    for (position, first) in group.iter().enumerate() {
        structure.add_shape_sd(*first, new_index);
        for second in group.iter().skip(position + 1) {
            if structure.add_interference(*first, *second) {
                let record = structure.interf_vv().append(InterfVV::default());
                record.set_indices(*first, *second);
                record.set_index_new(new_index);
            }
        }
    }
}
