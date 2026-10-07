//! `BOPAlgo_PaveFiller_6.cxx`: UpdateFaceInfo, UpdateExistingPaveBlocks, UpdatePaveBlocks and
//! RemovePaveBlocks, which carry the post treatment results into the face records and pool.
use super::super::{BOPAlgoPaveFiller, missing_structure};
use super::{BlockFaces, ExistingEdges};
use crate::bop_ds::{
    CommonBlock, CommonBlockHandle, Pave, PaveBlock, PaveBlockHandle, PaveBlockHandleIdentity,
    PaveBlockIndexMap, common_block_handle, pave_block_handle,
};
use crate::int_tools::EdgeFace;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

type EdgeBlocks = DataMap<i32, Vec<PaveBlockHandle>, DefaultHashPolicy>;

impl BOPAlgoPaveFiller {
    /// Adds the section edges and vertices to the face records, unifies the existing edges
    /// united by the post treatment into common blocks, then substitutes the new vertices and
    /// blocks in the face records of the intersected faces (`UpdateFaceInfo`).
    pub(super) fn update_face_info(
        &mut self,
        existing: &mut ExistingEdges,
        new_sd: &DataMap<i32, i32, DefaultHashPolicy>,
        block_faces: &BlockFaces,
    ) -> Result<(), Failure> {
        let mut faces: Map<i32> = Map::new(1);
        let mut edge_blocks: EdgeBlocks = DataMap::default();
        let count = self
            .data_structure
            .as_ref()
            .ok_or_else(missing_structure)?
            .interf_ff_records()
            .len();
        for ff in 0..count {
            self.add_sections_to_faces(ff, existing, block_faces, &mut edge_blocks, &mut faces)?;
        }
        let new_common = self.unify_united_edges(&edge_blocks)?;
        let vertices = !new_sd.is_empty();
        let edges = !existing.is_empty() || new_common;
        if !vertices && !edges {
            return Ok(());
        }
        for face in faces.iter().copied() {
            if vertices {
                self.substitute_face_vertices(face, new_sd)?;
            }
            if edges {
                self.substitute_face_blocks(face, existing)?;
            }
        }
        Ok(())
    }

    fn add_sections_to_faces(
        &mut self,
        ff: usize,
        existing: &mut ExistingEdges,
        block_faces: &BlockFaces,
        edge_blocks: &mut EdgeBlocks,
        faces: &mut Map<i32>,
    ) -> Result<(), Failure> {
        let (pair, curve_count) = {
            let record = &self
                .data_structure
                .as_ref()
                .ok_or_else(missing_structure)?
                .interf_ff_records()[ff];
            (record.indices(), record.curves().len())
        };
        let structure = self.structure_mut()?;
        structure.change_face_info(pair.0)?;
        structure.change_face_info(pair.1)?;
        for curve in 0..curve_count {
            let blocks = self.ff_curve((ff, curve))?.pave_blocks().to_vec();
            let mut kept = Vec::with_capacity(blocks.len());
            for block in blocks {
                if let Some(mut replacements) = existing.seek(&block).cloned() {
                    self.update_existing_pave_blocks(&block, &mut replacements, block_faces)?;
                    for replacement in &replacements {
                        append_edge_block(edge_blocks, replacement);
                    }
                    *existing.change_find(&block)? = replacements;
                    continue;
                }
                let structure = self.structure_mut()?;
                structure
                    .change_face_info(pair.0)?
                    .change_pave_blocks_section()
                    .add(Arc::clone(&block));
                structure
                    .change_face_info(pair.1)?
                    .change_pave_blocks_section()
                    .add(Arc::clone(&block));
                append_edge_block(edge_blocks, &block);
                kept.push(block);
            }
            *self.ff_curve_mut((ff, curve))?.change_pave_blocks() = kept;
        }
        let structure = self.structure_mut()?;
        let points: Vec<i32> = structure.interf_ff_records()[ff]
            .points()
            .iter()
            .map(|point| point.index())
            .collect();
        for vertex in points {
            if vertex < 0 {
                continue;
            }
            structure
                .change_face_info(pair.0)?
                .change_vertices_section()
                .add(vertex);
            structure
                .change_face_info(pair.1)?
                .change_vertices_section()
                .add(vertex);
        }
        faces.add(pair.0);
        faces.add(pair.1);
        Ok(())
    }

