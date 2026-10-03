//! Native sampled spatial area and infinite-point outer-bound classification.
use super::{ShapeAnalysis, ShapeAnalysisCurve};
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorSurface,
    topexp::Explorer,
    topods::{self, Edge, Face, ShapeIterator, Wire},
};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    top_abs::{Orientation, ShapeEnum, State},
};
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;
#[derive(Default)]
struct SpatialArea {
    first: Option<Point3>,
    previous: Point3,
    cross_sum: Xyz,
}
impl ShapeAnalysis {
    /// Compute the magnitude of the sampled 3D contour area, closing any final gap.
    /// Example: `let area = ShapeAnalysis::contour_area(&wire)?;`.
    pub fn contour_area(wire: &Wire) -> Result<f64, Failure> {
        let mut area = SpatialArea::default();
        let mut children = ShapeIterator::new(wire, false, true);
        while children.more() {
            area.include_edge(&topods::edge(children.value()?)?)?;
            children.next();
        }
        let first = area.first.unwrap_or_default();
        area.cross_sum += first.xyz().crossed(area.previous.xyz());
        Ok(area.cross_sum.modulus() * 0.5)
    }
    /// Determine whether a face has an outer bound using native single-wire area handling.
    /// Example: `assert!(ShapeAnalysis::is_outer_bound(&face)?);`.
    pub fn is_outer_bound(face: &Face) -> Result<bool, Failure> {
        let forward = topods::face(&face.oriented(Orientation::Forward))?;
        let mut wires = Explorer::new(&forward, ShapeEnum::Wire, ShapeEnum::Shape)?;
        let (mut count, mut last) = (0, Wire::default());
        while wires.more() {
            last = topods::wire(wires.current())?;
            count += 1;
            wires.next()?;
        }
        if count == 1 {
            let ordered = Handle::new(ShapeExtendWireData::from_wire(&last, true, true)?);
            return Ok(Self::tot_cross_2d(&ordered, &forward)? >= 0.);
        }
        classify_infinite_point(&forward)
    }
}
impl SpatialArea {
    fn include_edge(&mut self, edge: &Edge) -> Result<(), Failure> {
        let (curve, first, last) = BRepTool::curve(edge)?;
        if curve.is_null() {
            return Ok(());
        }
        let mut samples = Sequence::default();
        if !ShapeAnalysisCurve::get_sample_points(&curve, first, last, &mut samples)? {
            return Ok(());
        }
        if edge.orientation() == Orientation::Reversed {
            samples.reverse();
        }
        self.include_samples(&samples)
    }
    fn include_samples(&mut self, samples: &Sequence<Point3>) -> Result<(), Failure> {
        if self.first.is_none() {
            self.previous = *samples.value(1)?;
            self.first = Some(self.previous);
        }
        for &point in samples.iter() {
            self.cross_sum += point.xyz().crossed(self.previous.xyz());
            self.previous = point;
        }
        Ok(())
    }
}
fn classify_infinite_point(face: &Face) -> Result<bool, Failure> {
    let surface = BRepAdaptorSurface::with_face(face, false)?;
    let tolerance = BRepTool::face_tolerance(face)?;
    let u = surface.u_resolution(tolerance)?;
    let v = surface.v_resolution(tolerance)?;
    let tolerance = if v < u { v } else { u };
    Ok(BRepTopAdaptorFClass2d::new(face, tolerance)?.perform_infinite_point()? == State::Out)
}
