//! Native package operations supporting topology and contour analysis.
mod bounds;
mod contour;
use super::ShapeAnalysisCurve;
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Face, Wire},
};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point2;
/// Native `ShapeAnalysis` package operations; `ShapeAnalysis::outer_wire(&face)?`.
/// Period adjustment helpers are exported from the enclosing package.
pub struct ShapeAnalysis;
impl ShapeAnalysis {
    /// First positive-area wire, or the last wire without analyzing it.
    /// Example: `ShapeAnalysis::outer_wire(&face)?`.
    pub fn outer_wire(face: &Face) -> Result<Wire, Failure> {
        let face = topods::face(&face.oriented(Orientation::Forward))?;
        let mut explorer = Explorer::new(&face, ShapeEnum::Wire, ShapeEnum::Shape)?;
        while explorer.more() {
            let wire = topods::wire(explorer.current())?;
            explorer.next()?;
            if !explorer.more() {
                return Ok(wire);
            }
            let ordered = Handle::new(ShapeExtendWireData::from_wire(&wire, true, true)?);
            if Self::tot_cross_2d(&ordered, &face)? >= 0. {
                return Ok(wire);
            }
        }
        Ok(Wire::default())
    }
    /// Signed sampled pcurve area, closing gaps between successive edge samples.
    /// Example: `ShapeAnalysis::tot_cross_2d(&wire_data, &face)?`.
    pub fn tot_cross_2d(wire: &Handle<ShapeExtendWireData>, face: &Face) -> Result<f64, Failure> {
        let mut start = Point2::default();
        let mut previous = start;
        let mut count = 0;
        let mut total = 0.;
        for index in 1..=wire.nb_edges() {
            let edge = wire.edge(index)?;
            let curve = BRepTool::curve_on_face(&edge, face)?;
            if curve.pcurve.is_null() {
                continue;
            }
            count += 1;
            let mut points = Sequence::default();
            ShapeAnalysisCurve::get_sample_points_2d(
                &curve.pcurve,
                curve.first,
                curve.last,
                &mut points,
            )?;
            if edge.orientation() == Orientation::Reversed {
                points.reverse();
            }
            if count == 1 {
                previous = *points.value(1)?;
                start = previous;
            }
            for &point in points.iter() {
                total += cross_segment(previous, point);
                previous = point;
            }
        }
        Ok(total + cross_segment(previous, start))
    }
}
fn cross_segment(first: Point2, last: Point2) -> f64 {
    (first.x() - last.x()) * (first.y() + last.y()) / 2.
}
