//! Forecasting sewing and actual shared-edge constructors.
use super::*;
use rustycad_tkbrep::{topexp::Explorer, topods::Shell};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPISewing;
impl ShapeAnalysisFreeBounds {
    /// Forecast free bounds with the native sewing analyzer; `ShapeAnalysisFreeBounds::forecast(&shape,1e-6,false,true)?`.
    pub fn forecast(
        shape: &Shape,
        tolerance: f64,
        split_closed: bool,
        split_open: bool,
    ) -> Result<Self, Failure> {
        let mut sewer = BRepBuilderAPISewing::new(tolerance, false, false, true, false);
        let mut iterator = ShapeIterator::new(shape, true, true);
        while iterator.more() {
            sewer.add(iterator.value()?)?;
            iterator.next();
        }
        sewer.perform(&ProgressRange::default())?;
        let mut edges = Vec::new();
        for index in 1..=sewer.nb_free_edges() {
            let edge = sewer.free_edge(index)?;
            if !BRepTool::degenerated(&edge)? {
                edges.push(edge.into_shape());
            }
        }
        Self::from_free_edges(&mut edges, tolerance, false, split_closed, split_open)
    }
    /// Analyze actual shared-edge boundaries; `ShapeAnalysisFreeBounds::from_shape(&shape,false,true,false)?`.
    pub fn from_shape(
        shape: &Shape,
        split_closed: bool,
        split_open: bool,
        check_internal: bool,
    ) -> Result<Self, Failure> {
        let builder = BRepBuilder::default();
        let mut shell = Shell::default();
        builder.make_shell(&mut shell);
        for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            builder.add(&mut shell, &face)?;
        }
        let mut analyzer = ShapeAnalysisShell::new();
        analyzer.check_oriented_shells(&shell, true, check_internal)?;
        if !analyzer.has_free_edges() {
            return Ok(Self::new());
        }
        let mut edges = compound_wires(&analyzer.free_edges()?)?;
        let wires = Self::connect_edges_to_wires(&mut edges, precision::confusion(), true)?;
        let mut result = Self::new();
        Self::dispatch_wires(&wires, &mut result.closed, &mut result.open)?;
        result.split_results(0., true, split_closed, split_open)?;
        Ok(result)
    }
    fn from_free_edges(
        edges: &mut [Shape],
        tolerance: f64,
        shared: bool,
        split_closed: bool,
        split_open: bool,
    ) -> Result<Self, Failure> {
        let wires = Self::connect_edges_to_wires(edges, tolerance, shared)?;
        let mut result = Self::new();
        Self::dispatch_wires(&wires, &mut result.closed, &mut result.open)?;
        result.split_results(tolerance, shared, split_closed, split_open)?;
        Ok(result)
    }
    fn split_results(
        &mut self,
        tolerance: f64,
        shared: bool,
        split_closed: bool,
        split_open: bool,
    ) -> Result<(), Failure> {
        if !split_closed && !split_open {
            return Ok(());
        }
        let closed = compound_wires(&self.closed)?;
        let open = compound_wires(&self.open)?;
        let (mut closed, mut extra_open) = if split_closed {
            Self::split_wires(&closed, tolerance, shared)?
        } else {
            (closed, Vec::new())
        };
        let (extra_closed, open) = if split_open {
            Self::split_wires(&open, tolerance, shared)?
        } else {
            (Vec::new(), open)
        };
        closed.extend(extra_closed);
        extra_open.extend(open);
        self.closed = collect_compound(&closed)?;
        self.open = collect_compound(&extra_open)?;
        Ok(())
    }
}
fn collect_compound(shapes: &[Shape]) -> Result<Compound, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for shape in shapes {
        builder.add(&mut compound, shape)?;
    }
    Ok(compound)
}
