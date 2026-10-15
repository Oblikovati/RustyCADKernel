//! `PrsDim_AngleDimension`: the angle between two lines, three points, two faces or the
//! generatrices of a cone, drawn as an arc around the center point.
use super::dimension_impls::prs_dim_dimension_impls;
use super::dimension_linear::program;
use super::*;
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, prs_mgr_presentable_bind};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelection};
use crate::select3d::{Select3DSensitiveGroup, Select3DSensitiveSegment};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkgeombase::gce::{MakeCirc, MakePln, Root};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Axis1, Axis3, Direction3, Plane3, Point3, Vector3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;
use std::f64::consts::PI;

/// `THE_DEGREE_SYMBOL`.
const DEGREE_SYMBOL: u16 = 0x00B0;

/// The protected fields of `PrsDim_AngleDimension`.
#[derive(Clone, Default)]
pub struct PrsDimAngleDimensionFields {
    /// `myFirstPoint`.
    pub first_point: Point3,
    /// `mySecondPoint`.
    pub second_point: Point3,
    /// `myCenterPoint`.
    pub center_point: Point3,
    /// `myFirstShape`.
    pub first_shape: Shape,
    /// `mySecondShape`.
    pub second_shape: Shape,
    /// `myThirdShape`.
    pub third_shape: Shape,
    /// `myType`.
    pub angle_type: PrsDimTypeOfAngle,
    /// `myArrowsVisibility`.
    pub arrows_visibility: PrsDimTypeOfAngleArrowVisibility,
}

/// `PrsDim_AngleDimension`; `PrsDimAngleDimension::from_points(&a, &center, &b)?`.
pub struct PrsDimAngleDimension {
    transient: StandardTransient,
    pub(super) base: PrsDimDimensionBase,
    /// The protected fields.
    pub fields: SharedState<PrsDimAngleDimensionFields>,
}
rustycad_tkernel::impl_transient!(PrsDimAngleDimension, "PrsDim_AngleDimension", dyn PrsDimDimensionSource, transient,
    interfaces: [dyn PrsDimDimensionSource, dyn crate::ais::AisInteractiveObject, dyn crate::select_mgr::SelectMgrSelectableObject, dyn crate::prs_mgr::PrsMgrPresentableObject]);
prs_dim_dimension_impls!(PrsDimAngleDimension);

/// `gp_Vec(center, point).Normalized() * distance` from the center.
pub(super) fn attach_at(center: &Point3, point: &Point3, distance: f64) -> Result<Point3, Failure> {
    Ok(center.translated(Vector3::between(*center, *point).normalized()? * distance))
}

/// `isSameLine`: the center-to-ends vectors are parallel.
fn is_same_line(first: &Point3, center: &Point3, second: &Point3) -> Result<bool, Failure> {
    Vector3::between(*first, *center)
        .is_parallel(Vector3::between(*center, *second), precision::angular())
}