    /// Makes common blocks of the blocks sharing an edge after the post treatment; returns
    /// true when one was made.
    fn unify_united_edges(&mut self, edge_blocks: &EdgeBlocks) -> Result<bool, Failure> {
        let mut made = false;
        let lists: Vec<Vec<PaveBlockHandle>> = edge_blocks.values().cloned().collect();
        for list in lists {
            if list.len() == 1 {
                continue;
            }
            made = true;
            self.unify_blocks_on_edge(&list)?;
        }
        Ok(made)
    }

    fn unify_blocks_on_edge(&mut self, list: &[PaveBlockHandle]) -> Result<(), Failure> {
        let structure = self.structure_mut()?;
        let mut common: Option<CommonBlockHandle> = None;
        let mut faces: Map<i32> = Map::new(1);
        let mut members = PaveBlockIndexMap::default();
        for block in list {
            members.add(Arc::clone(block));
            let Some(existing) = structure.common_block(block) else {
                continue;
            };
            let (blocks, block_faces) = {
                let locked = existing.lock().expect("common-block lock");
                (locked.pave_blocks().to_vec(), locked.faces().to_vec())
            };
            for member in blocks {
                members.add(member);
            }
            for face in block_faces {
                faces.add(face);
            }
            common.get_or_insert(existing);
        }
        let Some(common) = common else {
            let mut created = CommonBlock::new();
            created.set_pave_blocks(list);
            let created = common_block_handle(created);
            for block in list {
                structure.set_common_block(block, &created);
            }
            return Ok(());
        };
        let blocks: Vec<PaveBlockHandle> = members.iter().cloned().collect();
        for block in &blocks {
            structure.set_common_block(block, &common);
        }
        let face_list: Vec<i32> = faces.iter().copied().collect();
        let mut locked = common.lock().expect("common-block lock");
        locked.set_pave_blocks(&blocks);
        locked.set_faces(&face_list);
        Ok(())
    }

    fn substitute_face_vertices(
        &mut self,
        face: i32,
        new_sd: &DataMap<i32, i32, DefaultHashPolicy>,
    ) -> Result<(), Failure> {
        let info = self.structure_mut()?.change_face_info(face)?;
        for (old, new) in new_sd.items() {
            if info.change_vertices_on().remove(*old) {
                info.change_vertices_on().add(*new);
            }
            if info.change_vertices_in().remove(*old) {
                info.change_vertices_in().add(*new);
            }
        }
        Ok(())
    }

    fn substitute_face_blocks(
        &mut self,
        face: i32,
        existing: &ExistingEdges,
    ) -> Result<(), Failure> {
        let mut fence: Map<PaveBlockHandle, PaveBlockHandleIdentity> =
            Map::with_hasher(PaveBlockHandleIdentity, 1);
        for kind in 0..3 {
            let structure = self.structure_mut()?;
            let current: Vec<PaveBlockHandle> =
                face_blocks(structure.change_face_info(face)?, kind)
                    .iter()
                    .cloned()
                    .collect();
            let mut replaced = Vec::with_capacity(current.len());
            for block in current {
                let images = match existing.seek(&block) {
                    Some(list) if !list.is_empty() => list.clone(),
                    _ => vec![block],
                };
                for image in images {
                    let real = structure.real_pave_block(&image)?;
                    if fence.add(Arc::clone(&real)) {
                        replaced.push(real);
                    }
                }
            }
            let target = face_blocks_mut(structure.change_face_info(face)?, kind);
            target.clear();
            for block in replaced {
                target.add(block);
            }
        }
        Ok(())
    }

