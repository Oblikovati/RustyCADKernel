//! `BOPAlgo_PaveFiller_3.cxx`: edge/edge intersection (`PerformEE`) and its task class.
use super::{BOPAlgoPaveFiller, missing_structure};
use crate::bop_algo::tools::{PaveBlockLinks, fill_map, pave_block_links, perform_common_blocks};
use crate::bop_ds::{CoupleOfPaveBlocks, InterfEE, PaveBlockHandle};
use crate::bop_tools::make_new_vertex_edges;
use crate::int_tools::{CommonPrt, Range, Tools};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedDataMap, Map};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point3;
use std::collections::HashMap;
use std::sync::Arc;

#[path = "edge_edge/forced.rs"]
mod forced;
#[path = "edge_edge/new_vertices.rs"]
mod new_vertices;
#[path = "edge_edge/task.rs"]
mod task;
pub(super) use task::EdgeEdgeTask;

/// New vertices keyed by shape identity with the pave blocks that produced them.
pub(super) type VertexCouples = IndexedDataMap<Shape, CoupleOfPaveBlocks, ShapeMapHasher>;

/// A pave block's parameter range and the box used for its intersections.
pub(super) type RangeAndBox = ((f64, f64), BoundingBox3);

/// Cache of computed pave-block boxes (`BOPAlgo_DataMapOfPaveBlockBndBox`).
pub(super) type PaveBlockBoxes = HashMap<usize, BoundingBox3>;

impl BOPAlgoPaveFiller {
    /// Intersects the pave blocks of all edge pairs with interfering boxes, making new vertices
    /// at crossing points and common blocks of coinciding parts (`PerformEE`).
    /// `filler.perform_ee()?`.
    pub fn perform_ee(&mut self) -> Result<(), Failure> {
        self.fill_shrunk_data(ShapeEnum::Edge, ShapeEnum::Edge)?;
        let pairs = self.iterator_pairs(ShapeEnum::Edge, ShapeEnum::Edge)?;
        if pairs.is_empty() {
            return Ok(());
        }
        let mut boxes = PaveBlockBoxes::new();
        let mut tasks = Vec::new();
        for (first, second) in pairs {
            self.append_edge_edge_tasks(first, second, &mut boxes, &mut tasks)?;
        }
        for task in &mut tasks {
            task.perform();
        }
        let mut links = pave_block_links();
        let mut couples = VertexCouples::with_hasher(ShapeMapHasher, 100);
        let mut edges: Map<i32> = Map::new(1);
        for task in &tasks {
            self.analyze_edge_edge_task(task, &mut links, &mut couples, &mut edges)?;
        }
        let context = self.context.clone();
        perform_common_blocks(&links, self.structure_mut()?, context.get())?;
        self.update_vertices_of_cb()?;
        self.perform_new_vertices(&mut couples, true)?;
        if edges.extent() == 0 {
            return Ok(());
        }
        for index in 1..=couples.extent() {
            let couple = couples.find_from_index(index)?;
            for block in [couple.pave_block1(), couple.pave_block2()]
                .into_iter()
                .flatten()
            {
                edges.remove(&block.lock().expect("pave-block lock").original_edge());
            }
        }
        self.split_pave_blocks(&edges, false)
    }