impl PrsDimAngleDimension {
    fn create(setup: impl FnOnce(&Self) -> Result<(), Failure>) -> Result<Handle<Self>, Failure> {
        let dimension = Handle::new(Self {
            transient: StandardTransient::default(),
            base: PrsDimDimensionBase::new(PrsDimKindOfDimension::PlaneAngle),
            fields: SharedState::new(PrsDimAngleDimensionFields::default()),
        });
        prs_mgr_presentable_bind(&dimension.cast());
        dimension.init();
        setup(&dimension)?;
        Ok(dimension)
    }
    /// `PrsDim_AngleDimension(edge1, edge2)`; `PrsDimAngleDimension::from_edges(&e1, &e2)?`.
    pub fn from_edges(first: &Edge, second: &Edge) -> Result<Handle<Self>, Failure> {
        Self::create(|d| d.set_measured_edges(first, second))
    }
    /// `PrsDim_AngleDimension(p1, center, p2)`; `PrsDimAngleDimension::from_points(&a, &c, &b)?`.
    pub fn from_points(
        first: &Point3,
        center: &Point3,
        second: &Point3,
    ) -> Result<Handle<Self>, Failure> {
        Self::create(|d| d.set_measured_points(first, center, second))
    }
    /// `PrsDim_AngleDimension(v1, center, v2)`; `PrsDimAngleDimension::from_vertices(&a, &c, &b)?`.
    pub fn from_vertices(
        first: &Vertex,
        center: &Vertex,
        second: &Vertex,
    ) -> Result<Handle<Self>, Failure> {
        Self::create(|d| d.set_measured_vertices(first, center, second))
    }
    /// `PrsDim_AngleDimension(cone)`; `PrsDimAngleDimension::from_cone(&face)?`.
    pub fn from_cone(cone: &Face) -> Result<Handle<Self>, Failure> {
        Self::create(|d| d.set_measured_cone(cone))
    }
    /// `PrsDim_AngleDimension(face1, face2)`; `PrsDimAngleDimension::from_faces(&f1, &f2)?`.
    pub fn from_faces(first: &Face, second: &Face) -> Result<Handle<Self>, Failure> {
        Self::create(|d| d.set_measured_faces(first, second))
    }
    /// `PrsDim_AngleDimension(face1, face2, point)`: the first attach point on the first face.
    /// Example: `PrsDimAngleDimension::from_faces_point(&f1, &f2, &p)?`.
    pub fn from_faces_point(
        first: &Face,
        second: &Face,
        point: &Point3,
    ) -> Result<Handle<Self>, Failure> {
        Self::create(|d| d.set_measured_faces_point(first, second, point))
    }

    /// `Init`: interior angle, both arrows, a degree symbol after the value, flyout 15.
    pub fn init(&self) {
        self.set_type(PrsDimTypeOfAngle::Interior);
        self.set_arrows_visibility(PrsDimTypeOfAngleArrowVisibility::Both);
        self.set_special_symbol(DEGREE_SYMBOL);
        self.set_display_special_symbol(PrsDimDisplaySpecialSymbol::After);
        self.set_flyout(15.0);
    }
    /// `FirstPoint`; `dimension.first_point()`.
    pub fn first_point(&self) -> Point3 {
        self.fields.read().first_point
    }
    /// `SecondPoint`; `dimension.second_point()`.
    pub fn second_point(&self) -> Point3 {
        self.fields.read().second_point
    }
    /// `CenterPoint`; `dimension.center_point()`.
    pub fn center_point(&self) -> Point3 {
        self.fields.read().center_point
    }
    /// `FirstShape`; `dimension.first_shape()`.
    pub fn first_shape(&self) -> Shape {
        self.fields.read().first_shape.clone()
    }
    /// `SecondShape`; `dimension.second_shape()`.
    pub fn second_shape(&self) -> Shape {
        self.fields.read().second_shape.clone()
    }
    /// `ThirdShape`; `dimension.third_shape()`.
    pub fn third_shape(&self) -> Shape {
        self.fields.read().third_shape.clone()
    }
    /// `SetType`; `dimension.set_type(PrsDimTypeOfAngle::Exterior)`.
    pub fn set_type(&self, angle_type: PrsDimTypeOfAngle) {
        self.fields.write().angle_type = angle_type;
    }
    /// `GetType`; `dimension.get_type()`.
    pub fn get_type(&self) -> PrsDimTypeOfAngle {
        self.fields.read().angle_type
    }
    /// `SetArrowsVisibility`; `dimension.set_arrows_visibility(PrsDimTypeOfAngleArrowVisibility::First)`.
    pub fn set_arrows_visibility(&self, visibility: PrsDimTypeOfAngleArrowVisibility) {
        self.fields.write().arrows_visibility = visibility;
    }
    /// `GetArrowsVisibility`; `dimension.get_arrows_visibility()`.
    pub fn get_arrows_visibility(&self) -> PrsDimTypeOfAngleArrowVisibility {
        self.fields.read().arrows_visibility
    }

