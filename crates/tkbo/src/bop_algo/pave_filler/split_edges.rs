//! `BOPAlgo_PaveFiller_7.cxx`: split edges of pave blocks (`MakeSplitEdges`), p-curves of
//! split and section edges (`MakePCurves`) and planar p-curves of the inputs (`Prepare`).
use super::{BOPAlgoPaveFiller, missing_structure};
use crate::bop_algo::tools::compute_tolerance_of_cb;
use crate::bop_ds::{CommonBlockHandle, CommonBlockHandleIdentity, PaveBlockHandle, ShapeInfo};
use crate::bop_tools::make_split_edge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedMap, Map};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

use rustycad_tkg3d::geom::{GeomPlane, GeomRectangularTrimmedSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_lib::pcurve_for_edge_on_plane;
use std::sync::Arc;

/// `BOPAlgo_SplitEdge`: one split of an edge between two paves.
struct SplitEdgeTask {
    edge: Edge,
    vertices: [Vertex; 2],
    parameters: [f64; 2],
    block: PaveBlockHandle,
    common: Option<CommonBlockHandle>,
    split: Edge,
    bounds: BoundingBox3,
    tolerance: f64,
}

impl BOPAlgoPaveFiller {
    /// Makes the split edges of all pave blocks (one per common block), reusing an untouched
    /// edge when possible (`MakeSplitEdges`); `filler.make_split_edges()?`.
    pub fn make_split_edges(&mut self) -> Result<(), Failure> {
        if self
            .data_structure
            .as_ref()
            .ok_or_else(missing_structure)?
            .pave_blocks_pool()
            .is_empty()
        {
            return Ok(());
        }
        self.update_common_blocks_with_sd_vertices()?;
        let mut tasks = Vec::new();
        let mut fence: Map<CommonBlockHandle, CommonBlockHandleIdentity> =
            Map::with_hasher(CommonBlockHandleIdentity, 100);
        let lists: Vec<Vec<PaveBlockHandle>> = self
            .data_structure
            .as_ref()
            .ok_or_else(missing_structure)?
            .pave_blocks_pool()
            .to_vec();
        for list in &lists {
            for block in list {
                if let Some(task) = self.split_edge_task(block, list.len(), &mut fence)? {
                    tasks.push(task);
                }
            }
        }
        for task in &mut tasks {
            self.perform_split_edge_task(task)?;
        }
        for task in tasks {
            self.store_split_edge(task)?;
        }
        Ok(())
    }

    fn split_edge_task(
        &mut self,
        block: &PaveBlockHandle,
        list_length: usize,
        fence: &mut Map<CommonBlockHandle, CommonBlockHandleIdentity>,
    ) -> Result<Option<SplitEdgeTask>, Failure> {
        let structure = self.structure_mut()?;
        let original = block.lock().expect("pave-block lock").original_edge();
        if structure.shape_info(original)?.has_flag() {
            // Degenerated edges are split by ProcessDE.
            return Ok(None);
        }
        let common = structure.common_block(block);
        if let Some(common) = &common
            && !fence.add(Arc::clone(common))
        {
            return Ok(None);
        }
        let (first, last) = block.lock().expect("pave-block lock").indices();
        if !structure.is_new_shape(first)
            && !structure.is_new_shape(last)
            && (!self.non_destructive || common.is_none())
            && self.reuse_untouched_edge(block, common.as_ref(), list_length)?
        {
            return Ok(None);
        }
        let (block, edge) = match &common {
            Some(common) => {
                let real = Arc::clone(common.lock().expect("common-block lock").pave_block1()?);
                let edge = real.lock().expect("pave-block lock").original_edge();
                (real, edge)
            }
            None => (Arc::clone(block), original),
        };
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let ((first, last), range) = {
            let locked = block.lock().expect("pave-block lock");
            (locked.indices(), locked.range())
        };
        Ok(Some(SplitEdgeTask {
            edge: Edge::unchecked(structure.shape(edge)?.oriented(Orientation::Forward)),
            vertices: [
                Vertex::unchecked(structure.shape(first)?.oriented(Orientation::Forward)),
                Vertex::unchecked(structure.shape(last)?.oriented(Orientation::Reversed)),
            ],
            parameters: [range.0, range.1],
            block,
            common,
            split: Edge::default(),
            bounds: BoundingBox3::default(),
            tolerance: 0.,
        }))
    }

    /// No new vertices: a common block reuses a member edge that was not split, a lone block
    /// keeps its own edge. Returns true when no split is made.
    fn reuse_untouched_edge(
        &mut self,
        block: &PaveBlockHandle,
        common: Option<&CommonBlockHandle>,
        list_length: usize,
    ) -> Result<bool, Failure> {
        let Some(common) = common else {
            if list_length == 1 {
                let original = block.lock().expect("pave-block lock").original_edge();
                block.lock().expect("pave-block lock").set_edge(original);
                return Ok(true);
            }
            return Ok(false);
        };
        let members = common
            .lock()
            .expect("common-block lock")
            .pave_blocks()
            .to_vec();
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut found = None;
        for member in members {
            let edge = member.lock().expect("pave-block lock").original_edge();
            if structure.pave_blocks(edge)?.len() == 1 {
                found = Some((member, edge));
                break;
            }
        }
        let Some((member, edge)) = found else {
            return Ok(false);
        };
        {
            let mut locked = common.lock().expect("common-block lock");
            locked.set_real_pave_block(&member);
            locked.set_edge(edge);
        }
        let context = self.context.clone();
        let tolerance = compute_tolerance_of_cb(
            Some(common),
            self.data_structure.as_ref().ok_or_else(missing_structure)?,
            context.get(),
        )?;
        self.update_edge_tolerance(edge, tolerance)?;
        Ok(true)
    }

    /// Native `BOPAlgo_SplitEdge::Perform`.
    fn perform_split_edge_task(&self, task: &mut SplitEdgeTask) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let context = self.context.clone();
        task.tolerance = compute_tolerance_of_cb(task.common.as_ref(), structure, context.get())?;
        make_split_edge(
            &task.edge,
            &task.vertices[0],
            task.parameters[0],
            &task.vertices[1],
            task.parameters[1],
            &mut task.split,
        )?;
        BRepBndLib::add(&task.split, &mut task.bounds, true)?;
        task.bounds
            .set_gap(task.bounds.gap() + precision::confusion());
        Ok(())
    }

    fn store_split_edge(&mut self, task: SplitEdgeTask) -> Result<(), Failure> {
        let (first, last) = task.block.lock().expect("pave-block lock").indices();
        let mut info = ShapeInfo::default();
        info.set_shape_type(ShapeEnum::Edge);
        info.set_shape(&task.split.into_shape());
        info.set_box(&task.bounds);
        info.change_sub_shapes().extend([first, last]);
        let index = self.structure_mut()?.append_shape_info(&info);
        match task.common {
            Some(common) => {
                self.update_edge_tolerance(index, task.tolerance)?;
                common.lock().expect("common-block lock").set_edge(index);
            }
            None => task.block.lock().expect("pave-block lock").set_edge(index),
        }
        Ok(())
    }

    /// Stores the planar p-curves of the input edges on the planar faces met by vertex, edge
    /// or face intersections, unless the operation is non-destructive (`Prepare`).
    pub fn prepare(&mut self) -> Result<(), Failure> {
        if self.non_destructive {
            return Ok(());
        }
        let mut faces: IndexedMap<Shape, ShapeMapHasher> =
            IndexedMap::with_hasher(ShapeMapHasher, 1);
        for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
            for (_, face) in self.iterator_pairs(kind, ShapeEnum::Face)? {
                let shape = self
                    .data_structure
                    .as_ref()
                    .ok_or_else(missing_structure)?
                    .shape(face)?
                    .clone();
                if is_based_on_plane(&Face::unchecked(shape.clone()))? {
                    faces.add(shape);
                }
            }
        }
        let mut updates = Vec::new();
        for face in faces.iter() {
            let face = Face::unchecked(face.clone());
            for edge in topexp::Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let edge = Edge::unchecked(edge);
                let (pcurve, to_update) = pcurve_for_edge_on_plane(&edge, &face)?;
                updates.push((edge, face.clone(), pcurve, to_update));
            }
        }
        for (edge, face, pcurve, to_update) in updates {
            if to_update {
                let tolerance = BRepTool::edge_tolerance(&edge)?;
                BRepBuilder::default()
                    .update_edge_pcurve_on_face(&edge, &pcurve, &face, tolerance)?;
            }
        }
        Ok(())
    }
}

/// Native static `IsBasedOnPlane`: a plane or a rectangular trimmed plane.
fn is_based_on_plane(face: &Face) -> Result<bool, Failure> {
    let (surface, _) = BRepTool::surface_with_location(face)?;
    let trimmed = surface.cast::<GeomRectangularTrimmedSurface>();
    if let Some(trimmed) = trimmed.get() {
        return Ok(trimmed.basis_surface().cast::<GeomPlane>().is_some());
    }
    Ok(surface.cast::<GeomPlane>().is_some())
}
