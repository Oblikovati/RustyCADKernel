//! Native maximal wire chaining and endpoint replacement.
use super::selector::{BoundsSelector, endpoint_box};
use super::*;
use crate::shape_build::{ShapeBuildEdge, ShapeBuildVertex};
use rustycad_tkernel::ncollection::{UBTree, UBTreeFiller};
use rustycad_tkmath::bnd::BoundingBox3;
pub(super) fn connect_wires(
    wires: &mut [Shape],
    tolerance: f64,
    shared: bool,
    vertices: &mut Vec<(Shape, Shape)>,
    output: &mut Vec<Shape>,
) -> Result<(), Failure> {
    output.clear();
    if wires.is_empty() {
        return Ok(());
    }
    let mut working = wires.to_vec();
    let tolerance = tolerance.max(precision::confusion());
    let mut selector = BoundsSelector::new(&mut working, shared);
    let Some(wire) = initialize_chains(&mut selector, output)? else {
        return Ok(());
    };
    run_chains(&mut selector, &wire, tolerance, vertices, output)?;
    wires.clone_from_slice(&working);
    Ok(())
}
fn run_chains(
    selector: &mut BoundsSelector<'_>,
    wire: &Handle<ShapeExtendWireData>,
    tolerance: f64,
    vertices: &mut Vec<(Shape, Shape)>,
    output: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let analyzer = ShapeAnalysisWire::new();
    analyzer.load_wire_data(wire);
    analyzer.set_precision(tolerance);
    let tree = build_tree(selector, tolerance)?;
    chain_loop(
        selector, &tree, wire, &analyzer, tolerance, vertices, output,
    )
}