    fn append_edge_edge_tasks(
        &mut self,
        first: i32,
        second: i32,
        boxes: &mut PaveBlockBoxes,
        tasks: &mut Vec<EdgeEdgeTask>,
    ) -> Result<(), Failure> {
        let fuzzy = self.options.fuzzy_value();
        let structure = self.structure_mut()?;
        if structure.shape_info(first)?.has_flag() || structure.shape_info(second)?.has_flag() {
            return Ok(());
        }
        let blocks1 = structure.change_pave_blocks(first)?.clone();
        if blocks1.is_empty() {
            return Ok(());
        }
        let blocks2 = structure.change_pave_blocks(second)?.clone();
        if blocks2.is_empty() {
            return Ok(());
        }
        let edge1 = Edge::unchecked(structure.shape(first)?.clone());
        let edge2 = Edge::unchecked(structure.shape(second)?.clone());
        for block1 in &blocks1 {
            let Some((range1, box1)) = pave_block_box(&edge1, block1, boxes)? else {
                continue;
            };
            let bounds1 = block1.lock().expect("pave-block lock").indices();
            for block2 in &blocks2 {
                let Some((range2, box2)) = pave_block_box(&edge2, block2, boxes)? else {
                    continue;
                };
                if box1.is_out_box(&box2) {
                    continue;
                }
                let bounds2 = block2.lock().expect("pave-block lock").indices();
                let quick = (bounds1.0 == bounds2.0 && bounds1.1 == bounds2.1)
                    || (bounds1.1 == bounds2.0 && bounds1.0 == bounds2.1);
                let mut task = EdgeEdgeTask::new(
                    [Arc::clone(block1), Arc::clone(block2)],
                    [edge1.clone(), edge2.clone()],
                    [range1, range2],
                );
                task.configure([box1.clone(), box2], fuzzy, quick);
                tasks.push(task);
            }
        }
        Ok(())
    }

