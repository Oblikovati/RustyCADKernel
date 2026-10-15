//! `PrsDim_SymmetricRelation::ComputeSelection`: the symmetry line between the two images of
//! the offset point, with the attach segments, for lines, arcs and vertices.
use super::relation_support::{null_object, relation_owner};
use super::symmetric_relation::{lin_of, project};
use super::*;
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelection};
use crate::select3d::{Select3DSensitiveBox, Select3DSensitiveSegment};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomCircle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gce::MakeLin;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3};

/// The selection target and its owner.
struct Target<'a> {
    selection: &'a SelectMgrSelection,
    owner: Handle<dyn SelectMgrEntityOwnerSource>,
}
impl Target<'_> {
    fn segment(&self, a: Point3, b: Point3) -> Result<(), Failure> {
        self.selection
            .add(&Handle::new(Select3DSensitiveSegment::new(&self.owner, a, b)).cast())
    }
    /// The line through two images of the offset point (along a fallback direction with a
    /// text box when they coincide) and its segment covering the images and the position.
    fn symmetry_line(
        &self,
        p1: Point3,
        p2: Point3,
        position: Point3,
        fallback: Direction3,
        box_size: f64,
    ) -> Result<(), Failure> {
        let line = if !p1.is_equal(&p2, precision::confusion()) {
            MakeLin::from_points(p1, p2)?.value()?
        } else {
            let p = position;
            let bounds = Select3DSensitiveBox::from_bounds(
                &self.owner,
                p.x(),
                p.y(),
                p.z(),
                p.x() + box_size,
                p.y() + box_size,
                p.z() + box_size,
            );
            self.selection.add(&Handle::new(bounds).cast())?;
            Line3::new(p1, fallback)
        };
        let mut min = elclib::parameter(&line, p1)?;
        let mut max = min;
        for point in [p2, position] {
            let current = elclib::parameter(&line, point)?;
            min = min.min(current);
            max = max.max(current);
        }
        let (point_min, point_max) = (elclib::value(min, &line), elclib::value(max, &line));
        if !point_min.is_equal(&point_max, precision::confusion()) {
            self.segment(point_min, point_max)?;
        }
        Ok(())
    }
}

/// The body of `PrsDim_SymmetricRelation::ComputeSelection`.
pub(super) fn symmetric_selection(
    relation: &PrsDimSymmetricRelation,
    selection: &Handle<SelectMgrSelection>,
) -> Result<(), Failure> {
    let target = Target {
        selection: selection.get().ok_or_else(null_object)?,
        owner: relation_owner(relation, 7),
    };
    let Some(axis) = relation.tool_axis()? else {
        return Ok(());
    };
    let laxis = lin_of(&axis.curve)?;
    let f = relation.fields.read().clone();
    let r = relation.relation_base().fields.read().clone();
    let size = (r.val / 100.0 + 1.0e-6).min(r.arrow_size + 1.0e-6);
    let position = r.position;
    let first_shape = relation.first_shape();
    if first_shape.shape_type()? == ShapeEnum::Vertex {
        if f.f_attach.is_equal(&f.s_attach, precision::confusion()) {
            return target.segment(position, f.f_attach);
        }
        let projected = project(&laxis, position)?;
        let to_first = Vector3::between(project(&laxis, f.f_attach)?, f.f_attach);
        let p1 = projected.translated(to_first);
        let p2 = projected.translated(to_first.reversed());
        target.symmetry_line(p1, p2, position, f.f_dir_attach, size)?;
        return attach_segments(&target, &f, p1, p2);
    }
    let edge = topods::edge(&first_shape)?;
    let kind = BRepAdaptorCurve::with_edge(&edge)?.get_type()?;
    if kind == CurveType::Line {
        let pj_attach = project(&laxis, f.f_attach)?;
        let pj_offset = project(&laxis, position)?;
        let h =
            (pj_offset.distance(&pj_attach) / f.axis_dir_attach.angle(&f.f_dir_attach).cos()).abs();
        let mut along = Vector3::from(f.f_dir_attach);
        if along.dot(Vector3::between(pj_attach, pj_offset)) < 0.0 {
            along.reverse();
        }
        let p1 = f.f_attach.translated(along * h);
        let on_axis = project(&laxis, p1)?;
        let p2 = on_axis.translated(Vector3::between(p1, on_axis));
        target.symmetry_line(p1, p2, position, f.f_dir_attach, size)?;
        attach_segments(&target, &f, p1, p2)?;
    }
    if kind == CurveType::Circle {
        let (curve, _, _) = BRepTool::curve(&edge)?;
        let circle = curve
            .cast::<GeomCircle>()
            .get()
            .ok_or_else(null_object)?
            .circ()?;
        let center = circle.location();
        let mut pj_offset = project(&laxis, position)?;
        let pj_center = project(&laxis, center)?;
        let mut vp = Vector3::between(pj_center, center);
        if vp.magnitude() <= precision::confusion() {
            let normal = relation
                .plane()
                .get()
                .ok_or_else(null_object)?
                .pln()
                .position()
                .direction();
            vp = Vector3::from(laxis.direction()).crossed(Vector3::from(normal));
        }
        let mut dt = pj_center.distance(&pj_offset);
        let radius = circle.radius();
        if dt > 0.999 * radius {
            dt = 0.999 * radius;
            let out = Vector3::between(pj_center, pj_offset);
            pj_offset = pj_center.translated(out / out.magnitude() * dt);
        }
        let h = (radius * radius - dt * dt).sqrt();
        let p1 = pj_offset.translated(vp + vp / vp.magnitude() * h);
        let p2 = pj_offset.translated(Vector3::between(p1, pj_offset));
        target.symmetry_line(p1, p2, position, laxis.direction(), size)?;
    }
    Ok(())
}

/// The segments from the attach points to their images when distinct.
fn attach_segments(
    target: &Target<'_>,
    f: &PrsDimSymmetricFields,
    p1: Point3,
    p2: Point3,
) -> Result<(), Failure> {
    if !f.f_attach.is_equal(&p1, precision::confusion()) {
        target.segment(f.f_attach, p1)?;
    }
    if !f.s_attach.is_equal(&p2, precision::confusion()) {
        target.segment(f.s_attach, p2)?;
    }
    Ok(())
}