fn initialize_chains(
    selector: &mut BoundsSelector<'_>,
    output: &mut Vec<Shape>,
) -> Result<Option<Handle<ShapeExtendWireData>>, Failure> {
    let mut first = None;
    for index in 0..selector.wires.len() {
        let wire = Handle::new(ShapeExtendWireData::from_wire(
            &topods::wire(&selector.wires[index])?,
            true,
            true,
        )?);
        if wire.nb_edges() > 0 {
            if first.is_none() {
                first = Some(wire);
                selector.consumed[index] = true;
            }
            continue;
        }
        if wire.nb_non_manifold_edges() > 0 {
            output.push(wire.wire()?.into_shape());
        }
        selector.consumed[index] = true;
    }
    Ok(first)
}
fn build_tree(
    selector: &BoundsSelector<'_>,
    tolerance: f64,
) -> Result<UBTree<usize, BoundingBox3>, Failure> {
    let mut tree = UBTree::default();
    let mut filler = UBTreeFiller::new(&mut tree, None, true);
    for (index, wire) in selector.wires.iter().enumerate() {
        if selector.consumed[index] {
            continue;
        }
        let (first, last) = rustycad_tkbrep::topexp::wire_vertices(&topods::wire(wire)?);
        let mut bounds = endpoint_box(BRepTool::pnt(&first)?, tolerance);
        bounds.add_point(BRepTool::pnt(&last)?);
        filler.add(index, bounds);
    }
    filler.fill();
    drop(filler);
    Ok(tree)
}
fn chain_loop(
    selector: &mut BoundsSelector<'_>,
    tree: &UBTree<usize, BoundingBox3>,
    wire: &Handle<ShapeExtendWireData>,
    analyzer: &ShapeAnalysisWire,
    tolerance: f64,
    vertices: &mut Vec<(Shape, Shape)>,
    output: &mut Vec<Shape>,
) -> Result<(), Failure> {
    loop {
        if select_next(selector, tree, wire, tolerance)? {
            append_selected(selector, wire)?;
            continue;
        }
        reconcile_vertices(wire, analyzer, vertices)?;
        output.push(finish_chain(wire, analyzer)?);
        wire.clear();
        if !start_next_chain(selector, wire)? {
            return Ok(());
        }
    }
}
fn select_next(
    selector: &mut BoundsSelector<'_>,
    tree: &UBTree<usize, BoundingBox3>,
    wire: &ShapeExtendWireData,
    tolerance: f64,
) -> Result<bool, Failure> {
    let edge = ShapeAnalysisEdge::new();
    selector.reset(
        [
            edge.first_vertex(&wire.edge(1)?),
            edge.last_vertex(&wire.edge(wire.nb_edges())?),
        ],
        tolerance,
    )?;
    let accepted = tree.select(selector);
    if let Some(error) = selector.error.take() {
        return Err(error);
    }
    Ok(accepted != 0 && selector.status > 0)
}
fn finish_chain(
    wire: &ShapeExtendWireData,
    analyzer: &ShapeAnalysisWire,
) -> Result<Shape, Failure> {
    let result = wire.wire()?;
    if !analyzer.check_connected_at(1, 0.)? && analyzer.last_check_status(ShapeExtendStatus::Ok) {
        result.set_closed(true)?;
    }
    Ok(result.into_shape())
}
fn append_selected(
    selector: &mut BoundsSelector<'_>,
    wire: &ShapeExtendWireData,
) -> Result<(), Failure> {
    let index = selector.selected;
    selector.consumed[index] = true;
    if selector.status == 2 || selector.status == 4 {
        selector.wires[index].reverse();
    }
    let current =
        ShapeExtendWireData::from_wire(&topods::wire(&selector.wires[index])?, true, true)?;
    if current.nb_edges() > 0 {
        wire.add_wire_data(&current, if selector.status <= 2 { 0 } else { 1 })?;
    }
    Ok(())
}
fn start_next_chain(
    selector: &mut BoundsSelector<'_>,
    wire: &ShapeExtendWireData,
) -> Result<bool, Failure> {
    for index in 0..selector.wires.len() {
        if selector.consumed[index] {
            continue;
        }
        wire.add_wire(&topods::wire(&selector.wires[index])?, 0)?;
        selector.consumed[index] = true;
        if wire.nb_edges() > 0 {
            return Ok(true);
        }
        wire.clear();
    }
    Ok(false)
}
fn reconcile_vertices(
    wire: &ShapeExtendWireData,
    analyzer: &ShapeAnalysisWire,
    vertices: &mut Vec<(Shape, Shape)>,
) -> Result<(), Failure> {
    for index in 1..=wire.nb_edges() {
        if analyzer.check_connected_at(index, 0.)? {
            reconcile_connection(
                wire,
                index,
                analyzer.last_check_status(ShapeExtendStatus::Done1),
                vertices,
            )?;
        }
    }
    Ok(())
}
fn reconcile_connection(
    wire: &ShapeExtendWireData,
    index: i32,
    coincident: bool,
    vertices: &mut Vec<(Shape, Shape)>,
) -> Result<(), Failure> {
    let previous = if index > 1 {
        index - 1
    } else {
        wire.nb_edges()
    };
    let first = wire.edge(previous)?;
    let second = wire.edge(index)?;
    let edge = ShapeAnalysisEdge::new();
    let before = edge.last_vertex(&first);
    let after = edge.first_vertex(&second);
    let vertex = if coincident {
        before.clone()
    } else {
        ShapeBuildVertex.combine_vertices(&before, &after, 1.0001)?
    };
    bind_vertex(vertices, &before, &vertex);
    bind_vertex(vertices, &after, &vertex);
    let indices = (previous, index);
    let edges = (&first, &second);
    replace_connection(wire, indices, edges, &vertex, coincident)
}
fn bind_vertex(vertices: &mut Vec<(Shape, Shape)>, old: &Vertex, new: &Vertex) {
    if let Some(pair) = vertices.iter_mut().find(|(key, _)| key.is_same(old)) {
        pair.1 = new.clone().into_shape();
        return;
    }
    vertices.push((old.clone().into_shape(), new.clone().into_shape()));
}
fn replace_connection(
    wire: &ShapeExtendWireData,
    indices: (i32, i32),
    edges: (&Edge, &Edge),
    vertex: &Vertex,
    coincident: bool,
) -> Result<(), Failure> {
    let builder = ShapeBuildEdge;
    if wire.nb_edges() < 2 {
        return wire.set(
            &builder.copy_replace_vertices(edges.1, vertex, vertex)?,
            indices.1,
        );
    }
    wire.set(
        &builder.copy_replace_vertices(edges.1, vertex, &Vertex::default())?,
        indices.1,
    )?;
    if !coincident {
        wire.set(
            &builder.copy_replace_vertices(edges.0, &Vertex::default(), vertex)?,
            indices.0,
        )?;
    }
    Ok(())
}
