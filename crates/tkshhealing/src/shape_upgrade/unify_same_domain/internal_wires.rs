//! "Build wires from internal edges" of `IntUnifyFaces` (ShapeUpgrade_UnifySameDomain.cxx):
//! the internal edges chained in both directions from the first one left.
use super::face_bounds::remove_edge_from_map;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The wires of the internal edges; `internal` is emptied.
pub(super) fn build_internal_wires(
    internal: &mut IndexedMapOfShape,
) -> Result<Vec<Shape>, Failure> {
    let mut ve_map = IndexedDataMapOfShapeListOfShape::default();
    for index in 1..=internal.extent() {
        topexp::map_shapes_and_ancestors(
            internal.find_key(index)?,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut ve_map,
        )?;
    }
    let mut wires = Vec::new();
    while !internal.is_empty() {
        let first = topods::edge(internal.find_key(1)?)?;
        internal.remove_from_index(1)?;
        remove_edge_from_map(first.as_ref(), &mut ve_map)?;
        wires.push(grow_wire(first, internal, &mut ve_map)?.into_shape());
    }
    Ok(wires)
}

/// The wire grown from `first` at both ends until it closes or stops.
fn grow_wire(
    first: Edge,
    internal: &mut IndexedMapOfShape,
    ve_map: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &first)?;
    let mut end_edges = [first.clone(), first.clone()];
    let (v0, v1) = topexp::vertices(&first, false);
    let mut ends: [Vertex; 2] = [v0, v1];
    loop {
        if ends[0].is_same(&ends[1]) {
            return Ok(wire); // "closed wire"
        }
        let mut found = false;
        for ii in 0..2 {
            let list: Vec<Shape> = ve_map
                .find_from_key(ends[ii].as_ref())?
                .iter()
                .cloned()
                .collect();
            let Some(next) = list.iter().find(|e| !e.is_same(&end_edges[ii])) else {
                continue;
            };
            found = true;
            let edge = topods::edge(next)?;
            internal.remove_key(next);
            remove_edge_from_map(next, ve_map)?;
            builder.add(&mut wire, &edge)?;
            let (a, b) = topexp::vertices(&edge, false);
            ends[ii] = if a.is_same(&ends[ii]) { b } else { a };
            end_edges[ii] = edge;
        }
        if !found {
            return Ok(wire); // "end of open wire"
        }
    }
}
