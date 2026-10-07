//! `BOPAlgo_PaveFiller_10.cxx`: tolerance updates of DS shapes and same-domain propagation.
use super::{BOPAlgoPaveFiller, missing_structure};
use crate::bop_ds::{CommonBlockHandle, DataStructure, Interference, ShapeInfo};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::ncollection::DynamicArray;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use std::ops::DerefMut;
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    /// Raises an edge's tolerance and those of its vertices; in non-destructive mode source
    /// edges, or new edges with untouched source vertices, are left alone.
    /// `filler.update_edge_tolerance(edge, 1e-4)?`.
    pub fn update_edge_tolerance(&mut self, edge: i32, tolerance: f64) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let vertices = structure.shape_info(edge)?.sub_shapes().to_vec();
        if self.non_destructive && !edge_is_modifiable(structure, edge, &vertices) {
            return Ok(());
        }
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        let shape = Edge::unchecked(structure.shape(edge)?.clone());
        BRepBuilder::default().update_edge_tolerance(&shape, tolerance)?;
        let bounds = structure.change_shape_info(edge)?.change_box();
        BRepBndLib::add(&shape, bounds, true)?;
        bounds.set_gap(bounds.gap() + precision::confusion());
        for vertex in vertices {
            self.update_vertex(vertex, tolerance)?;
        }
        Ok(())
    }

    /// Raises a vertex tolerance, or in non-destructive mode replaces an untouched source
    /// vertex by a same-domain copy; returns the index carrying the new tolerance.
    /// `let updated = filler.update_vertex(vertex, 1e-4)?;`.
    pub fn update_vertex(&mut self, vertex: i32, tolerance: f64) -> Result<i32, Failure> {
        let non_destructive = self.non_destructive;
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        let mut target = vertex;
        let has_same_domain = structure.has_shape_sd_into(vertex, &mut target);
        if structure.is_new_shape(vertex) || has_same_domain || !non_destructive {
            if raise_vertex_tolerance(structure, target, tolerance)? {
                self.increased_ss.add(vertex);
            }
            return Ok(target);
        }
        let (copy, increased) = append_tolerant_copy(structure, vertex, tolerance)?;
        if !self.vertices_to_avoid_extension.contains(&copy) {
            self.vertices_to_avoid_extension.push(copy);
        }
        if increased {
            self.increased_ss.add(vertex);
        }
        Ok(copy)
    }

    /// Replaces pave indices of all pool blocks by same-domain representatives.
    /// `filler.update_pave_blocks_with_sd_vertices()?`.
    pub fn update_pave_blocks_with_sd_vertices(&mut self) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        structure.update_pave_blocks_with_sd_vertices();
        Ok(())
    }

    /// In non-destructive mode first gives each common block's end vertices a confusion
    /// tolerance copy, then updates all paves; `filler.update_common_blocks_with_sd_vertices()?`.
    pub fn update_common_blocks_with_sd_vertices(&mut self) -> Result<(), Failure> {
        if !self.non_destructive {
            return self.update_pave_blocks_with_sd_vertices();
        }
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        if structure.pave_blocks_pool().is_empty() {
            return Ok(());
        }
        for common in distinct_common_blocks(structure) {
            self.update_common_block_vertices(&common)?;
        }
        self.update_pave_blocks_with_sd_vertices()
    }

    /// Redirects the new-vertex index of VV/VE/VF/EE/EF records to its same-domain vertex.
    /// `filler.update_interfs_with_sd_vertices()?`.
    pub fn update_interfs_with_sd_vertices(&mut self) -> Result<(), Failure> {
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        let same_domain = |index: i32, structure: &DataStructure| {
            let mut representative = index;
            structure
                .has_shape_sd_into(index, &mut representative)
                .then_some(representative)
        };
        let mut updates: Vec<(usize, usize, i32)> = Vec::new();
        collect_sd_updates(
            0,
            structure.interf_vv_records(),
            structure,
            &same_domain,
            &mut updates,
        );
        collect_sd_updates(
            1,
            structure.interf_ve_records(),
            structure,
            &same_domain,
            &mut updates,
        );
        collect_sd_updates(
            2,
            structure.interf_vf_records(),
            structure,
            &same_domain,
            &mut updates,
        );
        collect_sd_updates(
            3,
            structure.interf_ee_records(),
            structure,
            &same_domain,
            &mut updates,
        );
        collect_sd_updates(
            4,
            structure.interf_ef_records(),
            structure,
            &same_domain,
            &mut updates,
        );
        for (kind, position, index) in updates {
            apply_sd_update(structure, kind, position, index);
        }
        Ok(())
    }

    fn update_common_block_vertices(&mut self, common: &CommonBlockHandle) -> Result<(), Failure> {
        let first = Arc::clone(
            common
                .lock()
                .expect("common-block lock must not be poisoned")
                .pave_block1()?,
        );
        let (first_vertex, last_vertex) = first
            .lock()
            .expect("pave-block lock must not be poisoned")
            .indices();
        self.update_vertex(first_vertex, precision::confusion())?;
        self.update_vertex(last_vertex, precision::confusion())?;
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        structure.update_common_block_with_sd_vertices(common);
        Ok(())
    }
}

