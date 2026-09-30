//! Legacy output overloads preserve assignment timing and partially completed wires.
use super::*;
impl ShapeAnalysisFreeBounds {
    /// Assign the legacy edge output only after successful chaining.
    /// Example: `ShapeAnalysisFreeBounds::connect_edges_into(&mut edges,1e-7,true,&mut output)?`.
    pub fn connect_edges_into(
        edges: &mut [Shape],
        tolerance: f64,
        shared: bool,
        output: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        *output = Self::connect_edges_to_wires(edges, tolerance, shared)?;
        Ok(())
    }
    /// Assign the legacy wire output only after successful return-value overload completion.
    /// Example: `ShapeAnalysisFreeBounds::connect_wires_into(&mut wires,1e-7,true,&mut output)?`.
    pub fn connect_wires_into(
        wires: &mut [Shape],
        tolerance: f64,
        shared: bool,
        output: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        *output = Self::connect_wires_to_wires(wires, tolerance, shared)?;
        Ok(())
    }
    /// The map/output overload exposes completed wires on later failures.
    /// Example: `ShapeAnalysisFreeBounds::connect_wires_with_vertices_into(&mut wires,1e-7,true,&mut output,&mut replacements)?`.
    pub fn connect_wires_with_vertices_into(
        wires: &mut [Shape],
        tolerance: f64,
        shared: bool,
        output: &mut Vec<Shape>,
        vertices: &mut Vec<(Shape, Shape)>,
    ) -> Result<(), Failure> {
        connect::connect_wires(wires, tolerance, shared, vertices, output)
    }
}
