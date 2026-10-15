//! `PrsDim_OffsetDimension`: the offset between two faces (planes, or the axes of revolved
//! faces), displayed relative to a transformation.
use super::dimension_impls::prs_dim_relation_class;
use super::relation_support::{null_object, relation_arrow, relation_owner};
use super::*;
use crate::dsg_prs::{DsgPrsArrowSide, DsgPrsOffsetPresentation};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::SelectMgrSelection;
use crate::select3d::{Select3DSensitiveBox, Select3DSensitiveSegment};
use crate::std_prs::StdPrsWfShape;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gce::MakeLin;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Line3, Point3, Transform3, Vector3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransform};

/// The attach points, directions and relative transformation of `PrsDim_OffsetDimension`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PrsDimOffsetFields {
    /// `myFAttach`.
    pub f_attach: Point3,
    /// `mySAttach`.
    pub s_attach: Point3,
    /// `myDirAttach`.
    pub dir_attach: Direction3,
    /// `myDirAttach2`.
    pub dir_attach2: Direction3,
    /// `myRelativePos`.
    pub relative_pos: Transform3,
}

prs_dim_relation_class!(
    PrsDimOffsetDimension,
    "PrsDim_OffsetDimension",
    PrsDimOffsetFields,
    "`PrsDim_OffsetDimension`; `PrsDimOffsetDimension::new(&f1, &f2, 10.0, &text)`.",
    kind = PrsDimKindOfDimension::Offset,
    movable = true
);

/// `|value / 10|` limited to [15, 30].
fn offset_arrow_size(value: f64) -> f64 {
    (value / 10.0).abs().clamp(15.0, 30.0)
}

fn is_revolved(kind: SurfaceType) -> bool {
    matches!(
        kind,
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Torus
    )
}

/// The axis of a cylinder, cone or torus face and its points at the first U on both V bounds.
fn axis_and_ends(surface: &BRepAdaptorSurface) -> Result<(Axis1, Point3, Point3), Failure> {
    let axis = match surface.get_type()? {
        SurfaceType::Cylinder => surface.cylinder()?.axis(),
        SurfaceType::Cone => surface.cone()?.axis(),
        SurfaceType::Torus => surface.torus()?.axis(),
        _ => Axis1::default(),
    };
    let u = surface.first_u_parameter()?;
    Ok((
        axis,
        surface.value(u, surface.first_v_parameter()?)?,
        surface.value(u, surface.last_v_parameter()?)?,
    ))
}

/// The projection on a line farther from a point of two points projected on it.
fn farther_projection(
    line: &Line3,
    point: &Point3,
    a: Point3,
    b: Point3,
) -> Result<Point3, Failure> {
    let pa = elclib::value(elclib::parameter(line, a)?, line);
    let pb = elclib::value(elclib::parameter(line, b)?, line);
    Ok(if pa.distance(point) > pb.distance(point) {
        pa
    } else {
        pb
    })
}

