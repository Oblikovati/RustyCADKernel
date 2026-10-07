//! `BOPAlgo_PaveFiller::CorrectToleranceOfSE`: tolerance reduction of section edges.
use super::super::BOPAlgoPaveFiller;
use super::pool_updates::BlockFence;
use crate::bop_ds::{PaveBlockHandle, PaveBlockHandleIdentity};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy, IndexedDataMap, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    /// Lowers the tolerance of new section edges to their curve tolerance when the tangential
    /// tolerance inflated it, then lowers their vertices' tolerances to the real distances
    /// (`CorrectToleranceOfSE`).
    pub(super) fn correct_tolerance_of_se(&mut self) -> Result<(), Failure> {
        let mut by_vertex: IndexedDataMap<i32, Vec<PaveBlockHandle>, DefaultHashPolicy> =
            IndexedDataMap::with_hasher(DefaultHashPolicy, 1);
        let mut to_reduce: Map<i32> = Map::new(1);
        let mut fence: BlockFence = Map::with_hasher(PaveBlockHandleIdentity, 1);
        let structure = self.structure_mut()?;
        let count = structure.interf_ff_records().len();
        for ff in 0..count {
            let curve_count = structure.interf_ff_records()[ff].curves().len();
            for curve in 0..curve_count {
                reduce_curve_edges(
                    structure,
                    (ff, curve),
                    &mut fence,
                    &mut by_vertex,
                    &mut to_reduce,
                )?;
            }
        }
        if to_reduce.is_empty() {
            return Ok(());
        }
        let maxima = pool_edge_tolerances(structure, &to_reduce, &mut by_vertex)?;
        for index in 1..=by_vertex.extent() {
            let vertex = *by_vertex.find_key(index)?;
            if !to_reduce.contains(&vertex) {
                continue;
            }
            let blocks = by_vertex.find_from_index(index)?.clone();
            reduce_vertex_tolerance(
                structure,
                vertex,
                &blocks,
                maxima.seek(&vertex).copied().unwrap_or(0.),
            )?;
        }
        Ok(())
    }
}

/// Drops blocks without edge from the curve, lowers new section edges' tolerance, and maps
/// their (same-domain) end vertices to the blocks.
fn reduce_curve_edges(
    structure: &mut crate::bop_ds::DataStructure,
    (ff, curve): (usize, usize),
    fence: &mut BlockFence,
    by_vertex: &mut IndexedDataMap<i32, Vec<PaveBlockHandle>, DefaultHashPolicy>,
    to_reduce: &mut Map<i32>,
) -> Result<(), Failure> {
    let (tolerance, tangential) = {
        let stored = &structure.interf_ff_records()[ff].curves()[curve];
        (stored.tolerance(), stored.tangential_tolerance())
    };
    let blocks = structure.interf_ff_records()[ff].curves()[curve]
        .pave_blocks()
        .to_vec();
    let mut kept = Vec::with_capacity(blocks.len());
    for block in blocks {
        let (has_edge, edge, original, paves) = {
            let locked = block.lock().expect("pave-block lock");
            (
                locked.has_edge(),
                locked.edge(),
                locked.original_edge(),
                [locked.pave1().index(), locked.pave2().index()],
            )
        };
        if !has_edge {
            continue;
        }
        kept.push(Arc::clone(&block));
        if !fence.add(Arc::clone(&block)) {
            continue;
        }
        let mut reduced = false;
        if original < 0 && tolerance < tangential {
            // A small angle between the faces inflates the tangential tolerance of the curve.
            let shape = Edge::unchecked(structure.shape(edge)?.clone());
            if tolerance < BRepTool::edge_tolerance(&shape)? {
                edge_tshape(&shape, "BRep_TEdge::Tolerance")?.set_tolerance(tolerance);
                reduced = true;
            }
        }
        for vertex in paves {
            let vertex = structure.same_domain_index(vertex);
            if let Some(list) = by_vertex.change_seek(&vertex) {
                list.push(Arc::clone(&block));
            } else {
                by_vertex.add(vertex, vec![Arc::clone(&block)]);
            }
            if reduced {
                to_reduce.add(vertex);
            }
        }
    }
    *structure.interf_ff()[ff].change_curves()[curve].change_pave_blocks() = kept;
    Ok(())
}

/// The maximal tolerance of the pool edges at each vertex to reduce; also maps the vertices
/// to those blocks.
fn pool_edge_tolerances(
    structure: &crate::bop_ds::DataStructure,
    to_reduce: &Map<i32>,
    by_vertex: &mut IndexedDataMap<i32, Vec<PaveBlockHandle>, DefaultHashPolicy>,
) -> Result<DataMap<i32, f64>, Failure> {
    let mut maxima: DataMap<i32, f64> = DataMap::default();
    for list in structure.pave_blocks_pool() {
        for block in list {
            let (has_edge, edge, bounds) = {
                let locked = block.lock().expect("pave-block lock");
                (locked.has_edge(), locked.edge(), locked.indices())
            };
            if !has_edge {
                continue;
            }
            let tolerance =
                BRepTool::edge_tolerance(&Edge::unchecked(structure.shape(edge)?.clone()))?;
            for vertex in [bounds.0, bounds.1] {
                if !to_reduce.contains(&vertex) {
                    continue;
                }
                match maxima.change_seek(&vertex) {
                    None => {
                        maxima.bind(vertex, tolerance);
                    }
                    Some(maximum) if tolerance > *maximum => *maximum = tolerance,
                    Some(_) => {}
                }
                by_vertex.change_from_key(&vertex)?.push(Arc::clone(block));
            }
        }
    }
    Ok(maxima)
}

/// Lowers the vertex tolerance to the largest distance (plus edge tolerance) to the ends of
/// its blocks, when that gains at least 0.1%.
fn reduce_vertex_tolerance(
    structure: &crate::bop_ds::DataStructure,
    vertex: i32,
    blocks: &[PaveBlockHandle],
    minimum: f64,
) -> Result<(), Failure> {
    let shape = rustycad_tkbrep::topods::Vertex::unchecked(structure.shape(vertex)?.clone());
    let tolerance = BRepTool::vertex_tolerance(&shape)?;
    let mut maximum = minimum;
    if tolerance - maximum < 0.001 * tolerance {
        return Ok(());
    }
    let point = BRepTool::pnt(&shape)?;
    let mut fence: BlockFence = Map::with_hasher(PaveBlockHandleIdentity, 1);
    for block in blocks {
        if !fence.add(Arc::clone(block)) {
            continue;
        }
        let (edge, paves) = {
            let locked = block.lock().expect("pave-block lock");
            (locked.edge(), [locked.pave1(), locked.pave2()])
        };
        let edge = Edge::unchecked(structure.shape(edge)?.clone());
        let curve = BRepAdaptorCurve::with_edge(&edge)?;
        for pave in paves {
            if structure.same_domain_index(pave.index()) != vertex {
                continue;
            }
            let distance =
                point.distance(&curve.value(pave.parameter())?) + BRepTool::edge_tolerance(&edge)?;
            if distance > maximum {
                maximum = distance;
            }
        }
    }
    if maximum < tolerance {
        rustycad_tkbrep::brep::access::vertex_tshape(&shape, "BRep_TVertex::Tolerance")?
            .set_tolerance(maximum);
    }
    Ok(())
}