    /// Replaces an existing (possibly common) pave block by the post treatment splits in the
    /// pool, then tries to put the splits on the faces they must touch
    /// (`UpdateExistingPaveBlocks`).
    pub(super) fn update_existing_pave_blocks(
        &mut self,
        original: &PaveBlockHandle,
        splits: &mut Vec<PaveBlockHandle>,
        block_faces: &BlockFaces,
    ) -> Result<(), Failure> {
        if splits.is_empty() {
            return Ok(());
        }
        let common = self.structure_mut()?.common_block(original);
        let olds: Vec<PaveBlockHandle> = match &common {
            Some(common) => common
                .lock()
                .expect("common-block lock")
                .pave_blocks()
                .to_vec(),
            None => vec![Arc::clone(original)],
        };
        for old in &olds {
            let edge = old.lock().expect("pave-block lock").original_edge();
            let list = self.structure_mut()?.change_pave_blocks(edge)?;
            if let Some(position) = list.iter().position(|item| Arc::ptr_eq(item, old)) {
                list.remove(position);
            }
        }
        if let Some(common) = common {
            *splits = self.split_common_blocks(&common, &olds, splits)?;
        } else {
            let edge = original.lock().expect("pave-block lock").original_edge();
            let list = self.structure_mut()?.change_pave_blocks(edge)?;
            list.extend(splits.iter().cloned());
        }
        let Some(faces) = block_faces.seek(original) else {
            return Ok(());
        };
        for face in faces.clone() {
            self.project_splits_on_face(face, splits)?;
        }
        Ok(())
    }

    /// New common blocks per split, with one pave block on each original edge of the old
    /// common block; returns the first block of each.
    fn split_common_blocks(
        &mut self,
        common: &CommonBlockHandle,
        olds: &[PaveBlockHandle],
        splits: &[PaveBlockHandle],
    ) -> Result<Vec<PaveBlockHandle>, Failure> {
        let faces = common.lock().expect("common-block lock").faces().to_vec();
        let mut firsts = Vec::with_capacity(splits.len());
        for split in splits {
            let (paves, split_edge, split_original) = {
                let locked = split.lock().expect("pave-block lock");
                (
                    [locked.pave1(), locked.pave2()],
                    locked.edge(),
                    locked.original_edge(),
                )
            };
            let mut created = CommonBlock::new();
            let created_handle = common_block_handle(CommonBlock::new());
            for old in olds {
                let edge = old.lock().expect("pave-block lock").original_edge();
                let (first, last) = if split_original == edge {
                    (paves[0], paves[1])
                } else {
                    self.paves_on_other_edge(paves, old, edge)?
                };
                let mut block = PaveBlock::default();
                block.set_pave1(first);
                block.set_pave2(last);
                block.set_edge(split_edge);
                block.set_original_edge(edge);
                let block = pave_block_handle(block);
                created.add_pave_block(Arc::clone(&block));
                let structure = self.structure_mut()?;
                structure.set_common_block(&block, &created_handle);
                structure.change_pave_blocks(edge)?.push(block);
            }
            created.set_faces(&faces);
            let first = Arc::clone(created.pave_block1()?);
            *created_handle.lock().expect("common-block lock") = created;
            firsts.push(first);
        }
        Ok(firsts)
    }

    /// The split's paves expressed on another original edge of the common block.
    fn paves_on_other_edge(
        &self,
        paves: [Pave; 2],
        old: &PaveBlockHandle,
        edge: i32,
    ) -> Result<(Pave, Pave), Failure> {
        let (old1, old2) = {
            let locked = old.lock().expect("pave-block lock");
            (locked.pave1(), locked.pave2())
        };
        let mut result = [Pave::default(); 2];
        if paves[0].index() == paves[1].index() && old1.index() == old2.index() {
            // Still closed.
            result[0] = Pave::new(paves[0].index(), old1.parameter());
            result[1] = Pave::new(paves[1].index(), old2.parameter());
        } else {
            for position in 0..2 {
                let vertex = paves[position].index();
                let parameter = if vertex == old1.index() {
                    old1.parameter()
                } else if vertex == old2.index() {
                    old2.parameter()
                } else {
                    self.project_vertex_parameter(vertex, edge, old1, old2)?
                };
                result[position] = Pave::new(vertex, parameter);
            }
        }
        if result[1].parameter() < result[0].parameter() {
            result.swap(0, 1);
        }
        Ok((result[0], result[1]))
    }

