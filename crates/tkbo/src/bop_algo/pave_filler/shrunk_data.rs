//! `BOPAlgo_PaveFiller_9.cxx` FillShrunkData and the `_3.cxx` single-block FillShrunkData and
//! AnalyzeShrunkData: working (shrunk) ranges of pave blocks.
use super::{BOPAlgoPaveFiller, missing_structure};
use crate::bop_algo::{AlertBadPositioning, AlertNotSplittableEdge, AlertTooSmallEdge};
use crate::bop_ds::PaveBlockHandle;
use crate::int_tools::ShrunkRange;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Compound, Edge, Shape, Vertex},
};
use rustycad_tkernel::{message::AlertHandle, ncollection::Map, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    /// Computes shrunk ranges of all pave blocks (without valid ones) of the edges met in
    /// the iterator pairs of the two types; `filler.fill_shrunk_data(Vertex, Edge)?`.
    pub fn fill_shrunk_data(
        &mut self,
        first_type: ShapeEnum,
        second_type: ShapeEnum,
    ) -> Result<(), Failure> {
        let pairs = self.iterator_pairs(first_type, second_type)?;
        let mut visited: Map<i32> = Map::new(1);
        let mut computed: Vec<(PaveBlockHandle, ShrunkRange)> = Vec::new();
        for (first, second) in pairs {
            for (kind, edge) in [(first_type, first), (second_type, second)] {
                if kind != ShapeEnum::Edge || !visited.add(edge) {
                    continue;
                }
                self.prepare_edge_ranges(edge, &mut computed)?;
            }
        }
        for (_, range) in &mut computed {
            range.perform()?;
        }
        for (block, range) in &computed {
            self.analyze_shrunk_data(block, range)?;
        }
        Ok(())
    }

    /// Computes the shrunk range of one pave block on its split edge, or else on its original
    /// edge; `filler.fill_shrunk_data_block(&block)?`.
    pub fn fill_shrunk_data_block(&mut self, block: &PaveBlockHandle) -> Result<(), Failure> {
        let (first, last, edge, range) = {
            let locked = block.lock().expect("pave-block lock must not be poisoned");
            let (first, last) = locked.indices();
            let edge = if locked.has_edge() {
                locked.edge()
            } else {
                locked.original_edge()
            };
            (first, last, edge, locked.range())
        };
        if first < 0 || last < 0 || edge < 0 {
            return Ok(());
        }
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let edge = Edge::unchecked(structure.shape(edge)?.clone());
        let mut shrunk = ShrunkRange::new();
        shrunk.set_context(&self.context);
        shrunk.set_data(
            &edge,
            range.0,
            range.1,
            &Vertex::unchecked(structure.shape(first)?.clone()),
            &Vertex::unchecked(structure.shape(last)?.clone()),
        );
        shrunk.perform()?;
        self.analyze_shrunk_data(block, &shrunk)
    }

    /// Stores the shrunk range on the pave block, warning about too small, unsplittable or
    /// badly positioned blocks (`AnalyzeShrunkData`); `filler.analyze_shrunk_data(&pb, &sr)?`.
    pub fn analyze_shrunk_data(
        &mut self,
        block: &PaveBlockHandle,
        shrunk: &ShrunkRange,
    ) -> Result<(), Failure> {
        let (first, last) = shrunk.shrunk_range();
        if !shrunk.is_done() || !shrunk.is_splittable() {
            let (whole, warning) = self.shrunk_warning_shape(block, shrunk.edge())?;
            let alert: AlertHandle = match (shrunk.is_done(), whole) {
                (false, true) => Arc::new(AlertTooSmallEdge::new(&warning)),
                (true, true) => Arc::new(AlertNotSplittableEdge::new(&warning)),
                _ => Arc::new(AlertBadPositioning::new(&warning)),
            };
            self.options.add_warning(&alert);
            if !shrunk.is_done() {
                set_shrunk(block, first, last, &BoundingBox3::default(), false);
                return Ok(());
            }
        }
        let mut bounds = shrunk.bounding_box().clone();
        bounds.set_gap(bounds.gap() + self.options.fuzzy_value() / 2.);
        set_shrunk(block, first, last, &bounds, shrunk.is_splittable());
        Ok(())
    }

    fn prepare_edge_ranges(
        &mut self,
        edge: i32,
        computed: &mut Vec<(PaveBlockHandle, ShrunkRange)>,
    ) -> Result<(), Failure> {
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        if structure.shape_info(edge)?.has_flag() {
            return Ok(());
        }
        let shape = Edge::unchecked(structure.shape(edge)?.clone());
        let blocks = structure.change_pave_blocks(edge)?.clone();
        for block in blocks {
            let has_shrunk = block.lock().expect("pave-block lock").has_shrunk_data();
            if has_shrunk && structure.is_valid_shrunk_data(&block)? {
                continue;
            }
            let (indices, range) = {
                let locked = block.lock().expect("pave-block lock must not be poisoned");
                (locked.indices(), locked.range())
            };
            let mut shrunk = ShrunkRange::new();
            shrunk.set_context(&self.context);
            shrunk.set_data(
                &shape,
                range.0,
                range.1,
                &Vertex::unchecked(structure.shape(indices.0)?.clone()),
                &Vertex::unchecked(structure.shape(indices.1)?.clone()),
            );
            computed.push((block, shrunk));
        }
        Ok(())
    }

    /// Whether the block covers its whole edge, and the shape the warning refers to.
    fn shrunk_warning_shape(
        &self,
        block: &PaveBlockHandle,
        edge: &Edge,
    ) -> Result<(bool, Shape), Failure> {
        let (edge_first, edge_last) = BRepTool::range(edge)?;
        let (range, original, indices) = {
            let locked = block.lock().expect("pave-block lock must not be poisoned");
            (locked.range(), locked.original_edge(), locked.indices())
        };
        let whole = range.0 <= edge_first && range.1 >= edge_last;
        if whole && original >= 0 {
            return Ok((whole, edge.clone().into_shape()));
        }
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        let mut shape = compound.into_shape();
        builder.add(&mut shape, &edge.clone().into_shape())?;
        builder.add(&mut shape, structure.shape(indices.0)?)?;
        builder.add(&mut shape, structure.shape(indices.1)?)?;
        Ok((whole, shape))
    }
}

fn set_shrunk(
    block: &PaveBlockHandle,
    first: f64,
    last: f64,
    bounds: &BoundingBox3,
    splittable: bool,
) {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .set_shrunk_data(first, last, bounds, splittable);
}