    /// The end of the measured-geometry setters: the validity, the computed plane unless
    /// custom (when asked), and the update.
    fn finish_measured(&self, valid: bool, compute_plane: bool) -> Result<(), Failure> {
        self.base.fields.write().is_geometry_valid = valid;
        let custom = self.base.fields.read().is_plane_custom;
        if valid && !custom && compute_plane {
            self.compute_plane()?;
        }
        self.set_to_update_all();
        Ok(())
    }
    fn set_shapes(&self, first: Shape, second: Shape, third: Shape, kind: PrsDimGeometryType) {
        {
            let mut f = self.fields.write();
            f.first_shape = first;
            f.second_shape = second;
            f.third_shape = third;
        }
        self.base.fields.write().geometry_type = kind;
    }
    /// `SetMeasuredGeometry(edge1, edge2)`: the angle between two linear edges.
    /// Example: `dimension.set_measured_edges(&e1, &e2)?`.
    pub fn set_measured_edges(&self, first: &Edge, second: &Edge) -> Result<(), Failure> {
        self.set_shapes(
            first.clone().into(),
            second.clone().into(),
            Shape::default(),
            PrsDimGeometryType::Edges,
        );
        let (valid, plane) = self.init_two_edges_angle()?;
        let custom = self.base.fields.read().is_plane_custom;
        {
            let mut f = self.base.fields.write();
            f.is_geometry_valid = valid;
            if valid && !custom {
                f.plane = plane;
            }
        }
        self.set_to_update_all();
        Ok(())
    }
    /// `SetMeasuredGeometry(p1, center, p2)`: three points (as vertices).
    /// Example: `dimension.set_measured_points(&a, &c, &b)?`.
    pub fn set_measured_points(
        &self,
        first: &Point3,
        center: &Point3,
        second: &Point3,
    ) -> Result<(), Failure> {
        let vertices = [
            BRepLibMakeVertex::new(*first)?.vertex()?,
            BRepLibMakeVertex::new(*center)?.vertex()?,
            BRepLibMakeVertex::new(*second)?.vertex()?,
        ];
        self.set_points(*first, *center, *second)?;
        let [a, b, c] = vertices;
        self.set_shapes(a.into(), b.into(), c.into(), PrsDimGeometryType::Points);
        self.finish_points()
    }
    /// `SetMeasuredGeometry(v1, center, v2)`: the points of three vertices.
    /// Example: `dimension.set_measured_vertices(&a, &c, &b)?`.
    pub fn set_measured_vertices(
        &self,
        first: &Vertex,
        center: &Vertex,
        second: &Vertex,
    ) -> Result<(), Failure> {
        self.set_shapes(
            first.clone().into(),
            center.clone().into(),
            second.clone().into(),
            PrsDimGeometryType::Points,
        );
        self.set_points(
            BRepTool::pnt(first)?,
            BRepTool::pnt(center)?,
            BRepTool::pnt(second)?,
        )?;
        self.finish_points()
    }
    fn set_points(&self, first: Point3, center: Point3, second: Point3) -> Result<(), Failure> {
        let mut f = self.fields.write();
        f.first_point = first;
        f.center_point = center;
        f.second_point = second;
        Ok(())
    }
    fn finish_points(&self) -> Result<(), Failure> {
        let (first, center, second) = self.points();
        let valid = self.is_valid_points(&first, &center, &second);
        let same_line = is_same_line(&first, &center, &second)?;
        self.finish_measured(valid, !same_line)
    }
    /// `SetMeasuredGeometry(cone)`: the angle of a conical face.
    /// Example: `dimension.set_measured_cone(&face)?`.
    pub fn set_measured_cone(&self, cone: &Face) -> Result<(), Failure> {
        self.set_shapes(
            cone.clone().into(),
            Shape::default(),
            Shape::default(),
            PrsDimGeometryType::Face,
        );
        let valid = self.init_cone_angle()?;
        self.finish_measured(valid, true)
    }
    /// `SetMeasuredGeometry(face1, face2)`: the angle between two faces.
    /// Example: `dimension.set_measured_faces(&f1, &f2)?`.
    pub fn set_measured_faces(&self, first: &Face, second: &Face) -> Result<(), Failure> {
        self.set_shapes(
            first.clone().into(),
            second.clone().into(),
            Shape::default(),
            PrsDimGeometryType::Faces,
        );
        let valid = self.init_two_faces_angle(None)?;
        self.finish_measured(valid, true)
    }
    /// `SetMeasuredGeometry(face1, face2, point)`; `dimension.set_measured_faces_point(&f1, &f2, &p)?`.
    pub fn set_measured_faces_point(
        &self,
        first: &Face,
        second: &Face,
        point: &Point3,
    ) -> Result<(), Failure> {
        self.set_shapes(
            first.clone().into(),
            second.clone().into(),
            Shape::default(),
            PrsDimGeometryType::Faces,
        );
        let valid = self.init_two_faces_angle(Some(*point))?;
        self.finish_measured(valid, true)
    }

