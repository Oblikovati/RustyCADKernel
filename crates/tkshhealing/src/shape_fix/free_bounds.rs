// ShapeFix_FreeBounds at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::shape_analysis::ShapeAnalysisFreeBounds;
use crate::shape_extend::ShapeExtendExplorer;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Edge, Shape, ShapeIterator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Connect free boundaries and update affected source vertices; `ShapeFixFreeBounds::new()`.
#[derive(Default)]
pub struct ShapeFixFreeBounds {
    closed: Compound,
    open: Compound,
    shape: Shape,
}
impl ShapeFixFreeBounds {
    /// Construct null outputs; `ShapeFixFreeBounds::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Forecast free boundaries using sewing; `ShapeFixFreeBounds::forecast(&shape,1e-7,1e-4,false,true)?`.
    pub fn forecast(
        shape: &Shape,
        sew_tolerance: f64,
        close_tolerance: f64,
        split_closed: bool,
        split_open: bool,
    ) -> Result<Self, Failure> {
        let analysis =
            ShapeAnalysisFreeBounds::forecast(shape, sew_tolerance, split_closed, split_open)?;
        Self::from_analysis(shape, analysis, sew_tolerance, close_tolerance, false)
    }
    /// Repair actual shared-edge boundaries; `ShapeFixFreeBounds::from_shape(&shape,1e-4,false,true)?`.
    pub fn from_shape(
        shape: &Shape,
        close_tolerance: f64,
        split_closed: bool,
        split_open: bool,
    ) -> Result<Self, Failure> {
        let analysis = ShapeAnalysisFreeBounds::from_shape(shape, split_closed, split_open, false)?;
        Self::from_analysis(shape, analysis, 0., close_tolerance, true)
    }
    /// Return closed boundary wires; `tool.closed_wires()`.
    pub fn closed_wires(&self) -> &Compound {
        &self.closed
    }
    /// Return open boundary wires; `tool.open_wires()`.
    pub fn open_wires(&self) -> &Compound {
        &self.open
    }
    /// Return the source shape sharing any repaired vertices; `tool.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }
    fn from_analysis(
        shape: &Shape,
        analysis: ShapeAnalysisFreeBounds,
        sew_tolerance: f64,
        close_tolerance: f64,
        shared: bool,
    ) -> Result<Self, Failure> {
        let mut result = Self {
            shape: shape.clone(),
            closed: analysis.closed_wires().clone(),
            open: analysis.open_wires().clone(),
        };
        if close_tolerance > sew_tolerance {
            result.connect_open_wires(close_tolerance, shared)?;
        }
        Ok(result)
    }
    fn connect_open_wires(&mut self, tolerance: f64, shared: bool) -> Result<(), Failure> {
        let sequence = ShapeExtendExplorer::new().seq_from_compound(&self.open, false)?;
        let mut wires: Vec<Shape> = sequence.sequence(|values| values.iter().cloned().collect());
        let mut replacements = Vec::new();
        let connected = ShapeAnalysisFreeBounds::connect_wires_with_vertices(
            &mut wires,
            tolerance,
            shared,
            &mut replacements,
        )?;
        self.open.nullify();
        ShapeAnalysisFreeBounds::dispatch_wires(&connected, &mut self.closed, &mut self.open)?;
        for edge in Explorer::new(&self.shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            replace_edge_vertices(&mut topods::edge(&edge)?, &replacements)?;
        }
        Ok(())
    }
}
fn replace_edge_vertices(edge: &mut Edge, replacements: &[(Shape, Shape)]) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut children = ShapeIterator::new(edge, true, true);
    while children.more() {
        let vertex = topods::vertex(children.value()?)?;
        if let Some((_, replacement)) = replacements.iter().find(|(old, _)| old.is_same(&vertex)) {
            let mut replacement = topods::vertex(replacement)?;
            replacement.set_orientation(vertex.orientation());
            builder.remove(edge, &vertex)?;
            builder.add(edge, &replacement)?;
        }
        children.next();
    }
    Ok(())
}
