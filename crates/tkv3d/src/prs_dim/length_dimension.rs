//! `PrsDim_LengthDimension`: the distance between two points, an edge's ends, or two shapes
//! (faces, edges, vertices), optionally measured along a direction.
use super::dimension_impls::prs_dim_dimension_impls;
use super::*;
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, prs_mgr_presentable_bind};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelection};
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkgeombase::gce::MakePln;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction3, Line3, Plane3, Point3, Vector3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;

/// The protected fields of `PrsDim_LengthDimension`.
#[derive(Clone)]
pub struct PrsDimLengthDimensionFields {
    /// `myFirstPoint`.
    pub first_point: Point3,
    /// `mySecondPoint`.
    pub second_point: Point3,
    /// `myFirstShape`.
    pub first_shape: Shape,
    /// `mySecondShape`.
    pub second_shape: Shape,
    /// `myDirection`.
    pub direction: Direction3,
    /// `myHasCustomDirection`.
    pub has_custom_direction: bool,
}

/// `PrsDim_LengthDimension`; `PrsDimLengthDimension::from_points(&a, &b, &plane)?`.
pub struct PrsDimLengthDimension {
    transient: StandardTransient,
    pub(super) base: PrsDimDimensionBase,
    /// The protected fields.
    pub fields: SharedState<PrsDimLengthDimensionFields>,
}
rustycad_tkernel::impl_transient!(PrsDimLengthDimension, "PrsDim_LengthDimension", dyn PrsDimDimensionSource, transient,
    interfaces: [dyn PrsDimDimensionSource, dyn crate::ais::AisInteractiveObject, dyn crate::select_mgr::SelectMgrSelectableObject, dyn crate::prs_mgr::PrsMgrPresentableObject]);
prs_dim_dimension_impls!(PrsDimLengthDimension);

impl PrsDimLengthDimension {
    /// `PrsDim_LengthDimension()`: no geometry, flyout 15; `PrsDimLengthDimension::new()`.
    pub fn new() -> Handle<Self> {
        let dimension = Self::new_unflown();
        dimension.set_flyout(15.0);
        dimension
    }
    fn new_unflown() -> Handle<Self> {
        let dimension = Handle::new(Self {
            transient: StandardTransient::default(),
            base: PrsDimDimensionBase::new(PrsDimKindOfDimension::Length),
            fields: SharedState::new(PrsDimLengthDimensionFields {
                first_point: Point3::default(),
                second_point: Point3::default(),
                first_shape: Shape::default(),
                second_shape: Shape::default(),
                direction: Direction3::default(),
                has_custom_direction: false,
            }),
        });
        prs_mgr_presentable_bind(&dimension.cast());
        dimension
    }
    fn with(setup: impl FnOnce(&Self) -> Result<(), Failure>) -> Result<Handle<Self>, Failure> {
        let dimension = Self::new_unflown();
        setup(&dimension)?;
        dimension.set_flyout(15.0);
        Ok(dimension)
    }
    /// `PrsDim_LengthDimension(face1, face2)`; `PrsDimLengthDimension::from_faces(&f1, &f2)?`.
    pub fn from_faces(first: &Face, second: &Face) -> Result<Handle<Self>, Failure> {
        Self::with(|d| d.set_measured_faces(first, second))
    }
    /// `PrsDim_LengthDimension(face, edge)`; `PrsDimLengthDimension::from_face_edge(&f, &e)?`.
    pub fn from_face_edge(face: &Face, edge: &Edge) -> Result<Handle<Self>, Failure> {
        Self::with(|d| d.set_measured_face_edge(face, edge))
    }
    /// `PrsDim_LengthDimension(p1, p2, plane)`; `PrsDimLengthDimension::from_points(&a, &b, &plane)?`.
    pub fn from_points(
        first: &Point3,
        second: &Point3,
        plane: &Plane3,
    ) -> Result<Handle<Self>, Failure> {
        Self::with(|d| d.set_measured_points(first, second, plane))
    }
    /// `PrsDim_LengthDimension(shape1, shape2, plane)`: the custom plane, then the shapes.
    /// Example: `PrsDimLengthDimension::from_shapes(&s1, &s2, &plane)?`.
    pub fn from_shapes(
        first: &Shape,
        second: &Shape,
        plane: &Plane3,
    ) -> Result<Handle<Self>, Failure> {
        Self::with(|d| {
            d.set_custom_plane(plane);
            d.set_measured_shapes(first, second)
        })
    }
    /// `PrsDim_LengthDimension(edge, plane)`; `PrsDimLengthDimension::from_edge(&edge, &plane)?`.
    pub fn from_edge(edge: &Edge, plane: &Plane3) -> Result<Handle<Self>, Failure> {
        Self::with(|d| d.set_measured_edge(edge, plane))
    }