    /// The first, center and second points.
    pub(super) fn points(&self) -> (Point3, Point3, Point3) {
        let f = self.fields.read();
        (f.first_point, f.center_point, f.second_point)
    }
    /// The attach points at a distance from the center along the two sides.
    pub(super) fn attach_points(&self, distance: f64) -> Result<(Point3, Point3), Failure> {
        let (first, center, second) = self.points();
        Ok((
            attach_at(&center, &first, distance)?,
            attach_at(&center, &second, distance)?,
        ))
    }
    /// `ComputePlane`: the plane at the center with its Y axis on the bisector.
    /// Example: `dimension.compute_plane()?`.
    pub fn compute_plane(&self) -> Result<(), Failure> {
        if !self.base.fields.read().is_geometry_valid {
            return Ok(());
        }
        let (first, center, second) = self.points();
        let first_vec = Vector3::between(center, first);
        let second_vec = Vector3::between(center, second);
        let normal = second_vec.crossed(first_vec);
        let y = first_vec + second_vec;
        let x = y.crossed(normal);
        let position = Axis3::new(
            center,
            Direction3::from_vector(normal)?,
            Direction3::from_vector(x)?,
        )?;
        self.base.fields.write().plane = Plane3::from_position(position);
        Ok(())
    }
    /// `IsValidPoints`: ends off the center at a nonzero angle.
    /// Example: `dimension.is_valid_points(&a, &c, &b)`.
    pub fn is_valid_points(&self, first: &Point3, center: &Point3, second: &Point3) -> bool {
        first.distance(center) > precision::confusion()
            && second.distance(center) > precision::confusion()
            && Vector3::between(*center, *first)
                .angle(Vector3::between(*center, *second))
                .is_ok_and(|angle| angle > precision::angular())
    }
    /// `isArrowVisible`: whether the arrows setting shows an arrow.
    /// Example: `dimension.is_arrow_visible(PrsDimTypeOfAngleArrowVisibility::First)`.
    pub fn is_arrow_visible(&self, arrow: PrsDimTypeOfAngleArrowVisibility) -> bool {
        use PrsDimTypeOfAngleArrowVisibility as V;
        let current = self.get_arrows_visibility();
        match arrow {
            V::Both => current == V::Both,
            V::First => current == V::Both || current == V::First,
            V::Second => current == V::Both || current == V::Second,
            V::None => false,
        }
    }
    /// `GetNormalForMinAngle`: the plane normal, reversed when the angle from the first side
    /// to the second around it is negative; `dimension.get_normal_for_min_angle()?`.
    pub fn get_normal_for_min_angle(&self) -> Result<Direction3, Failure> {
        let normal = self.get_plane().axis().direction();
        let (first, center, second) = self.points();
        let first = Direction3::from_vector(Vector3::between(center, first))?;
        let second = Direction3::from_vector(Vector3::between(center, second))?;
        Ok(if first.angle_with_ref(second, normal) < 0.0 {
            normal.reversed()
        } else {
            normal
        })
    }
    /// `GetCenterOnArc`: the middle of the arc between two attach points (the origin when the
    /// plane or circle cannot be built); `dimension.get_center_on_arc(&a, &b, &c)?`.
    pub fn get_center_on_arc(
        &self,
        first_attach: &Point3,
        second_attach: &Point3,
        center: &Point3,
    ) -> Result<Point3, Failure> {
        let maker = MakePln::through_points(*first_attach, *second_attach, *center)?;
        if !maker.is_done() {
            return Ok(gp::origin());
        }
        let mut plane = maker.value()?;
        if self.get_type() == PrsDimTypeOfAngle::Exterior {
            let axis = plane.axis();
            plane.set_axis(Axis1::new(axis.location(), axis.direction().reversed()))?;
        }
        let radius = first_attach.distance(center);
        let circle = MakeCirc::from_center_plane(*center, plane, radius)?;
        if !circle.is_done() {
            return Ok(gp::origin());
        }
        let circle = circle.value()?;
        let (begin, end) = elclib::adjust_periodic(
            0.0,
            PI * 2.0,
            precision::p_confusion(),
            elclib::parameter(&circle, *first_attach)?,
            elclib::parameter(&circle, *second_attach)?,
        );
        Ok(elclib::value((begin + end) * 0.5, &circle))
    }