    fn analyze_edge_edge_task(
        &mut self,
        task: &EdgeEdgeTask,
        links: &mut PaveBlockLinks,
        couples: &mut VertexCouples,
        edges: &mut Map<i32>,
    ) -> Result<(), Failure> {
        if task.failed() {
            let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
            let first = structure.shape(original_edge(&task.block1))?.clone();
            let second = structure.shape(original_edge(&task.block2))?.clone();
            return self.add_intersection_failed_warning(&first, &second);
        }
        if task.parts.is_empty() {
            return Ok(());
        }
        let blocks = EdgePairBlocks::new(&task.block1, &task.block2);
        let analytical = self.is_line_circle_pair(&blocks)?;
        for part in &task.parts {
            match part.shape_type() {
                ShapeEnum::Vertex => {
                    self.analyze_ee_vertex(part, &blocks, analytical, couples, edges)?;
                }
                ShapeEnum::Edge => {
                    // The native `break` leaves the switch only: the next part is still read.
                    if task.parts.len() > 1 {
                        continue;
                    }
                    let same_bounds = {
                        let first = task.block1.lock().expect("pave-block lock");
                        first.has_same_bounds(&task.block2.lock().expect("pave-block lock"))
                    };
                    if !same_bounds {
                        continue;
                    }
                    self.append_ee_record(blocks.edges, part);
                    fill_map(links, Arc::clone(&task.block1), Arc::clone(&task.block2));
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn is_line_circle_pair(&self, blocks: &EdgePairBlocks) -> Result<bool, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let first = BRepAdaptorCurve::with_edge(&Edge::unchecked(
            structure.shape(blocks.edges.0)?.clone(),
        ))?;
        let second = BRepAdaptorCurve::with_edge(&Edge::unchecked(
            structure.shape(blocks.edges.1)?.clone(),
        ))?;
        let types = (first.get_type()?, second.get_type()?);
        Ok(matches!(
            types,
            (CurveType::Line, CurveType::Circle) | (CurveType::Circle, CurveType::Line)
        ))
    }

    fn analyze_ee_vertex(
        &mut self,
        part: &CommonPrt,
        blocks: &EdgePairBlocks,
        analytical: bool,
        couples: &mut VertexCouples,
        edges: &mut Map<i32>,
    ) -> Result<(), Failure> {
        if !blocks.splittable[0] || !blocks.splittable[1] {
            return Ok(());
        }
        let (mut t1, mut t2) = (0., 0.);
        Tools::vertex_parameters(part, &mut t1, &mut t2)?;
        let tolerance = precision::confusion();
        let common1 = *part.range1();
        let common2 = *part.ranges2().first().ok_or_else(missing_range2)?;
        let ranges = &blocks.side_ranges;
        let mut on_pave = [
            Tools::is_on_pave1(t1, &ranges[0], tolerance)
                || Tools::is_on_pave1(ranges[0].first(), &common1, tolerance),
            Tools::is_on_pave1(t1, &ranges[1], tolerance)
                || Tools::is_on_pave1(ranges[1].last(), &common1, tolerance),
            Tools::is_on_pave1(t2, &ranges[2], tolerance)
                || Tools::is_on_pave1(ranges[2].first(), &common2, tolerance),
            Tools::is_on_pave1(t2, &ranges[3], tolerance)
                || Tools::is_on_pave1(ranges[3].last(), &common2, tolerance),
        ];
        if (on_pave[0] && on_pave[2])
            || (on_pave[0] && on_pave[3])
            || (on_pave[1] && on_pave[2])
            || (on_pave[1] && on_pave[3])
        {
            return Ok(());
        }
        let vertices = blocks.vertices;
        let mut exists = false;
        for (position, flag) in on_pave.iter_mut().enumerate() {
            if *flag {
                let block = if position < 2 {
                    &blocks.block2
                } else {
                    &blocks.block1
                };
                *flag = self.force_interf_ve(vertices[position], block, edges)?;
                exists |= *flag;
            }
        }
        let mut vertex = Vertex::default();
        make_new_vertex_edges(part.edge1(), t1, part.edge2(), t2, &mut vertex)?;
        let point = BRepTool::pnt(&vertex)?;
        if exists && !self.extend_existing_ee_vertices(part, (t1, t2), &on_pave, vertices, point)? {
            return Ok(());
        }
        let mut new_tolerance = BRepTool::vertex_tolerance(&vertex)?;
        if analytical {
            // A Line/Circle hit gets half the common range as tolerance, applied only later.
            let line_first =
                BRepAdaptorCurve::with_edge(part.edge1())?.get_type()? == CurveType::Line;
            let minimum = if line_first {
                (common1.last() - common1.first()) / 2.
            } else {
                (common2.last() - common2.first()) / 2.
            };
            if minimum > new_tolerance {
                new_tolerance = minimum;
            }
        }
        if self.near_shared_ee_vertex(vertices, point, new_tolerance)? {
            return Ok(());
        }
        let index = self.append_ee_record(blocks.edges, part);
        let mut couple = CoupleOfPaveBlocks::new();
        couple.set_pave_blocks(
            Some(Arc::clone(&blocks.block1)),
            Some(Arc::clone(&blocks.block2)),
        );
        couple.set_interference_index(index);
        couple.set_tolerance(new_tolerance);
        couples.add(vertex.into_shape(), couple);
        Ok(())
    }

    /// For an intersection close to existing vertices: false when it is only a touching point,
    /// otherwise grows the existing vertices to the new point and protects them from extension.
    fn extend_existing_ee_vertices(
        &mut self,
        part: &CommonPrt,
        (t1, t2): (f64, f64),
        on_pave: &[bool; 4],
        vertices: [i32; 4],
        new_point: Point3,
    ) -> Result<bool, Failure> {
        let first = BRepAdaptorCurve::with_edge(part.edge1())?.value(t1)?;
        let second = BRepAdaptorCurve::with_edge(part.edge2())?.value(t2)?;
        if first.distance(&second) > precision::intersection() {
            return Ok(false);
        }
        for (position, flag) in on_pave.iter().enumerate() {
            if !*flag {
                continue;
            }
            let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
            let vertex = Vertex::unchecked(structure.shape(vertices[position])?.clone());
            let distance = BRepTool::pnt(&vertex)?.distance(&new_point);
            self.update_vertex(vertices[position], distance)?;
            if !self
                .vertices_to_avoid_extension
                .contains(&vertices[position])
            {
                self.vertices_to_avoid_extension.push(vertices[position]);
            }
        }
        Ok(true)
    }

    /// Native "LXBR" check: a new point near a vertex shared by both pave blocks is dropped.
    fn near_shared_ee_vertex(
        &self,
        vertices: [i32; 4],
        point: Point3,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut first_block: Map<i32> = Map::new(1);
        first_block.add(vertices[0]);
        first_block.add(vertices[1]);
        let shared: Vec<i32> = [vertices[2], vertices[3]]
            .into_iter()
            .filter(|vertex| first_block.contains(vertex))
            .collect();
        for vertex in shared {
            let shape = Vertex::unchecked(structure.shape(vertex)?.clone());
            let vertex_tolerance = BRepTool::vertex_tolerance(&shape)?;
            let distance = point.square_distance(&BRepTool::pnt(&shape)?);
            let limit = 100. * (tolerance + vertex_tolerance) * (tolerance + vertex_tolerance);
            if distance < limit {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn append_ee_record(&mut self, (first, second): (i32, i32), part: &CommonPrt) -> i32 {
        let structure = self
            .data_structure
            .as_mut()
            .expect("PerformEE runs on an initialized DS");
        let record = structure.interf_ee().append(InterfEE::default());
        record.set_indices(first, second);
        record.set_common_part(part);
        let index = structure.interf_ee().len() as i32 - 1;
        structure.add_interference(first, second);
        index
    }
}

/// The two pave blocks of an EE task with their native bookkeeping values.
struct EdgePairBlocks {
    block1: PaveBlockHandle,
    block2: PaveBlockHandle,
    edges: (i32, i32),
    vertices: [i32; 4],
    splittable: [bool; 2],
    /// aR11, aR12, aR21, aR22: the parts of each block outside its shrunk range.
    side_ranges: [Range; 4],
}

impl EdgePairBlocks {
    fn new(block1: &PaveBlockHandle, block2: &PaveBlockHandle) -> Self {
        let (edge1, vertices1, splittable1, ranges1) = block_values(block1);
        let (edge2, vertices2, splittable2, ranges2) = block_values(block2);
        Self {
            block1: Arc::clone(block1),
            block2: Arc::clone(block2),
            edges: (edge1, edge2),
            vertices: [vertices1.0, vertices1.1, vertices2.0, vertices2.1],
            splittable: [splittable1, splittable2],
            side_ranges: [ranges1[0], ranges1[1], ranges2[0], ranges2[1]],
        }
    }
}

/// Edge index, end vertices, splittable flag and the two side ranges of a block; without
/// shrunk data the shrunk range is the whole range and the block is not splittable.
fn block_values(block: &PaveBlockHandle) -> (i32, (i32, i32), bool, [Range; 2]) {
    let locked = block.lock().expect("pave-block lock must not be poisoned");
    let (first, last) = locked.range();
    let (shrunk_first, shrunk_last, splittable) = if locked.has_shrunk_data() {
        let (shrunk_first, shrunk_last, _, splittable) = locked.shrunk_data();
        (shrunk_first, shrunk_last, splittable)
    } else {
        (first, last, false)
    };
    let ranges = [
        Range::new(first, shrunk_first),
        Range::new(shrunk_last, last),
    ];
    (locked.original_edge(), locked.indices(), splittable, ranges)
}

/// Native `GetPBBox`: the block range, and its shrunk box (or a cached curve box); `None`
/// for a block without a valid range.
pub(super) fn pave_block_box(
    edge: &Edge,
    block: &PaveBlockHandle,
    boxes: &mut PaveBlockBoxes,
) -> Result<Option<RangeAndBox>, Failure> {
    let locked = block.lock().expect("pave-block lock must not be poisoned");
    let range = locked.range();
    if range.1 - range.0 <= precision::p_confusion() {
        return Ok(None);
    }
    if locked.has_shrunk_data() {
        return Ok(Some((range, locked.shrunk_data().2.clone())));
    }
    let key = Arc::as_ptr(block) as usize;
    if let Some(bounds) = boxes.get(&key) {
        return Ok(Some((range, bounds.clone())));
    }
    let curve = BRepAdaptorCurve::with_edge(edge)?;
    let tolerance = BRepTool::edge_tolerance(edge)? + precision::confusion();
    let mut bounds = BoundingBox3::default();
    rustycad_tkgeombase::bnd_lib::Add3dCurve::add_range(
        &curve,
        range.0,
        range.1,
        tolerance,
        &mut bounds,
    )?;
    boxes.insert(key, bounds.clone());
    Ok(Some((range, bounds)))
}

fn original_edge(block: &PaveBlockHandle) -> i32 {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .original_edge()
}

fn missing_range2() -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        "IntTools_CommonPrt::Ranges2 is empty; expected one range for an EE vertex",
        "",
    )
}