fn edge_is_modifiable(structure: &DataStructure, edge: i32, vertices: &[i32]) -> bool {
    if !structure.is_new_shape(edge) {
        return false;
    }
    vertices.iter().all(|vertex| {
        let mut representative = *vertex;
        structure.is_new_shape(*vertex) || structure.has_shape_sd_into(*vertex, &mut representative)
    })
}

/// Returns true when the stored tolerance was below `tolerance` and has been raised.
fn raise_vertex_tolerance(
    structure: &mut DataStructure,
    index: i32,
    tolerance: f64,
) -> Result<bool, Failure> {
    let vertex = Vertex::unchecked(structure.shape(index)?.clone());
    if BRepTool::vertex_tolerance(&vertex)? >= tolerance {
        return Ok(false);
    }
    BRepBuilder::default().update_vertex_tolerance(&vertex, tolerance)?;
    let bounds = structure.change_shape_info(index)?.change_box();
    BRepBndLib::add(&vertex, bounds, true)?;
    bounds.set_gap(bounds.gap() + precision::confusion());
    Ok(true)
}

/// Appends a copy of a source vertex with the larger tolerance and links it as same-domain.
fn append_tolerant_copy(
    structure: &mut DataStructure,
    vertex: i32,
    tolerance: f64,
) -> Result<(i32, bool), Failure> {
    let source = Vertex::unchecked(structure.shape(vertex)?.clone());
    let source_tolerance = BRepTool::vertex_tolerance(&source)?;
    let point = BRepTool::pnt(&source)?;
    let mut copy = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut copy, point, source_tolerance.max(tolerance))?;
    let mut info = ShapeInfo::default();
    info.set_shape_type(ShapeEnum::Vertex);
    info.set_shape(&copy.clone().into_shape());
    let index = structure.append_shape_info(&info);
    let bounds = structure.change_shape_info(index)?.change_box();
    BRepBndLib::add(&copy, bounds, true)?;
    bounds.set_gap(bounds.gap() + precision::confusion());
    structure.add_shape_sd(vertex, index);
    Ok((index, source_tolerance < tolerance))
}

/// The common blocks of the pool in first-encounter order (the native fence map only dedups).
fn distinct_common_blocks(structure: &DataStructure) -> Vec<CommonBlockHandle> {
    let mut found: Vec<CommonBlockHandle> = Vec::new();
    for list in structure.pave_blocks_pool() {
        for block in list {
            let Some(common) = structure.common_block(block) else {
                continue;
            };
            if !found.iter().any(|known| Arc::ptr_eq(known, &common)) {
                found.push(common);
            }
        }
    }
    found
}

fn collect_sd_updates<T: std::ops::Deref<Target = Interference>>(
    kind: usize,
    records: &DynamicArray<T>,
    structure: &DataStructure,
    same_domain: &dyn Fn(i32, &DataStructure) -> Option<i32>,
    updates: &mut Vec<(usize, usize, i32)>,
) {
    for (position, record) in records.iter().enumerate() {
        let Some(index) = record.get_index_new() else {
            continue;
        };
        if let Some(representative) = same_domain(index, structure) {
            updates.push((kind, position, representative));
        }
    }
}

fn apply_sd_update(structure: &mut DataStructure, kind: usize, position: usize, index: i32) {
    let record: &mut Interference = match kind {
        0 => structure
            .interf_vv()
            .iter_mut()
            .nth(position)
            .map(DerefMut::deref_mut),
        1 => structure
            .interf_ve()
            .iter_mut()
            .nth(position)
            .map(DerefMut::deref_mut),
        2 => structure
            .interf_vf()
            .iter_mut()
            .nth(position)
            .map(DerefMut::deref_mut),
        3 => structure
            .interf_ee()
            .iter_mut()
            .nth(position)
            .map(DerefMut::deref_mut),
        _ => structure
            .interf_ef()
            .iter_mut()
            .nth(position)
            .map(DerefMut::deref_mut),
    }
    .expect("collected interference position exists");
    record.set_index_new(index);
}
