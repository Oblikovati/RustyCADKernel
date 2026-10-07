//! `BOPAlgo_Tools::IntersectVertices`: chains of vertices whose tolerance spheres interfere.
use super::connection::{ConnectionMap, fill_map, make_blocks};
use crate::bop_tools::{BoxSet, PairSelector};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::ncollection::DefaultHashPolicy;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::{BoundingBox3, box3_to_bvh};

/// Groups vertices (each with a minimal tolerance) into chains of interfering vertices; chains
/// of connected pairs come first in BVH selection order, then every isolated vertex alone.
/// The input plays the native `IndexedDataMap<Shape, double>` and must not repeat a shape.
/// `let chains = intersect_vertices(&[(v1, 1e-7), (v2, 1e-7)], 0.)?;`.
pub fn intersect_vertices(
    vertices: &[(Shape, f64)],
    fuzzy_value: f64,
) -> Result<Vec<Vec<Shape>>, Failure> {
    if vertices.len() <= 1 {
        return Ok(vertices
            .iter()
            .map(|(shape, _)| vec![shape.clone()])
            .collect());
    }
    let tolerances = vertex_tolerances(vertices)?;
    let pairs = candidate_pairs(vertices, &tolerances, fuzzy_value)?;
    let mut links: ConnectionMap<i32, i32, DefaultHashPolicy> =
        ConnectionMap::with_hasher(DefaultHashPolicy, 1);
    for (first, second) in pairs {
        if spheres_interfere(vertices, &tolerances, first, second, fuzzy_value)? {
            fill_map(&mut links, first, second);
        }
    }
    let mut chains: Vec<Vec<Shape>> = Vec::new();
    for block in make_blocks(&links, DefaultHashPolicy)? {
        chains.push(
            block
                .iter()
                .map(|index| vertices[slot(*index)].0.clone())
                .collect(),
        );
    }
    for (index, (shape, _)) in vertices.iter().enumerate() {
        if !links.contains(&(index as i32 + 1)) {
            chains.push(vec![shape.clone()]);
        }
    }
    Ok(chains)
}

/// The larger of the vertex tolerance and the supplied minimum, per 1-based input index.
fn vertex_tolerances(vertices: &[(Shape, f64)]) -> Result<Vec<f64>, Failure> {
    vertices
        .iter()
        .map(|(shape, minimum)| {
            let tolerance = BRepTool::vertex_tolerance(&Vertex::unchecked(shape.clone()))?;
            Ok(if tolerance < *minimum {
                *minimum
            } else {
                tolerance
            })
        })
        .collect()
}

/// The non-rejected box pairs in native BVH traversal order; the tolerance test is applied
/// afterwards because the traversal does not depend on what Accept returns.
fn candidate_pairs(
    vertices: &[(Shape, f64)],
    tolerances: &[f64],
    fuzzy_value: f64,
) -> Result<Vec<(i32, i32)>, Failure> {
    let additional = fuzzy_value / 2.;
    let mut boxes = BoxSet::default();
    boxes.set_size(vertices.len());
    for (position, (shape, _)) in vertices.iter().enumerate() {
        let mut bounds = BoundingBox3::default();
        bounds.add_point(BRepTool::pnt(&Vertex::unchecked(shape.clone()))?);
        bounds.set_gap(tolerances[position] + additional);
        boxes.add(position as i32 + 1, box3_to_bvh(&bounds)?);
    }
    boxes.build()?;
    let mut selector = PairSelector::default();
    selector.set_shared_bvh_set(&mut boxes);
    selector.set_same(true);
    selector.select()?;
    Ok(selector
        .pairs()
        .iter()
        .map(|pair| (pair.id1, pair.id2))
        .collect())
}

fn spheres_interfere(
    vertices: &[(Shape, f64)],
    tolerances: &[f64],
    first: i32,
    second: i32,
    fuzzy_value: f64,
) -> Result<bool, Failure> {
    let first_point = BRepTool::pnt(&Vertex::unchecked(vertices[slot(first)].0.clone()))?;
    let second_point = BRepTool::pnt(&Vertex::unchecked(vertices[slot(second)].0.clone()))?;
    let mut limit = tolerances[slot(first)] + tolerances[slot(second)] + fuzzy_value;
    limit *= limit;
    Ok(first_point.square_distance(&second_point) < limit)
}

fn slot(index: i32) -> usize {
    (index - 1) as usize
}
