use super::BOPAlgoPaveFiller;
use crate::bop_algo::AlertIntersectionOfPairOfShapesFailed;
use crate::bop_ds::{CommonBlock, InterfEE, Pave, PaveBlockHandle, ShapeInfo};
use crate::int_tools::{CommonPrt, EdgeEdge, Range, Tools};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Shape, Vertex},
};
use rustycad_tkernel::{message::AlertHandle, precision, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use std::sync::Arc;

#[path = "edge_edge/candidates.rs"]
mod candidates;

pub(super) struct EdgeBlockPair {
    pub edge1: i32,
    pub edge2: i32,
    pub block1: PaveBlockHandle,
    pub block2: PaveBlockHandle,
    pub range1: Range,
    pub range2: Range,
}

pub(super) struct EdgeVertexPart {
    pub pair: EdgeBlockPair,
    pub common: CommonPrt,
}

impl BOPAlgoPaveFiller {
    /// Intersects candidate edge pave blocks and splits interior vertex crossings.
    /// `filler.perform_ee()?` runs the ordinary edge/edge intersection stage.
    pub fn perform_ee(&mut self) -> Result<(), Failure> {
        self.fill_shrunk_data(ShapeEnum::Edge, ShapeEnum::Edge)?;
        let candidates = candidates::collect(self)?;
        let mut modified_edges = Vec::new();
        let mut vertex_parts = Vec::new();
        for pair in candidates {
            self.intersect_edge_blocks(pair, &mut vertex_parts)?;
        }
        for part in vertex_parts {
            self.append_vertex_part(part, &mut modified_edges)?;
        }
        self.split_intersected_edges(&modified_edges)
    }

    fn intersect_edge_blocks(
        &mut self,
        pair: EdgeBlockPair,
        vertex_parts: &mut Vec<EdgeVertexPart>,
    ) -> Result<(), Failure> {
        let (first, second) = self.edge_shapes(pair.edge1, pair.edge2)?;
        let mut solver = EdgeEdge::with_ranges(
            &first,
            pair.range1.first(),
            pair.range1.last(),
            &second,
            pair.range2.first(),
            pair.range2.last(),
        );
        solver.set_fuzzy_value(self.options.fuzzy_value());
        solver.use_quick_coincidence_check(blocks_share_bounds(&pair.block1, &pair.block2));
        if solver.perform().is_err() || !solver.is_done() {
            return self.warn_edge_edge(&first, &second);
        }
        self.collect_edge_parts(pair, solver.common_parts(), vertex_parts)
    }

    fn edge_shapes(&self, first: i32, second: i32) -> Result<(Edge, Edge), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformEE requires initialized data structure")
        })?;
        Ok((
            Edge::unchecked(structure.shape(first)?.clone()),
            Edge::unchecked(structure.shape(second)?.clone()),
        ))
    }

    fn collect_edge_parts(
        &mut self,
        pair: EdgeBlockPair,
        parts: &[CommonPrt],
        vertex_parts: &mut Vec<EdgeVertexPart>,
    ) -> Result<(), Failure> {
        for common in parts {
            match common.shape_type() {
                ShapeEnum::Vertex => {
                    if blocks_are_splittable(&pair) {
                        vertex_parts.push(EdgeVertexPart {
                            pair: clone_pair(&pair),
                            common: common.clone(),
                        });
                    }
                }
                ShapeEnum::Edge if parts.len() == 1 => {
                    self.bind_edge_part(&pair, common)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn bind_edge_part(
        &mut self,
        pair: &EdgeBlockPair,
        common_part: &CommonPrt,
    ) -> Result<(), Failure> {
        if !blocks_share_bounds(&pair.block1, &pair.block2) {
            return Ok(());
        }
        let mut record = InterfEE::default();
        record.set_indices(pair.edge1, pair.edge2);
        record.set_common_part(common_part);
        self.append_interference(record, pair.edge1, pair.edge2);
        self.bind_common_blocks(pair)?;
        Ok(())
    }

    fn bind_common_blocks(&mut self, pair: &EdgeBlockPair) -> Result<(), Failure> {
        let mut common = CommonBlock::default();
        common.set_pave_blocks(&[pair.block1.clone(), pair.block2.clone()]);
        let common = Arc::new(std::sync::Mutex::new(common));
        let structure = self.data_structure.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformEE requires initialized data structure")
        })?;
        structure.set_common_block(&pair.block1, &common);
        structure.set_common_block(&pair.block2, &common);
        Ok(())
    }

    fn append_vertex_part(
        &mut self,
        event: EdgeVertexPart,
        modified_edges: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        let (first_parameter, second_parameter) = vertex_parameters(&event.common)?;
        if event_hits_existing_pave(&event, first_parameter, second_parameter)? {
            return Ok(());
        }
        let vertex = self.make_intersection_vertex(&event, first_parameter, second_parameter)?;
        self.append_vertex_record(&event, vertex.0);
        append_block_pave(&event.pair.block1, vertex.0, first_parameter);
        append_block_pave(&event.pair.block2, vertex.0, second_parameter);
        append_unique(modified_edges, event.pair.edge1);
        append_unique(modified_edges, event.pair.edge2);
        Ok(())
    }

    fn make_intersection_vertex(
        &mut self,
        event: &EdgeVertexPart,
        first_parameter: f64,
        second_parameter: f64,
    ) -> Result<(i32, Vertex), Failure> {
        let (first, second) = self.edge_shapes(event.pair.edge1, event.pair.edge2)?;
        let mut vertex = Vertex::default();
        crate::bop_tools::make_new_vertex_edges(
            &first,
            first_parameter,
            &second,
            second_parameter,
            &mut vertex,
        )?;
        let index = self.append_new_vertex(&vertex)?;
        Ok((index, vertex))
    }

    fn append_new_vertex(&mut self, vertex: &Vertex) -> Result<i32, Failure> {
        let tolerance = BRepTool::vertex_tolerance(vertex)?;
        let point = BRepTool::pnt(vertex)?;
        let mut info = ShapeInfo::default();
        info.set_shape_type(ShapeEnum::Vertex);
        info.set_shape(&vertex.clone().into_shape());
        let mut bounds = BoundingBox3::default();
        bounds.add_point(point);
        bounds.set_gap(tolerance + self.options.fuzzy_value() / 2.0);
        info.set_box(&bounds);
        let structure = self.data_structure.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformEE requires initialized data structure")
        })?;
        Ok(structure.append_shape_info(&info))
    }

    fn append_vertex_record(&mut self, event: &EdgeVertexPart, vertex_index: i32) {
        let mut record = InterfEE::default();
        record.set_indices(event.pair.edge1, event.pair.edge2);
        record.set_common_part(&event.common);
        record.set_index_new(vertex_index);
        self.append_interference(record, event.pair.edge1, event.pair.edge2);
    }

    fn append_interference(&mut self, record: InterfEE, first: i32, second: i32) {
        let structure = self
            .data_structure
            .as_mut()
            .expect("initialized data structure");
        structure.add_interference(first, second);
        structure.interf_ee().append(record);
    }

    fn warn_edge_edge(&self, first: &Edge, second: &Edge) -> Result<(), Failure> {
        let shape = edge_pair_shape(first, second)?;
        let alert: AlertHandle = Arc::new(AlertIntersectionOfPairOfShapesFailed::new(&shape));
        self.options.add_warning(&alert);
        Ok(())
    }
}

