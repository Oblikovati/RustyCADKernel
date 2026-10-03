//! ShapeUpgrade_ClosedEdgeDivide at OCCT 3d097a0328e71b8.
use super::{EdgeDivideData, EdgeDivider, ShapeUpgradeEdgeDivide, SplitCurve};
use crate::shape_analysis::{PCurveRange, ShapeAnalysisEdge};
use rustycad_tkbrep::{brep::BRepTool, topods::Edge};
use rustycad_tkernel::{
    impl_transient,
    ncollection::HSequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::Point3;

/// Split a closed edge at its farthest sampled point when it exceeds vertex tolerance.
/// `let divider = ShapeUpgradeClosedEdgeDivide::new();`
#[derive(Default)]
pub struct ShapeUpgradeClosedEdgeDivide {
    parent: ShapeUpgradeEdgeDivide,
}
impl_transient!(ShapeUpgradeClosedEdgeDivide, "ShapeUpgrade_ClosedEdgeDivide", ShapeUpgradeEdgeDivide,
    parent, interfaces: [dyn EdgeDivider]);
impl std::ops::Deref for ShapeUpgradeClosedEdgeDivide {
    type Target = ShapeUpgradeEdgeDivide;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
impl EdgeDivider for ShapeUpgradeClosedEdgeDivide {
    fn edge_divide_data(&self) -> &EdgeDivideData {
        self.parent.edge_divide_data()
    }
    fn compute(&self, edge: &Edge) -> Result<bool, Failure> {
        self.clear();
        let analysis = ShapeAnalysisEdge::new();
        let first = analysis.first_vertex(edge);
        let last = analysis.last_vertex(edge);
        if !first.is_same(&last) || BRepTool::degenerated(edge)? {
            return Ok(false);
        }
        let vertex = BRepTool::pnt(&first)?;
        let tolerance = self.limit_tolerance(BRepTool::vertex_tolerance(&first)?);
        self.compute_closed(edge, vertex, tolerance * tolerance)
    }
}
impl ShapeUpgradeClosedEdgeDivide {
    /// Create the default edge-division tools; `ShapeUpgradeClosedEdgeDivide::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    fn compute_closed(&self, edge: &Edge, vertex: Point3, tolerance: f64) -> Result<bool, Failure> {
        let spatial = BRepTool::curve(edge)?;
        self.edge_divide_data()
            .with_mut(|f| f.has_curve_3d = !spatial.0.is_null());
        let planar = self.closed_pcurve(edge)?;
        self.edge_divide_data()
            .with_mut(|f| f.has_curve_2d = !planar.0.is_null());
        if !spatial.0.is_null() {
            return self.split_spatial(spatial, planar, vertex, tolerance);
        }
        if planar.0.is_null() {
            return Ok(false);
        }
        self.split_planar(planar, vertex, tolerance)
    }
    fn closed_pcurve(&self, edge: &Edge) -> Result<PCurveRange, Failure> {
        let face = self.edge_divide_data().with(|f| f.face.clone());
        if face.is_null() {
            return Ok((Handle::default(), 0., 0.));
        }
        ShapeAnalysisEdge::new().pcurve_on_face(edge, &face, false)
    }
    fn split_spatial(
        &self,
        spatial: (Handle<dyn GeomCurve>, f64, f64),
        planar: PCurveRange,
        vertex: Point3,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let (parameter, distance) =
            farthest_sample(vertex, spatial.1, spatial.2, |u| spatial.0.value(u))?;
        if !exceeds_tolerance(distance, tolerance) {
            return Ok(false);
        }
        let tool = self.split_curve_3d_tool();
        tool.init_curve_range(&spatial.0, spatial.1, spatial.2)?;
        tool.set_split_values(&single_parameter(parameter))?;
        self.edge_divide_data()
            .with_mut(|f| f.knots_3d = tool.split_values());
        if !planar.0.is_null() {
            self.initialize_planar(planar)?;
        }
        Ok(true)
    }
    fn initialize_planar(&self, planar: PCurveRange) -> Result<(), Failure> {
        let tool = self.split_curve_2d_tool();
        tool.init_curve_range(&planar.0, planar.1, planar.2)?;
        self.edge_divide_data()
            .with_mut(|f| f.knots_2d = tool.split_values());
        Ok(())
    }
    fn split_planar(
        &self,
        planar: PCurveRange,
        vertex: Point3,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let face = self.edge_divide_data().with(|f| f.face.clone());
        let surface = BRepTool::surface(&face)?;
        let (parameter, distance) = farthest_sample(vertex, planar.1, planar.2, |u| {
            let point = planar.0.value(u)?;
            surface.value(point.x(), point.y())
        })?;
        if !exceeds_tolerance(distance, tolerance) {
            return Ok(false);
        }
        let tool = self.split_curve_2d_tool();
        tool.init_curve_range(&planar.0, planar.1, planar.2)?;
        tool.set_split_values(&single_parameter(parameter))?;
        self.edge_divide_data()
            .with_mut(|f| f.knots_2d = tool.split_values());
        Ok(true)
    }
}
fn single_parameter(parameter: f64) -> Handle<HSequence<f64>> {
    let values = Handle::new(HSequence::default());
    values.append(parameter);
    values
}
fn exceeds_tolerance(distance: f64, tolerance: f64) -> bool {
    distance > tolerance
}
fn farthest_sample(
    vertex: Point3,
    first: f64,
    last: f64,
    mut evaluate: impl FnMut(f64) -> Result<Point3, Failure>,
) -> Result<(f64, f64), Failure> {
    let step = (last - first) / 22.;
    let (mut maximum_parameter, mut maximum_distance) = (first, 0.);
    let mut parameter = first + step;
    // The native loop uses repeated addition and samples its final endpoint too.
    for _ in 1..23 {
        let distance = vertex.square_distance(&evaluate(parameter)?);
        if distance > maximum_distance {
            maximum_parameter = parameter;
            maximum_distance = distance;
        }
        parameter += step;
    }
    Ok((maximum_parameter, maximum_distance))
}