    /// The vertex parameter on the edge, or the parameter of the closest old bound.
    fn project_vertex_parameter(
        &self,
        vertex: i32,
        edge: i32,
        old1: Pave,
        old2: Pave,
    ) -> Result<f64, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let shape = Vertex::unchecked(structure.shape(vertex)?.clone());
        let edge_shape = Edge::unchecked(structure.shape(edge)?.clone());
        let (mut parameter, mut distance) = (0., 0.);
        let fuzzy = self.options.fuzzy_value();
        if self.context_ref()?.compute_ve(
            &shape,
            &edge_shape,
            &mut parameter,
            &mut distance,
            fuzzy,
        )? == 0
        {
            return Ok(parameter);
        }
        let point = BRepTool::pnt(&shape)?;
        let first = BRepTool::pnt(&Vertex::unchecked(structure.shape(old1.index())?.clone()))?;
        let last = BRepTool::pnt(&Vertex::unchecked(structure.shape(old2.index())?.clone()))?;
        Ok(
            if point.square_distance(&first) < point.square_distance(&last) {
                old1.parameter()
            } else {
                old2.parameter()
            },
        )
    }

    /// Puts the splits on the face as In blocks when an edge/face intersection finds them
    /// coincident with it.
    fn project_splits_on_face(
        &mut self,
        face: i32,
        splits: &[PaveBlockHandle],
    ) -> Result<(), Failure> {
        let face_shape = Face::unchecked(self.structure_mut()?.shape(face)?.clone());
        for split in splits {
            let structure = self.structure_mut()?;
            let info = structure.change_face_info(face)?;
            if info.pave_blocks_on().contains(split) || info.pave_blocks_in().contains(split) {
                continue;
            }
            let (edge, range) = {
                let locked = split.lock().expect("pave-block lock");
                (
                    locked.edge(),
                    (locked.pave1().parameter(), locked.pave2().parameter()),
                )
            };
            let edge_shape = Edge::unchecked(structure.shape(edge)?.clone());
            let mut intersector = EdgeFace::new();
            intersector.set_edge(&edge_shape);
            intersector.set_face(&face_shape);
            intersector.set_fuzzy_value(self.options.fuzzy_value());
            intersector.set_range_bounds(range.0, range.1);
            intersector.set_context(&self.context);
            intersector.perform()?;
            let parts = intersector.common_parts();
            if parts.len() != 1
                || parts.iter().next().map(|part| part.shape_type()) != Some(ShapeEnum::Edge)
            {
                continue;
            }
            let structure = self.structure_mut()?;
            let common = structure.common_block(split).unwrap_or_else(|| {
                let mut created = CommonBlock::new();
                created.add_pave_block(Arc::clone(split));
                common_block_handle(created)
            });
            structure.set_common_block(split, &common);
            common.lock().expect("common-block lock").add_face(face);
            structure
                .change_face_info(face)?
                .change_pave_blocks_in()
                .add(Arc::clone(split));
        }
        Ok(())
    }
}

fn append_edge_block(edge_blocks: &mut EdgeBlocks, block: &PaveBlockHandle) {
    let edge = block.lock().expect("pave-block lock").edge();
    if let Some(list) = edge_blocks.change_seek(&edge) {
        list.push(Arc::clone(block));
        return;
    }
    edge_blocks.bind(edge, vec![Arc::clone(block)]);
}

fn face_blocks(info: &crate::bop_ds::FaceInfo, kind: usize) -> &PaveBlockIndexMap {
    match kind {
        0 => info.pave_blocks_on(),
        1 => info.pave_blocks_in(),
        _ => info.pave_blocks_section(),
    }
}

fn face_blocks_mut(info: &mut crate::bop_ds::FaceInfo, kind: usize) -> &mut PaveBlockIndexMap {
    match kind {
        0 => info.change_pave_blocks_on(),
        1 => info.change_pave_blocks_in(),
        _ => info.change_pave_blocks_section(),
    }
}