    /// `FirstPoint`; `dimension.first_point()`.
    pub fn first_point(&self) -> Point3 {
        self.fields.read().first_point
    }
    /// `SecondPoint`; `dimension.second_point()`.
    pub fn second_point(&self) -> Point3 {
        self.fields.read().second_point
    }
    /// `FirstShape`; `dimension.first_shape()`.
    pub fn first_shape(&self) -> Shape {
        self.fields.read().first_shape.clone()
    }
    /// `SecondShape`; `dimension.second_shape()`.
    pub fn second_shape(&self) -> Shape {
        self.fields.read().second_shape.clone()
    }

    /// `SetMeasuredGeometry(p1, p2, plane)`: two points (as vertices) in a custom plane.
    /// Example: `dimension.set_measured_points(&a, &b, &plane)?`.
    pub fn set_measured_points(
        &self,
        first: &Point3,
        second: &Point3,
        plane: &Plane3,
    ) -> Result<(), Failure> {
        let first_vertex = BRepLibMakeVertex::new(*first)?.vertex()?;
        let second_vertex = BRepLibMakeVertex::new(*second)?.vertex()?;
        {
            let mut f = self.fields.write();
            f.first_point = *first;
            f.second_point = *second;
            f.first_shape = first_vertex.into();
            f.second_shape = second_vertex.into();
        }
        self.base.fields.write().geometry_type = PrsDimGeometryType::Points;
        self.set_custom_plane(plane);
        self.base.fields.write().is_geometry_valid = self.is_valid_points(first, second);
        self.set_to_update_all();
        Ok(())
    }
    /// `SetMeasuredGeometry(edge, plane)`: the ends of a straight edge in a custom plane.
    /// Example: `dimension.set_measured_edge(&edge, &plane)?`.
    pub fn set_measured_edge(&self, edge: &Edge, plane: &Plane3) -> Result<(), Failure> {
        {
            let mut f = self.fields.write();
            f.first_shape = edge.clone().into();
            f.second_shape = Shape::default();
        }
        self.base.fields.write().geometry_type = PrsDimGeometryType::Edge;
        self.set_custom_plane(plane);
        let shape = self.first_shape();
        let valid = self.init_one_shape_points(&shape)?;
        self.base.fields.write().is_geometry_valid = valid;
        self.set_to_update_all();
        Ok(())
    }
    /// `SetMeasuredGeometry(face1, face2)`; `dimension.set_measured_faces(&f1, &f2)?`.
    pub fn set_measured_faces(&self, first: &Face, second: &Face) -> Result<(), Failure> {
        self.set_measured_shapes(&first.clone().into(), &second.clone().into())
    }
    /// `SetMeasuredGeometry(face, edge)`; `dimension.set_measured_face_edge(&f, &e)?`.
    pub fn set_measured_face_edge(&self, face: &Face, edge: &Edge) -> Result<(), Failure> {
        self.set_measured_shapes(&face.clone().into(), &edge.clone().into())
    }
    /// `SetMeasuredShapes`: the points and (unless custom) the plane of two shapes.
    /// Example: `dimension.set_measured_shapes(&s1, &s2)?`.
    pub fn set_measured_shapes(&self, first: &Shape, second: &Shape) -> Result<(), Failure> {
        {
            let mut f = self.fields.write();
            f.first_shape = first.clone();
            f.second_shape = second.clone();
        }
        let (valid, plane) = self.init_two_shapes_points(first, second)?;
        let custom = self.base.fields.read().is_plane_custom;
        let mut f = self.base.fields.write();
        f.is_geometry_valid = valid;
        if valid && !custom {
            match plane {
                Some(plane) => f.plane = plane,
                None => f.is_geometry_valid = false,
            }
        }
        drop(f);
        self.set_to_update_all();
        Ok(())
    }

    /// `ComputePlane`: the plane through the points and a third point along a direction
    /// (the default plane for invalid points); `dimension.compute_plane(&dir)?`.
    pub fn compute_plane(&self, attach: &Direction3) -> Result<Plane3, Failure> {
        let (first, second) = (self.first_point(), self.second_point());
        if !self.is_valid_points(&first, &second) {
            return Ok(Plane3::default());
        }
        let third = first.translated(Vector3::from(*attach));
        MakePln::through_points(first, second, third)?.value()
    }
    /// `IsValidPoints`: distinct points; `dimension.is_valid_points(&a, &b)`.
    pub fn is_valid_points(&self, first: &Point3, second: &Point3) -> bool {
        first.distance(second) > precision::confusion()
    }
    /// `SetDirection`: measures along a direction (when used); `dimension.set_direction(&dir, true)`.
    pub fn set_direction(&self, direction: &Direction3, use_direction: bool) {
        let mut f = self.fields.write();
        f.has_custom_direction = use_direction;
        if use_direction {
            f.direction = *direction;
        }
    }

