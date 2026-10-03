use super::{
    ShapeAnalysisFreeBoundData, ShapeAnalysisFreeBoundsProperties as Properties, property_record,
    property_records,
};
use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::{
    brep_tools::WireExplorer,
    topods::{Edge, Wire},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Xyz;
#[derive(Default)]
struct ContourSamples {
    count: i32,
    previous: Xyz,
    initial: Xyz,
    area: Xyz,
    length: f64,
}
impl Properties {
    /// Measure all accumulated contours; `analyzer.check_contours(0.)?`.
    pub fn check_contours(&self, precision: f64) -> Result<bool, Failure> {
        let mut status = false;
        for sequence in [&self.closed, &self.open] {
            for properties in property_records(sequence) {
                status |= self.fill_properties(&properties, precision)?;
            }
        }
        Ok(status)
    }
    /// Store the native 23-sample area, length, ratio and width; `analyzer.fill_properties(&record,0.)?`.
    pub fn fill_properties(
        &self,
        properties: &Handle<ShapeAnalysisFreeBoundData>,
        _precision: f64,
    ) -> Result<bool, Failure> {
        let properties = property_record(properties)?;
        let (area, length) = contour_properties(&properties.free_bound())?;
        let (ratio, width) = contour_aspect(area, length);
        properties.set_area(area);
        properties.set_perimeter(length);
        properties.set_ratio(ratio);
        properties.set_width(width);
        Ok(true)
    }
}
fn contour_properties(wire: &Wire) -> Result<(f64, f64), Failure> {
    let mut samples = ContourSamples::default();
    let mut explorer = WireExplorer::with_wire(wire)?;
    while explorer.more() {
        samples.add_edge(explorer.current())?;
        explorer.next()?;
    }
    samples.area += samples.initial.crossed(samples.previous);
    Ok((samples.area.modulus() / 2., samples.length))
}
impl ContourSamples {
    fn add_edge(&mut self, edge: &Edge) -> Result<(), Failure> {
        self.count += 1;
        let (curve, first, last) = ShapeAnalysisEdge::new().curve_3d(edge, true)?;
        if curve.is_null() {
            return Ok(());
        }
        let start = if self.count == 1 {
            self.previous = curve.value(first)?.xyz();
            self.initial = self.previous;
            1
        } else {
            0
        };
        for index in start..23 {
            let parameter = (f64::from(22 - index) * first + f64::from(index) * last) / 22.;
            self.add_point(curve.value(parameter)?.xyz());
        }
        Ok(())
    }
    fn add_point(&mut self, current: Xyz) {
        self.length += (current - self.previous).modulus();
        self.area += current.crossed(self.previous);
        self.previous = current;
    }
}
fn contour_aspect(area: f64, length: f64) -> (f64, f64) {
    if length == 0. {
        return (0., 0.);
    }
    let k = area / (length * length);
    if k == 0. {
        return (0., 0.);
    }
    let auxiliary = 1. - 16. * k;
    if auxiliary >= 0. {
        let ratio = (1. + auxiliary.sqrt()) / (8. * k);
        return (ratio - 1., length / (2. * ratio));
    }
    (0., 0.)
}
