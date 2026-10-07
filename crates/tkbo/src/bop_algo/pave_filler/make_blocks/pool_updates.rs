//! `BOPAlgo_PaveFiller_6.cxx` pool-wide updates: UpdatePaveBlocks(new SD), RemovePaveBlocks,
//! CorrectToleranceOfSE, PutSEInOtherFaces, UpdateBlocksWithSharedVertices, RemoveMicroEdges.
use super::super::{BOPAlgoPaveFiller, missing_structure};
use crate::bop_ds::{Pave, PaveBlockHandle, PaveBlockHandleIdentity, PaveBlockIndexMap};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

pub(super) type BlockFence = Map<PaveBlockHandle, PaveBlockHandleIdentity>;

impl BOPAlgoPaveFiller {
    /// Substitutes the new same-domain vertices in all section and pool blocks, splitting
    /// their edges anew; blocks that became micro edges are removed (`UpdatePaveBlocks`).
    pub(super) fn update_pave_blocks_sd(
        &mut self,
        new_sd: &DataMap<i32, i32, DefaultHashPolicy>,
    ) -> Result<(), Failure> {
        if new_sd.is_empty() {
            return Ok(());
        }
        let all = self.section_and_pool_blocks()?;
        let mut fence: BlockFence = Map::with_hasher(PaveBlockHandleIdentity, 1);
        let mut micro_edges: Map<i32> = Map::new(1);
        for block in all {
            let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
            let common = structure.common_block(&block);
            let block = match &common {
                Some(common) => {
                    Arc::clone(common.lock().expect("common-block lock").pave_block1()?)
                }
                None => block,
            };
            if !fence.add(Arc::clone(&block)) {
                continue;
            }
            self.rebuild_block_with_sd(&block, common.as_ref(), new_sd, &mut micro_edges)?;
        }
        if micro_edges.extent() > 0 {
            self.remove_pave_blocks(&micro_edges)?;
        }
        Ok(())
    }

    pub(super) fn rebuild_block_with_sd(
        &mut self,
        block: &PaveBlockHandle,
        common: Option<&crate::bop_ds::CommonBlockHandle>,
        new_sd: &DataMap<i32, i32, DefaultHashPolicy>,
        micro_edges: &mut Map<i32>,
    ) -> Result<(), Failure> {
        let (mut vertices, range) = {
            let locked = block.lock().expect("pave-block lock");
            let (first, last) = locked.indices();
            ([first, last], locked.range())
        };
        let parameters = [range.0, range.1];
        // Whether the edge had different vertices before the substitution.
        let was_regular = vertices[0] != vertices[1];
        let mut rebuild = false;
        for end in 0..2 {
            let Some(target) = new_sd.seek(&vertices[end]) else {
                continue;
            };
            vertices[end] = *target;
            rebuild = true;
            let pave = Pave::new(vertices[end], parameters[end]);
            let mut locked = block.lock().expect("pave-block lock");
            if end == 0 {
                locked.set_pave1(pave)
            } else {
                locked.set_pave2(pave)
            }
        }
        if !rebuild {
            return Ok(());
        }
        let mut edge = block.lock().expect("pave-block lock").edge();
        if edge < 0 {
            edge = block.lock().expect("pave-block lock").original_edge();
        }
        let degenerated = self.structure_mut()?.shape_info(edge)?.has_flag();
        if was_regular && !degenerated && vertices[0] == vertices[1] {
            // The edge now starts and ends at one vertex: a closed curve or a micro edge.
            self.fill_shrunk_data_block(block)?;
            if !block.lock().expect("pave-block lock").has_shrunk_data() {
                micro_edges.add(edge);
                return Ok(());
            }
        }
        let split =
            self.split_edge(edge, vertices[0], parameters[0], vertices[1], parameters[1])?;
        match common {
            Some(common) => common.lock().expect("common-block lock").set_edge(split),
            None => block.lock().expect("pave-block lock").set_edge(split),
        }
        Ok(())
    }

