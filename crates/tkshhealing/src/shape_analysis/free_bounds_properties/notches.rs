use super::{
    ShapeAnalysisFreeBoundData, ShapeAnalysisFreeBoundsProperties as Properties, property_record,
    property_records,
};
use crate::{
    shape_analysis::{Curve3dRange, ShapeAnalysisEdge, ShapeAnalysisWire},
    shape_extend::ShapeExtendWireData,
};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Wire},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::Vector3;
impl Properties {
    /// Append detected notches to every record; `analyzer.check_notches(0.)?`.
    pub fn check_notches(&self, precision: f64) -> Result<bool, Failure> {
        for sequence in [&self.closed, &self.open] {
            for properties in property_records(sequence) {
                self.check_notches_data(&properties, precision)?;
            }
        }
        Ok(true)
    }
    /// Inspect each junction of a contour with at least two edges; `analyzer.check_notches_data(&record,0.)?`.
    pub fn check_notches_data(
        &self,
        properties: &Handle<ShapeAnalysisFreeBoundData>,
        precision: f64,
    ) -> Result<bool, Failure> {
        let record = property_record(properties)?;
        let wire = record.free_bound();
        let edges = ShapeExtendWireData::from_wire(&wire, true, true)?;
        if edges.nb_edges() <= 1 {
            return Ok(true);
        }
        for index in 1..=edges.nb_edges() {
            let mut notch = Wire::default();
            let mut width = 0.;
            if self.check_notches_at(&wire, index, &mut notch, &mut width, precision)? {
                record.add_notch(&notch, width);
            }
        }
        Ok(true)
    }
    /// Build the attempted notch even when rejected; `analyzer.check_notches_at(&wire,1,&mut notch,&mut width,0.)?`.
    pub fn check_notches_at(
        &self,
        wire: &Wire,
        index: i32,
        notch: &mut Wire,
        width: &mut f64,
        _precision: f64,
    ) -> Result<bool, Failure> {
        let edges = Handle::new(ShapeExtendWireData::from_wire(wire, true, true)?);
        BRepBuilder::default().make_wire(notch);
        if index <= 0 || index > edges.nb_edges() {
            return Ok(false);
        }
        let (first, last) = notch_edges(&edges, index, self.tolerance, notch)?;
        let Some((a, b)) = notch_curves(&first, &last)? else {
            return Ok(false);
        };
        let is_notch = junction_angle(&first, &last, &a, &b)? > 0.95 * std::f64::consts::PI;
        if !is_notch {
            return Ok(false);
        }
        *width = 0.;
        sample_notch_width(&a, &b, width)?;
        Ok(true)
    }
}
fn notch_edges(
    edges: &Handle<ShapeExtendWireData>,
    index: i32,
    tolerance: f64,
    notch: &mut Wire,
) -> Result<(Edge, Edge), Failure> {
    let builder = BRepBuilder::default();
    let first = edges.edge(index)?;
    builder.add(notch, &first)?;
    let analyzer = ShapeAnalysisWire::new();
    analyzer.load_wire_data(edges);
    analyzer.set_precision(tolerance);
    let mut next = next_index(index, edges.nb_edges());
    let tolerance = if tolerance < precision::confusion() {
        precision::confusion()
    } else {
        tolerance
    };
    if analyzer.check_small_at(next, tolerance)? {
        builder.add(notch, &edges.edge(next)?.into_shape())?;
        next = next_index(next, edges.nb_edges());
    }
    let last = edges.edge(next)?;
    builder.add(notch, &last)?;
    Ok((first, last))
}
fn next_index(index: i32, count: i32) -> i32 {
    if index < count { index + 1 } else { 1 }
}
fn notch_curves(
    first: &Edge,
    last: &Edge,
) -> Result<Option<(Curve3dRange, Curve3dRange)>, Failure> {
    let analyzer = ShapeAnalysisEdge::new();
    let a = analyzer.curve_3d(first, true)?;
    if a.0.is_null() {
        return Ok(None);
    }
    let b = analyzer.curve_3d(last, true)?;
    if b.0.is_null() {
        return Ok(None);
    }
    Ok(Some((a, b)))
}
fn junction_angle(
    first: &Edge,
    last: &Edge,
    a: &Curve3dRange,
    b: &Curve3dRange,
) -> Result<f64, Failure> {
    let (_, v1) = a.0.d1(a.2)?;
    let (_, v2) = b.0.d1(b.1)?;
    Ok(oriented_vector(first, v1)
        .angle(oriented_vector(last, v2))?
        .abs())
}
fn oriented_vector(edge: &Edge, vector: Vector3) -> Vector3 {
    if edge.orientation() == Orientation::Reversed {
        -vector
    } else {
        vector
    }
}
fn sample_notch_width(a: &Curve3dRange, b: &Curve3dRange, width: &mut f64) -> Result<(), Failure> {
    let (first, last) = if b.1 < b.2 { (b.1, b.2) } else { (b.2, b.1) };
    for index in 0..23 {
        let parameter = (f64::from(22 - index) * a.1 + f64::from(index) * a.2) / 22.;
        let projected = ProjectPointOnCurve::with_range(a.0.value(parameter)?, &b.0, first, last)?;
        let distance = if projected.nb_points() > 0 {
            projected.lower_distance()?
        } else {
            0.
        };
        if distance > *width {
            *width = distance;
        }
    }
    Ok(())
}