fn vertex_parameters(part: &CommonPrt) -> Result<(f64, f64), Failure> {
    let mut first = part.vertex_parameter1();
    let mut second = part.vertex_parameter2();
    Tools::vertex_parameters(part, &mut first, &mut second)?;
    Ok((first, second))
}

fn blocks_are_splittable(pair: &EdgeBlockPair) -> bool {
    block_splittable(&pair.block1) && block_splittable(&pair.block2)
}

fn block_splittable(block: &PaveBlockHandle) -> bool {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .is_splittable()
}

fn block_range(block: &PaveBlockHandle) -> Range {
    let locked = block.lock().expect("pave-block lock must not be poisoned");
    let (first, last) = locked.range();
    Range::new(first, last)
}

fn shrunk_range(block: &PaveBlockHandle) -> Range {
    let locked = block.lock().expect("pave-block lock must not be poisoned");
    let (first, last, _, _) = locked.shrunk_data();
    Range::new(first, last)
}

fn event_hits_existing_pave(
    event: &EdgeVertexPart,
    first: f64,
    second: f64,
) -> Result<bool, Failure> {
    let range1 = block_range(&event.pair.block1);
    let range2 = block_range(&event.pair.block2);
    let shrunk1 = shrunk_range(&event.pair.block1);
    let shrunk2 = shrunk_range(&event.pair.block2);
    let tolerance = precision::confusion();
    let second_range = event.common.ranges2().first().ok_or_else(|| {
        Failure::new("BOPAlgo_PaveFiller::PerformEE vertex common part requires range2[0]")
    })?;
    let on_pave = [
        Tools::is_on_pave1(
            first,
            &Range::new(range1.first(), shrunk1.first()),
            tolerance,
        ) || Tools::is_on_pave1(shrunk1.first(), event.common.range1(), tolerance),
        Tools::is_on_pave1(first, &Range::new(shrunk1.last(), range1.last()), tolerance)
            || Tools::is_on_pave1(shrunk1.last(), event.common.range1(), tolerance),
        Tools::is_on_pave1(
            second,
            &Range::new(range2.first(), shrunk2.first()),
            tolerance,
        ) || Tools::is_on_pave1(shrunk2.first(), second_range, tolerance),
        Tools::is_on_pave1(
            second,
            &Range::new(shrunk2.last(), range2.last()),
            tolerance,
        ) || Tools::is_on_pave1(shrunk2.last(), second_range, tolerance),
    ];
    Ok(on_pave.iter().any(|is_on_pave| *is_on_pave))
}

fn append_block_pave(block: &PaveBlockHandle, vertex: i32, parameter: f64) {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .append_ext_pave(Pave::new(vertex, parameter));
}

fn append_unique(indices: &mut Vec<i32>, value: i32) {
    if !indices.contains(&value) {
        indices.push(value);
    }
}

fn edge_pair_shape(first: &Edge, second: &Edge) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = rustycad_tkbrep::topods::Compound::default();
    builder.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    builder.add(&mut shape, &first.clone().into_shape())?;
    builder.add(&mut shape, &second.clone().into_shape())?;
    Ok(shape)
}

fn blocks_share_bounds(first: &PaveBlockHandle, second: &PaveBlockHandle) -> bool {
    let first = first.lock().expect("pave-block lock must not be poisoned");
    let second = second.lock().expect("pave-block lock must not be poisoned");
    first.has_same_bounds(&second)
}

fn clone_pair(pair: &EdgeBlockPair) -> EdgeBlockPair {
    EdgeBlockPair {
        edge1: pair.edge1,
        edge2: pair.edge2,
        block1: pair.block1.clone(),
        block2: pair.block2.clone(),
        range1: pair.range1,
        range2: pair.range2,
    }
}
