//! Constructor, setter, handle and copied-limit behavior of Contap search records.
use super::common::{Reader, attempt, end, flag, pnt, real, show};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{adaptor3d::Adaptor3dHVertexBase, top_abs::Orientation};
use rustycad_tkhlr::contap::{
    ContapThePathPointOfTheSearch as Point, ContapTheSegmentOfTheSearch as Segment,
};
fn path_point(point: &Point) {
    pnt(point.value());
    real(point.tolerance());
    flag(point.is_new());
    flag(point.arc().is_null());
    real(point.parameter());
    show(point.vertex(), |vertex| flag(vertex.is_null()));
}
fn segment(segment: &Segment) {
    flag(segment.curve().is_null());
    flag(segment.has_first_point());
    flag(segment.has_last_point());
    show(segment.first_point(), path_point);
    show(segment.last_point(), path_point);
}
/// Check every record member on each arc; `records_case(numbers)`.
pub fn records_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let arc = super::contap_hcurve::arc(&mut reader)?;
        let position = reader.pnt();
        let tolerance = reader.real();
        let parameter = reader.real();
        let vertex = Handle::new(Adaptor3dHVertexBase::new(
            Default::default(),
            Orientation::Forward,
            tolerance,
        ))
        .cast();
        let mut point = Point::new();
        path_point(&point);
        let mut segment_record = Segment::new();
        segment(&segment_record);
        segment_record.set_value(&arc);
        segment(&segment_record);
        point = Point::on_vertex(position, tolerance, &vertex, &arc, parameter);
        path_point(&point);
        segment_record.set_limit_point(&point, true);
        segment(&segment_record);
        point.set_value_on_arc(position, tolerance, &arc, parameter + 0.5);
        path_point(&point);
        segment(&segment_record);
        segment_record.set_limit_point(&point, false);
        segment(&segment_record);
        point = Point::on_arc(position, tolerance, &arc, parameter - 0.5);
        path_point(&point);
        point.set_value_on_vertex(position, tolerance, &vertex, &arc, parameter);
        path_point(&point);
        segment_record.set_value(&Default::default());
        segment(&segment_record);
        Ok(())
    });
    end();
}
