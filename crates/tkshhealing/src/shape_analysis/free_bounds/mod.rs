//! Free boundary chaining and splitting from OCCT 3d097a0328e71b8.
use super::{ShapeAnalysisEdge, ShapeAnalysisShell, ShapeAnalysisWire};
use crate::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{self, Compound, Edge, Shape, ShapeIterator, Vertex, Wire},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::Orientation;
mod connect;
mod construct;
mod outputs;
mod selector;
mod split;
/// Finds open and closed free boundaries; `ShapeAnalysisFreeBounds::new()`.
#[derive(Default)]
pub struct ShapeAnalysisFreeBounds {
    closed: Compound,
    open: Compound,
}
impl ShapeAnalysisFreeBounds {
    /// Creates null result compounds; `ShapeAnalysisFreeBounds::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Closed boundary compound; `bounds.closed_wires()`.
    pub fn closed_wires(&self) -> &Compound {
        &self.closed
    }
    /// Open boundary compound; `bounds.open_wires()`.
    pub fn open_wires(&self) -> &Compound {
        &self.open
    }
    /// Chain manifold edges and propagate input reversals; `ShapeAnalysisFreeBounds::connect_edges_to_wires(&mut edges, 1e-7, true)?`.
    pub fn connect_edges_to_wires(
        edges: &mut [Shape],
        tolerance: f64,
        shared: bool,
    ) -> Result<Vec<Shape>, Failure> {
        let mut wires = Vec::new();
        let mut indices = Vec::new();
        for (index, edge) in edges.iter().enumerate() {
            if matches!(
                edge.orientation(),
                Orientation::Internal | Orientation::External
            ) {
                continue;
            }
            wires.push(make_wire(std::slice::from_ref(edge))?.into_shape());
            indices.push(index);
        }
        let result = Self::connect_wires_to_wires(&mut wires, tolerance, shared)?;
        for (wire, index) in wires.iter().zip(indices) {
            if wire.orientation() == Orientation::Reversed {
                edges[index].reverse();
            }
        }
        Ok(result)
    }
    /// Maximal wire chaining; input wire reversals are retained.
    /// Example: `ShapeAnalysisFreeBounds::connect_wires_to_wires(&mut wires, 1e-7, true)?`.
    pub fn connect_wires_to_wires(
        wires: &mut [Shape],
        tolerance: f64,
        shared: bool,
    ) -> Result<Vec<Shape>, Failure> {
        Self::connect_wires_with_vertices(wires, tolerance, shared, &mut Vec::new())
    }
    /// Chain wires and record endpoint substitutions by shape identity.
    /// Example: `ShapeAnalysisFreeBounds::connect_wires_with_vertices(&mut wires, 1e-7, false, &mut replacements)?`.
    pub fn connect_wires_with_vertices(
        wires: &mut [Shape],
        tolerance: f64,
        shared: bool,
        vertices: &mut Vec<(Shape, Shape)>,
    ) -> Result<Vec<Shape>, Failure> {
        let mut output = Vec::new();
        Self::connect_wires_with_vertices_into(wires, tolerance, shared, &mut output, vertices)?;
        Ok(output)
    }
    /// Append wires to compounds according to their closed flags; null compounds are initialized.
    /// Example: `ShapeAnalysisFreeBounds::dispatch_wires(&wires, &mut closed, &mut open)?`.
    pub fn dispatch_wires(
        wires: &[Shape],
        closed: &mut Compound,
        open: &mut Compound,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        if closed.is_null() {
            builder.make_compound(closed);
        }
        if open.is_null() {
            builder.make_compound(open);
        }
        for wire in wires {
            builder.add(if wire.closed()? { closed } else { open }, wire)?;
        }
        Ok(())
    }
}
fn make_wire(edges: &[Shape]) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for edge in edges {
        builder.add(&mut wire, edge)?;
    }
    Ok(wire)
}
fn compound_wires(compound: &Compound) -> Result<Vec<Shape>, Failure> {
    let mut shapes = Vec::new();
    let mut iterator = ShapeIterator::new(compound, true, true);
    while iterator.more() {
        shapes.push(iterator.value()?.clone());
        iterator.next();
    }
    Ok(shapes)
}