    fn compute_dimension(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        super::angle_dimension_draw::compute_angle(self, prs, mode)
    }
}

impl PrsDimDimensionSource for PrsDimAngleDimension {
    fn dimension_base(&self) -> &PrsDimDimensionBase {
        &self.base
    }
    /// `SetTextPosition`: a fixed label position in the working plane of a valid dimension
    /// (`Standard_ProgramError` off the plane); no update is requested.
    fn set_text_position(&self, position: &Point3) {
        if !self.is_valid() {
            return;
        }
        if !self.get_plane().contains(*position, precision::confusion()) {
            rustycad_tkernel::standard::raise_native_failure::<()>(Err(program(
                "The text position point for angle dimension doesn't belong to the working plane.",
            )));
        }
        let mut f = self.base.fields.write();
        f.is_text_position_fixed = true;
        f.fixed_text_position = *position;
    }
    /// `GetTextPosition`: the fixed position, else the label position from the alignment
    /// (the origin for an invalid dimension).
    fn get_text_position(&self) -> Point3 {
        if !self.is_valid() {
            return gp::origin();
        }
        if self.is_text_position_custom() {
            return self.base.fields.read().fixed_text_position;
        }
        rustycad_tkernel::standard::raise_native_failure(
            super::angle_dimension_layout::text_position(self),
        )
    }
    fn get_display_units(&self) -> AsciiString {
        self.attributes().dim_angle_display_units()
    }
    fn get_model_units(&self) -> AsciiString {
        self.attributes().dim_angle_model_units()
    }
    fn set_display_units(&self, units: &AsciiString) {
        self.attributes().set_dim_angle_display_units(units);
    }
    fn set_model_units(&self, units: &AsciiString) {
        self.attributes().set_dim_angle_model_units(units);
    }
    /// `CheckPlane`: a plane through one of the three points.
    fn check_plane(&self, plane: &Plane3) -> bool {
        let (first, center, second) = self.points();
        plane.contains(first, precision::confusion())
            || plane.contains(second, precision::confusion())
            || plane.contains(center, precision::confusion())
    }
    /// `ComputeValue`: the angle from the first side to the second around the min-angle
    /// normal, in `(0, 2 pi]`.
    fn compute_value(&self) -> f64 {
        if !self.is_valid() {
            return 0.0;
        }
        let (first, center, second) = self.points();
        let angle = rustycad_tkernel::standard::raise_native_failure(
            self.get_normal_for_min_angle().and_then(|normal| {
                Vector3::between(center, first)
                    .angle_with_ref(Vector3::between(center, second), Vector3::from(normal))
            }),
        );
        if angle > 0.0 { angle } else { 2.0 * PI + angle }
    }
    /// `ComputeFlyoutSelection`: the two flyout segments as one sensitive group.
    fn compute_flyout_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> Result<(), Failure> {
        let (first_attach, second_attach) = self.attach_points(self.get_flyout())?;
        let center = self.center_point();
        let group = Select3DSensitiveGroup::new(owner, true);
        group.add(&Handle::new(Select3DSensitiveSegment::new(owner, center, first_attach)).cast());
        group.add(&Handle::new(Select3DSensitiveSegment::new(owner, center, second_attach)).cast());
        selection
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "null selection"))?
            .add(&Handle::new(group).cast())
    }
}
