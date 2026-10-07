//! `BOPAlgo_PaveFiller_1.cxx`: vertex/vertex intersection and same-domain vertex creation.
use super::{BOPAlgoPaveFiller, missing_structure};
use crate::bop_algo::AlertSelfInterferingShape;
use crate::bop_algo::tools::{ConnectionMap, fill_map, make_blocks};
use crate::bop_ds::{DataStructure, InterfVV, ShapeInfo};
use crate::bop_tools::{compute_vertices, make_vertex};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Compound, Shape, Vertex},
};
use rustycad_tkernel::{
    message::AlertHandle,
    ncollection::{DefaultHashPolicy, List},
    precision,
    standard::Failure,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

type VertexLinks = ConnectionMap<i32, i32, DefaultHashPolicy>;

impl BOPAlgoPaveFiller {
    /// Intersects all vertex pairs with interfering boxes, merges connected coincident vertices
    /// into same-domain vertices and prepares pave blocks of the touched edges (`PerformVV`).
    /// `filler.perform_vv()?`.
    pub fn perform_vv(&mut self) -> Result<(), Failure> {
        let pairs = self.iterator_pairs(ShapeEnum::Vertex, ShapeEnum::Vertex)?;
        if pairs.is_empty() {
            return Ok(());
        }
        let links = self.coincident_vertex_links(&pairs)?;
        for block in make_blocks(&links, DefaultHashPolicy)? {
            self.make_sd_vertices(&block, true)?;
        }
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        let sources: Vec<i32> = structure
            .shape_sd_items()
            .map(|(source, _)| *source)
            .collect();
        for source in sources {
            structure.init_pave_blocks_for_vertex(source)?;
        }
        Ok(())
    }

    /// Merges the given vertices (and their current same-domain vertices) into one vertex,
    /// updating the first existing same-domain vertex in place or appending a new one, then
    /// binds every input to it and optionally records VV interferences (`MakeSDVertices`).
    /// `let merged = filler.make_sd_vertices(&[3, 7], true)?;`.
    pub fn make_sd_vertices(
        &mut self,
        vertices: &[i32],
        add_interfs: bool,
    ) -> Result<i32, Failure> {
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        let (existing, shapes) = sd_vertex_inputs(structure, vertices)?;
        let mut merged = Vertex::default();
        make_vertex(&shapes, &mut merged)?;
        let index = store_sd_vertex(structure, existing, &mut merged)?;
        let point = BRepTool::pnt(&merged)?;
        let gap = BRepTool::vertex_tolerance(&merged)? + precision::confusion();
        let bounds = structure.change_shape_info(index)?.change_box();
        bounds.add_point(point);
        bounds.set_gap(gap);
        self.bind_sd_vertices(vertices, index, add_interfs)?;
        Ok(index)
    }

    /// Returns the iterator pairs of the two shape types in traversal order.
    pub(super) fn iterator_pairs(
        &mut self,
        first: ShapeEnum,
        second: ShapeEnum,
    ) -> Result<Vec<(i32, i32)>, Failure> {
        let iterator = self.iterator.as_mut().ok_or_else(missing_structure)?;
        iterator.initialize(first, second);
        let mut pairs = Vec::with_capacity(iterator.expected_length().max(0) as usize);
        while iterator.more() {
            pairs.push(iterator.value()?);
            iterator.next();
        }
        Ok(pairs)
    }

    fn coincident_vertex_links(&self, pairs: &[(i32, i32)]) -> Result<VertexLinks, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut links = VertexLinks::with_hasher(DefaultHashPolicy, 100);
        for (first, second) in pairs.iter().copied() {
            if structure.has_interference_between(first, second) {
                fill_map(&mut links, first, second);
                continue;
            }
            let first_shape = sd_vertex(structure, first)?;
            let second_shape = sd_vertex(structure, second)?;
            if compute_vertices(&first_shape, &second_shape, self.options.fuzzy_value())? == 0 {
                fill_map(&mut links, first, second);
            }
        }
        Ok(links)
    }

    fn bind_sd_vertices(
        &mut self,
        vertices: &[i32],
        merged: i32,
        add_interfs: bool,
    ) -> Result<(), Failure> {
        for (position, first) in vertices.iter().copied().enumerate() {
            let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
            structure.add_shape_sd(first, merged);
            let rank = structure.rank(first);
            for second in vertices[position + 1..].iter().copied() {
                self.check_vertex_pair(first, second, rank, merged, add_interfs)?;
            }
        }
        Ok(())
    }

    fn check_vertex_pair(
        &mut self,
        first: i32,
        second: i32,
        rank: i32,
        merged: i32,
        add_interfs: bool,
    ) -> Result<(), Failure> {
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        if rank >= 0 && rank == structure.rank(second) {
            let pair = pair_compound(structure.shape(first)?, structure.shape(second)?)?;
            let alert: AlertHandle = Arc::new(AlertSelfInterferingShape::new(&pair));
            self.options.add_warning(&alert);
        }
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        if add_interfs && structure.add_interference(first, second) {
            let record = structure.interf_vv().append(InterfVV::default());
            record.set_indices(first, second);
            record.set_index_new(merged);
        }
        Ok(())
    }
}

fn sd_vertex(structure: &DataStructure, index: i32) -> Result<Vertex, Failure> {
    let mut representative = index;
    structure.has_shape_sd_into(index, &mut representative);
    Ok(Vertex::unchecked(structure.shape(representative)?.clone()))
}

/// The first same-domain vertex (kept out of the list, as natively) and the merge inputs:
/// the other same-domain vertices and every input vertex.
fn sd_vertex_inputs(
    structure: &DataStructure,
    vertices: &[i32],
) -> Result<(Option<i32>, List<Shape>), Failure> {
    let mut existing: Option<i32> = None;
    let mut shapes = List::default();
    for vertex in vertices.iter().copied() {
        let mut representative = vertex;
        if structure.has_shape_sd_into(vertex, &mut representative) {
            if existing.is_none() {
                existing = Some(representative);
            } else {
                shapes.append(structure.shape(representative)?.clone());
            }
        }
        shapes.append(structure.shape(vertex)?.clone());
    }
    Ok((existing, shapes))
}

/// Moves the merged geometry into the existing same-domain vertex, or appends the new vertex.
fn store_sd_vertex(
    structure: &mut DataStructure,
    existing: Option<i32>,
    merged: &mut Vertex,
) -> Result<i32, Failure> {
    if let Some(index) = existing {
        let target = Vertex::unchecked(structure.shape(index)?.clone());
        BRepBuilder::default().set_vertex_point_and_tolerance(
            &target,
            BRepTool::pnt(merged)?,
            BRepTool::vertex_tolerance(merged)?,
        )?;
        *merged = target;
        return Ok(index);
    }
    let mut info = ShapeInfo::default();
    info.set_shape_type(ShapeEnum::Vertex);
    info.set_shape(&merged.clone().into_shape());
    Ok(structure.append_shape_info(&info))
}

/// A compound of two shapes for warning alerts; `pair_compound(&v1, &v2)?`.
pub(super) fn pair_compound(first: &Shape, second: &Shape) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    builder.add(&mut shape, first)?;
    builder.add(&mut shape, second)?;
    Ok(shape)
}