impl PrsDimOffsetDimension {
    /// `PrsDim_OffsetDimension(face1, face2, value, text)`: both arrows, size from the value.
    /// Example: `PrsDimOffsetDimension::new(&f1, &f2, 10.0, &text)`.
    pub fn new(first: &Shape, second: &Shape, value: f64, text: &ExtendedString) -> Handle<Self> {
        let relation = Self::alloc(PrsDimOffsetFields::default());
        {
            let mut f = relation.base.fields.write();
            f.f_shape = first.clone();
            f.s_shape = second.clone();
            f.symbol_prs = DsgPrsArrowSide::BothAr;
            f.val = value;
            f.text = text.clone();
            f.arrow_size = offset_arrow_size(value);
        }
        relation
    }
    /// `SetRelativePos`; `dimension.set_relative_pos(trsf)`.
    pub fn set_relative_pos(&self, trsf: Transform3) {
        self.fields.write().relative_pos = trsf;
    }
    fn reset_arrow_size(&self) {
        let mut f = self.relation_base().fields.write();
        f.arrow_size = offset_arrow_size(f.val);
    }

    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        let trsf = self.fields.read().relative_pos;
        self.reset_arrow_size();
        let surface1 = BRepAdaptorSurface::with_face(&topods::face(&self.first_shape())?, true)?;
        let surface2 = BRepAdaptorSurface::with_face(&topods::face(&self.second_shape())?, true)?;
        if is_revolved(surface1.get_type()?) {
            if is_revolved(surface2.get_type()?) {
                return self.compute_two_axes_offset(prs, &trsf);
            }
            return self.compute_axe_face_offset(prs, &trsf);
        }
        let plane1 = surface1.plane()?;
        let point1 = plane1.location();
        let plane2 = surface2.plane()?;
        let (u, v) = elslib::plane_parameters(plane2.position(), point1);
        let point2 = elslib::plane_value(u, v, plane2.position());
        let direction = if point1.is_equal(&point2, precision::confusion()) {
            plane1.axis().direction()
        } else {
            let vector = Vector3::between(point1, point2);
            Direction3::new(vector.x(), vector.y(), vector.z())?
        };
        self.fields.write().dir_attach = direction;
        self.compute_two_faces_offset(prs, &trsf)
    }

    /// The arrow length, the transformed shapes drawn in wireframe.
    fn draw_shapes(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        trsf: &Transform3,
    ) -> Result<(), Failure> {
        let drawer = self.attributes();
        for shape in [self.first_shape(), self.second_shape()] {
            let mut transform = BRepBuilderAPITransform::with_shape(&shape, *trsf, true, false)?;
            let moved = transform.shape()?.clone();
            StdPrsWfShape::add(prs, &moved, &drawer)?;
        }
        Ok(())
    }
    fn apply_arrow_length(&self) -> Result<(), Failure> {
        let arrow = relation_arrow(self)?;
        arrow
            .get()
            .ok_or_else(null_object)?
            .set_length(self.arrow_size());
        Ok(())
    }

    /// `ComputeTwoAxesOffset`: the offset between the axes of two revolved faces.
    /// Example: `dimension.compute_two_axes_offset(&prs, &trsf)?`.
    pub fn compute_two_axes_offset(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        trsf: &Transform3,
    ) -> Result<(), Failure> {
        let surface1 = BRepAdaptorSurface::with_face(&topods::face(&self.first_shape())?, true)?;
        let surface2 = BRepAdaptorSurface::with_face(&topods::face(&self.second_shape())?, true)?;
        let (axis1, first1, last1) = axis_and_ends(&surface1)?;
        let (axis2, first2, last2) = axis_and_ends(&surface2)?;
        let mut f = *self.fields.read();
        f.f_attach = axis1.location();
        f.s_attach = axis2.location();
        f.dir_attach = axis1.direction();
        f.dir_attach2 = f.dir_attach;
        let line = Line3::new(f.f_attach, f.dir_attach);
        let mut position = if self.automatic_position() {
            let (a, b) = (f.f_attach, f.s_attach);
            let position = Point3::new(
                (a.x() + b.x()) / 2.0 + 0.01,
                (a.y() + b.y()) / 2.0 + 0.01,
                (a.z() + b.z()) / 2.0 + 0.01,
            );
            self.relation_base().fields.write().position = position;
            position
        } else {
            self.position()
        };
        position = elclib::value(elclib::parameter(&line, position)?, &line);
        f.f_attach = farther_projection(&line, &position, first1, last1)?;
        f.s_attach = farther_projection(&line, &position, first2, last2)?;
        *self.fields.write() = f;
        self.apply_arrow_length()?;
        let t_first = f.f_attach.transformed(*trsf);
        let t_second = f.s_attach.transformed(*trsf);
        let t_dir = f.dir_attach.transformed(*trsf);
        let mut t_position = position.transformed(*trsf);
        let r = self.relation_base().fields.read().clone();
        if r.is_set_bnd_box {
            t_position = PrsDim::translate_point_to_bound(&t_position, &f.dir_attach, &r.bnd_box)?;
        }
        DsgPrsOffsetPresentation::add_axes(
            prs,
            &self.attributes(),
            &r.text,
            &t_first,
            &t_second,
            &t_dir,
            &t_dir,
            &t_position,
        )?;
        self.draw_shapes(prs, trsf)
    }

    /// `ComputeTwoFacesOffset`: the offset between two planar faces.
    /// Example: `dimension.compute_two_faces_offset(&prs, &trsf)?`.
    pub fn compute_two_faces_offset(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        trsf: &Transform3,
    ) -> Result<(), Failure> {
        let mut f = *self.fields.read();
        let normal = f.dir_attach;
        let r = self.relation_base().fields.read().clone();
        let mut position = Point3::default();
        let mut frame = Axis2::default();
        if r.automatic_position && !r.is_set_bnd_box {
            let first_shape = self.first_shape();
            let vertices = Explorer::new(&first_shape, ShapeEnum::Vertex, ShapeEnum::Shape)?;
            if vertices.more() {
                f.f_attach = BRepTool::pnt(&topods::vertex(vertices.current())?)?;
                let shift = Vector3::from(normal.xyz() * (r.val / 2.0).abs());
                let axes = Axis2::new(f.f_attach, normal)?;
                f.dir_attach = axes.x_direction();
                position = f.f_attach.translated(shift);
                if r.val <= precision::confusion() {
                    position = position.translated(Vector3::from(normal.xyz() * 0.001));
                }
                self.relation_base().fields.write().position = position;
                frame = axes;
            }
        } else {
            if r.automatic_position && r.is_set_bnd_box {
                let [_, _, _, xmax, ymax, zmax] = r.bnd_box.get()?;
                self.relation_base().fields.write().position = Point3::new(xmax, ymax, zmax);
            }
            position = self.position();
            f.f_attach = PrsDim::nearest_vertex(&self.first_shape(), &position)?;
            let axes = Axis2::new(f.f_attach, normal)?;
            if f.f_attach.distance(&position) <= precision::confusion() {
                f.dir_attach = axes.x_direction();
            } else {
                let orient = Direction3::from_xyz(f.f_attach.xyz() - position.xyz())?;
                if orient.angle(&normal) <= precision::angular() {
                    f.dir_attach = axes.x_direction();
                } else {
                    let side = normal.crossed(orient)?;
                    f.dir_attach = side.crossed(normal)?;
                }
            }
            frame = axes;
        }
        f.s_attach = PrsDim::nearest_vertex(&self.second_shape(), &position)?;
        let plane = Axis3::from_axis2(frame);
        let (uatt, vatt) = elslib::plane_parameters(plane, f.s_attach);
        let (u2, v2) = elslib::plane_parameters(plane, position);
        f.dir_attach2 = if uatt == u2 && vatt == v2 {
            f.dir_attach
        } else {
            let vector = Vector3::between(
                elslib::plane_value(uatt, vatt, plane),
                elslib::plane_value(u2, v2, plane),
            );
            Direction3::new(vector.x(), vector.y(), vector.z())?
        };
        *self.fields.write() = f;
        self.apply_arrow_length()?;
        DsgPrsOffsetPresentation::add(
            prs,
            &self.attributes(),
            &r.text,
            &f.f_attach.transformed(*trsf),
            &f.s_attach.transformed(*trsf),
            &f.dir_attach.transformed(*trsf),
            &f.dir_attach2.transformed(*trsf),
            &position.transformed(*trsf),
        )?;
        self.draw_shapes(prs, trsf)
    }

    /// `ComputeAxeFaceOffset`: only the transformed shapes.
    /// Example: `dimension.compute_axe_face_offset(&prs, &trsf)?`.
    pub fn compute_axe_face_offset(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        trsf: &Transform3,
    ) -> Result<(), Failure> {
        self.draw_shapes(prs, trsf)
    }

    fn compute_relation_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        self.reset_arrow_size();
        let f = *self.fields.read();
        let r = self.relation_base().fields.read().clone();
        let trsf = f.relative_pos;
        let t_first = f.f_attach.transformed(trsf);
        let t_second = f.s_attach.transformed(trsf);
        let t_dir = f.dir_attach.transformed(trsf);
        let t_dir2 = f.dir_attach2.transformed(trsf);
        let t_position = r.position.transformed(trsf);
        let l1 = Line3::new(t_first, t_dir);
        let l2 = Line3::new(t_second, t_dir2);
        let proj1 = elclib::value(elclib::parameter(&l1, t_position)?, &l1);
        let proj2 = elclib::value(elclib::parameter(&l2, t_position)?, &l2);
        let owner = relation_owner(self, 7);
        let target = selection.get().ok_or_else(null_object)?;
        let l3 = if !proj1.is_equal(&proj2, precision::confusion()) {
            MakeLin::from_points(proj1, proj2)?.value()?
        } else {
            let line = if !proj1.is_equal(&t_position, precision::confusion()) {
                Line3::new(
                    proj1,
                    Direction3::from_vector(Vector3::between(proj1, t_position))?,
                )
            } else {
                Line3::new(proj1, t_dir)
            };
            let size = (r.val / 100.0 + 1.0e-6).min(r.arrow_size + 1.0e-6);
            let p = t_position;
            let bounds = Select3DSensitiveBox::from_bounds(
                &owner,
                p.x(),
                p.y(),
                p.z(),
                p.x() + size,
                p.y() + size,
                p.z() + size,
            );
            target.add(&Handle::new(bounds).cast())?;
            line
        };
        let mut min = elclib::parameter(&l3, proj1)?;
        let mut max = min;
        for point in [proj2, t_position] {
            let current = elclib::parameter(&l3, point)?;
            min = min.min(current);
            max = max.max(current);
        }
        let (point_min, point_max) = (elclib::value(min, &l3), elclib::value(max, &l3));
        let segment = |a: Point3, b: Point3| {
            target.add(&Handle::new(Select3DSensitiveSegment::new(&owner, a, b)).cast())
        };
        if !point_min.is_equal(&point_max, precision::confusion()) {
            segment(point_min, point_max)?;
        }
        if !t_first.is_equal(&proj1, precision::confusion()) {
            segment(t_first, proj1)?;
        }
        if !t_second.is_equal(&proj2, precision::confusion()) {
            segment(t_second, proj2)?;
        }
        Ok(())
    }
}
