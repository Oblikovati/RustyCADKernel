use super::polygon::WirePolygon;
use super::*;
use rustycad_tkgeombase::gcpnts::QuasiUniformDeflection;
use rustycad_tkmath::{geom_abs::Shape, poly::polygon_properties};
impl WirePolygon {
    pub(super) fn refine(&mut self, wire: &Wire, face: &Face) -> Result<f64, Failure> {
        let (mut area, perimeter) = polygon_properties(&self.points);
        let thickness = std_max(2. * area.abs() / perimeter, 1e-7);
        let mut deflection = std_max(self.deflection[0], self.deflection[1]);
        let mut target = std_min(deflection * 0.1, thickness * 10.);
        let mut changed = false;
        while deflection > thickness && target > 1e-7 {
            changed = true;
            self.rediscretize(wire, face, target)?;
            deflection = std_max(self.deflection[0], self.deflection[1]);
            target = std_min(target * 0.1, thickness * 10.);
        }
        if changed {
            area = polygon_properties(&self.points).0;
        }
        Ok(area)
    }
    fn rediscretize(&mut self, wire: &Wire, face: &Face, target: f64) -> Result<(), Failure> {
        self.points.clear();
        self.deflection = [0.; 2];
        let mut first = true;
        let mut explorer = WireExplorer::with_wire_on_face(wire, face)?;
        while explorer.more() {
            if !self.refine_candidate(explorer.current(), face, target, &mut first)? {
                break;
            }
            explorer.next()?;
        }
        Ok(())
    }
    fn refine_candidate(
        &mut self,
        edge: &Edge,
        face: &Face,
        target: f64,
        first: &mut bool,
    ) -> Result<bool, Failure> {
        if !boundary_edge(edge) {
            return Ok(true);
        }
        let (a, b) = BRepTool::range_on_face(edge, face)?;
        if (b - a).abs() < 1e-9 {
            return Ok(true);
        }
        if !self.refine_edge(edge, face, target, *first)? {
            return Ok(false);
        }
        *first = false;
        Ok(true)
    }
    fn refine_edge(
        &mut self,
        edge: &Edge,
        face: &Face,
        target: f64,
        first: bool,
    ) -> Result<bool, Failure> {
        let curve = BRepAdaptorCurve2d::with_edge_on_face(edge, face)?;
        let discretization =
            QuasiUniformDeflection::with_curve(&curve as &dyn Adaptor2dCurve2d, target, Shape::C1)?;
        if !discretization.is_done() {
            return Ok(false);
        }
        self.append_discretization(&curve, &discretization, edge.orientation(), first)?;
        Ok(true)
    }
    fn append_discretization(
        &mut self,
        curve: &dyn Adaptor2dCurve2d,
        discretization: &QuasiUniformDeflection,
        orientation: Orientation,
        first: bool,
    ) -> Result<(), Failure> {
        let count = discretization.nb_points()?;
        let (mut index, step, end) = discretization_indices(count, orientation, first);
        while index != end {
            self.points
                .push(curve.value(discretization.parameter(index)?)?);
            index += step;
        }
        if count > 2 {
            self.update_deflection()?;
        }
        Ok(())
    }
}

fn discretization_indices(count: i32, orientation: Orientation, first: bool) -> (i32, i32, i32) {
    let (index, step, end) = if orientation == Orientation::Reversed {
        (count, -1, 0)
    } else {
        (1, 1, count + 1)
    };
    (index + if first { 0 } else { step }, step, end)
}