    fn compute_dimension(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.base.fields.write().selection_geom.clear(mode);
        if !self.is_valid() {
            return Ok(());
        }
        let (first, second) = (self.first_point(), self.second_point());
        self.draw_linear_dimension(prs, mode, &first, &second, false)
    }
}

impl PrsDimDimensionSource for PrsDimLengthDimension {
    fn dimension_base(&self) -> &PrsDimDimensionBase {
        &self.base
    }
    /// `SetTextPosition`: a fixed label position of a valid dimension.
    fn set_text_position(&self, position: &Point3) {
        if !self.is_valid() {
            return;
        }
        {
            let mut f = self.base.fields.write();
            f.is_text_position_fixed = true;
            f.fixed_text_position = *position;
        }
        self.set_to_update_all();
    }
    /// `GetTextPosition`: the fixed position, else the linear label position.
    fn get_text_position(&self) -> Point3 {
        if self.is_text_position_custom() {
            return self.base.fields.read().fixed_text_position;
        }
        let (first, second) = (self.first_point(), self.second_point());
        rustycad_tkernel::standard::raise_native_failure(
            self.get_text_position_for_linear(&first, &second, false),
        )
    }
    /// `GetDisplayUnits`: the drawer length display units.
    fn get_display_units(&self) -> AsciiString {
        self.attributes().dim_length_display_units()
    }
    /// `GetModelUnits`: the drawer length model units.
    fn get_model_units(&self) -> AsciiString {
        self.attributes().dim_length_model_units()
    }
    /// `SetDisplayUnits`.
    fn set_display_units(&self, units: &AsciiString) {
        self.attributes().set_dim_length_display_units(units);
    }
    /// `SetModelUnits`.
    fn set_model_units(&self, units: &AsciiString) {
        self.attributes().set_dim_length_model_units(units);
    }
    /// `CheckPlane`: a plane through a point, not normal to the dimension.
    fn check_plane(&self, plane: &Plane3) -> bool {
        let (first, second) = (self.first_point(), self.second_point());
        let Ok(direction) = Direction3::from_xyz(second.xyz() - first.xyz()) else {
            // gce_MakeDir raises StdFail_NotDone for coincident points.
            rustycad_tkernel::standard::raise_native_failure::<()>(Err(Failure::with_message(
                "StdFail_NotDone",
                "gce_MakeDir::Value() - no result",
            )));
            return false;
        };
        let faulty = plane
            .axis()
            .direction()
            .is_parallel(direction, precision::angular());
        (plane.contains(first, precision::confusion())
            || plane.contains(second, precision::confusion()))
            && !faulty
    }
    /// `ComputeValue`: the distance, or its projection on the custom direction.
    fn compute_value(&self) -> f64 {
        if !self.is_valid() {
            return 0.0;
        }
        let f = self.fields.read();
        if !f.has_custom_direction {
            return f.first_point.distance(&f.second_point);
        }
        Vector3::between(f.first_point, f.second_point)
            .dot(Vector3::from(f.direction))
            .abs()
    }
    /// `ComputeFlyoutLinePoints`: along the custom direction when set.
    fn compute_flyout_line_points(
        &self,
        first: &Point3,
        second: &Point3,
    ) -> Result<(Point3, Point3), Failure> {
        let (custom, direction) = {
            let f = self.fields.read();
            (f.has_custom_direction, f.direction)
        };
        if !custom {
            return prs_dim_dimension_flyout_line_points(self, first, second);
        }
        let normal = self.get_plane().axis().direction();
        let flyout_dir = Vector3::from(normal)
            .crossed(Vector3::from(direction))
            .normalized()?;
        let projected = Vector3::between(*first, *second).dot(flyout_dir);
        let flyout_dir = Direction3::from_vector(flyout_dir)?;
        let line1 = Line3::new(*first, flyout_dir);
        let line2 = Line3::new(*second, flyout_dir);
        let flyout = self.get_flyout();
        Ok((
            elclib::value(
                elclib::parameter(&line1, *first)? + flyout + projected,
                &line1,
            ),
            elclib::value(elclib::parameter(&line2, *second)? + flyout, &line2),
        ))
    }
    /// `ComputeFlyoutSelection`: the linear flyouts of a valid dimension.
    fn compute_flyout_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> Result<(), Failure> {
        if !self.is_valid() {
            return Ok(());
        }
        let (first, second) = (self.first_point(), self.second_point());
        self.compute_linear_flyouts(selection, owner, &first, &second)
    }
}
