//! Native sensitive-debug presentation accumulation and graphics emission.
use super::{SelectMgrSelection, sensitive_presentation_entities::append_sensitive_geometry};
use crate::prs3d::{Prs3d, Prs3dLineAspect};
use rustycad_tkernel::{
    ncollection::{HSequence, List, Sequence},
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkservice::{
    aspect::{AspectTypeOfLine, AspectTypeOfMarker},
    graphic3d::*,
};
/// Auxiliary tools for selection presentations; `SelectMgr::compute_sensitive_prs(&prs, &selection, &location, &persistence)?`.
pub struct SelectMgr;
impl SelectMgr {
    /// Append the native sensitive geometry groups, retaining prior groups.
    /// Example: `SelectMgr::compute_sensitive_prs(&prs, &selection, &location, &persistence)?`.
    pub fn compute_sensitive_prs(
        prs: &Handle<dyn Graphic3dStructureSource>,
        selection: &Handle<SelectMgrSelection>,
        location: &Transform3,
        persistence: &Handle<dyn Graphic3dTransformPersSource>,
    ) -> Result<(), Failure> {
        prs.set_transform_persistence(persistence);
        let mut geometry = SensitivePresentationGeometry::default();
        for sensitive in selection.entities() {
            append_sensitive_geometry(prs, &mut geometry, &sensitive.base_sensitive(), *location)?;
        }
        geometry.emit(prs)
    }
}
#[derive(Default)]
pub(super) struct SensitivePresentationGeometry {
    pub lines: List<Handle<HSequence<Point3>>>,
    pub free_edges: List<Handle<HSequence<Point3>>>,
    pub points: Sequence<Point3>,
}
impl SensitivePresentationGeometry {
    pub fn append_line(&mut self, points: impl IntoIterator<Item = Point3>) {
        self.lines.append(sensitive_polyline(points));
    }
    fn emit(&mut self, prs: &Handle<dyn Graphic3dStructureSource>) -> Result<(), Failure> {
        self.emit_points(prs)?;
        emit_sensitive_lines(
            prs,
            &mut self.lines,
            NamedColor::AQUAMARINE1,
            AspectTypeOfLine::Dash,
            1.,
        )?;
        emit_sensitive_lines(
            prs,
            &mut self.free_edges,
            NamedColor::GREEN,
            AspectTypeOfLine::Solid,
            2.,
        )?;
        Ok(())
    }
    fn emit_points(&self, prs: &Handle<dyn Graphic3dStructureSource>) -> Result<(), Failure> {
        if self.points.is_empty() {
            return Ok(());
        }
        let points = Graphic3dArrayOfPoints::new(self.points.length(), 0)?;
        for point in self.points.iter() {
            points.add_vertex_pnt(point)?;
        }
        let group = prs.new_group()?;
        let aspect = Graphic3dAspectMarker3d::new(
            AspectTypeOfMarker::OPlus,
            Color::named(NamedColor::INDIANRED3),
            2.,
        )?;
        group.set_primitives_aspect(&Handle::new(aspect).cast());
        group.add_primitive_array(&Handle::new(points).cast(), true);
        Ok(())
    }
}
pub(super) fn sensitive_polyline(
    points: impl IntoIterator<Item = Point3>,
) -> Handle<HSequence<Point3>> {
    let line = HSequence::default();
    for point in points {
        line.append(point);
    }
    Handle::new(line)
}

fn emit_sensitive_lines(
    prs: &Handle<dyn Graphic3dStructureSource>,
    lines: &mut List<Handle<HSequence<Point3>>>,
    color: NamedColor,
    style: AspectTypeOfLine,
    width: f64,
) -> Result<(), Failure> {
    if lines.is_empty() {
        return Ok(());
    }
    let aspect = Handle::new(Prs3dLineAspect::new(Color::named(color), style, width)?);
    Prs3d::add_primitives_group(prs, &aspect, lines)
}