    pub(super) fn section_and_pool_blocks(&self) -> Result<Vec<PaveBlockHandle>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut all = Vec::new();
        for record in structure.interf_ff_records().iter() {
            for curve in record.curves().iter() {
                all.extend(curve.pave_blocks().iter().cloned());
            }
        }
        for list in structure.pave_blocks_pool() {
            all.extend(list.iter().cloned());
        }
        Ok(all)
    }

    /// Removes every pave block whose edge is one of `edges` from the pool, the section
    /// curves and the face records (`RemovePaveBlocks`).
    pub(super) fn remove_pave_blocks(&mut self, edges: &Map<i32>) -> Result<(), Failure> {
        let structure = self.structure_mut()?;
        let on_edge = |block: &PaveBlockHandle| {
            edges.contains(&block.lock().expect("pave-block lock").edge())
        };
        for list in structure.change_pave_blocks_pool().iter_mut() {
            list.retain(|block| !on_edge(block));
        }
        for record in structure.interf_ff().iter_mut() {
            for curve in record.change_curves().iter_mut() {
                curve.change_pave_blocks().retain(|block| !on_edge(block));
            }
        }
        for face in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(face)?;
            if info.shape_type() != ShapeEnum::Face || !info.has_reference() {
                continue;
            }
            let face_info = structure.change_face_info(face)?;
            filter_block_map(face_info.change_pave_blocks_in(), &on_edge);
            filter_block_map(face_info.change_pave_blocks_on(), &on_edge);
            filter_block_map(face_info.change_pave_blocks_section(), &on_edge);
        }
        Ok(())
    }

    /// Checks every section edge against the faces that did not create it (`PutSEInOtherFaces`).
    pub(super) fn put_se_in_other_faces(&mut self) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut all = PaveBlockIndexMap::default();
        for record in structure.interf_ff_records().iter() {
            for curve in record.curves().iter() {
                for block in curve.pave_blocks() {
                    all.add(Arc::clone(block));
                }
            }
        }
        self.force_interf_ef_blocks(&all, false)
    }

    /// In non-destructive mode, puts the old vertices shared by both faces on the section
    /// curves through same-domain copies (`UpdateBlocksWithSharedVertices`).
    pub fn update_blocks_with_shared_vertices(&mut self) -> Result<(), Failure> {
        if !self.non_destructive {
            return Ok(());
        }
        let count = self
            .data_structure
            .as_ref()
            .ok_or_else(missing_structure)?
            .interf_ff_records()
            .len();
        if count == 0 {
            return Ok(());
        }
        let mut updated: Map<i32> = Map::new(1);
        for ff in 0..count {
            self.put_shared_vertices_on_curves(ff, &mut updated)?;
        }
        self.update_common_blocks_with_sd_vertices()
    }

    pub(super) fn put_shared_vertices_on_curves(
        &mut self,
        ff: usize,
        updated: &mut Map<i32>,
    ) -> Result<(), Failure> {
        let structure = self.structure_mut()?;
        let (faces, curve_count) = {
            let record = &structure.interf_ff_records()[ff];
            (record.indices(), record.curves().len())
        };
        if curve_count == 0 {
            return Ok(());
        }
        for face in [faces.0, faces.1] {
            if updated.add(face) {
                structure.update_face_info_on(face)?;
            }
        }
        let shared = old_shared_vertices(structure, faces)?;
        for curve in 0..curve_count {
            let stored = self.ff_curve((ff, curve))?;
            let tolerance = stored.tolerance().max(stored.tangential_tolerance());
            for vertex in shared.iter().copied() {
                let mut representative = vertex;
                if self
                    .structure_mut()?
                    .has_shape_sd_into(vertex, &mut representative)
                {
                    continue;
                }
                if !self.estimate_pave_on_curve(vertex, (ff, curve), tolerance)? {
                    continue;
                }
                let vertex_tolerance = BRepTool::vertex_tolerance(&self.vertex_shape(vertex)?)?;
                self.update_vertex(vertex, vertex_tolerance)?;
                self.structure_mut()?.init_pave_blocks_for_vertex(vertex)?;
            }
        }
        Ok(())
    }

    /// Removes the pool blocks that join one vertex without a valid range (`RemoveMicroEdges`).
    pub fn remove_micro_edges(&mut self) -> Result<(), Failure> {
        let mut fence: BlockFence = Map::with_hasher(PaveBlockHandleIdentity, 1);
        let mut micro_edges: Map<i32> = Map::new(1);
        let lists: Vec<Vec<PaveBlockHandle>> = self
            .data_structure
            .as_ref()
            .ok_or_else(missing_structure)?
            .pave_blocks_pool()
            .to_vec();
        for list in lists {
            if list.len() < 2 {
                continue;
            }
            let original = list[0].lock().expect("pave-block lock").original_edge();
            if self.structure_mut()?.shape_info(original)?.has_flag() {
                continue;
            }
            for block in list {
                let real = self.structure_mut()?.real_pave_block(&block)?;
                if !fence.add(Arc::clone(&real)) {
                    continue;
                }
                let (first, last) = real.lock().expect("pave-block lock").indices();
                if first != last {
                    continue;
                }
                self.fill_shrunk_data_block(&real)?;
                if !real.lock().expect("pave-block lock").has_shrunk_data() {
                    micro_edges.add(real.lock().expect("pave-block lock").edge());
                }
            }
        }
        self.remove_pave_blocks(&micro_edges)
    }

    /// Makes a split edge of `edge` between the two vertices and appends it (`SplitEdge`).
    pub(super) fn split_edge(
        &mut self,
        edge: i32,
        first: i32,
        t1: f64,
        last: i32,
        t2: f64,
    ) -> Result<i32, Failure> {
        let structure = self.structure_mut()?;
        let edge = Edge::unchecked(
            structure
                .shape(edge)?
                .oriented(rustycad_tkg3d::top_abs::Orientation::Forward),
        );
        let v1 = rustycad_tkbrep::topods::Vertex::unchecked(
            structure
                .shape(first)?
                .oriented(rustycad_tkg3d::top_abs::Orientation::Forward),
        );
        let v2 = rustycad_tkbrep::topods::Vertex::unchecked(
            structure
                .shape(last)?
                .oriented(rustycad_tkg3d::top_abs::Orientation::Reversed),
        );
        let mut split = Edge::default();
        crate::bop_tools::make_split_edge(&edge, &v1, t1, &v2, t2, &mut split)?;
        let mut info = crate::bop_ds::ShapeInfo::default();
        info.set_shape_type(ShapeEnum::Edge);
        info.set_shape(&split.clone().into_shape());
        let bounds = info.change_box();
        rustycad_tktopalgo::brep_bnd_lib::BRepBndLib::add(&split, bounds, true)?;
        bounds.set_gap(bounds.gap() + rustycad_tkernel::precision::confusion());
        Ok(structure.append_shape_info(&info))
    }
}

fn filter_block_map(map: &mut PaveBlockIndexMap, on_edge: &dyn Fn(&PaveBlockHandle) -> bool) {
    if !map.iter().any(on_edge) {
        return;
    }
    let kept: Vec<PaveBlockHandle> = map
        .iter()
        .filter(|block| !on_edge(block))
        .cloned()
        .collect();
    map.clear();
    for block in kept {
        map.add(block);
    }
}

/// The old vertices On/In the first face that are also On/In the second face.
fn old_shared_vertices(
    structure: &mut crate::bop_ds::DataStructure,
    faces: (i32, i32),
) -> Result<Map<i32>, Failure> {
    let mut shared: Map<i32> = Map::new(1);
    structure.change_face_info(faces.0)?;
    structure.change_face_info(faces.1)?;
    let first = structure.face_info(faces.0)?;
    let second = structure.face_info(faces.1)?;
    for set in [first.vertices_on(), first.vertices_in()] {
        for vertex in set.iter().copied() {
            if structure.is_new_shape(vertex) {
                continue;
            }
            if second.vertices_on().contains(vertex) || second.vertices_in().contains(vertex) {
                shared.add(vertex);
            }
        }
    }
    Ok(shared)
}
