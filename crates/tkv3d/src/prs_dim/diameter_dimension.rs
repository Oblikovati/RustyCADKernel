//! `PrsDim_DiameterDimension`: the diameter of a circle or closed circular shape, drawn
//! between the anchor point and the opposite point.
use super::dimension_impls::prs_dim_dimension_impls;
use super::radius_dimension::PrsDimCircularFields;
use super::*;
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, prs_mgr_presentable_bind};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelection};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkgeomalgo::geom_api::IntCS;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Axis3, Circle3, Direction3, Plane3, Point3, Vector3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// `THE_DIAMETER_SYMBOL`: Ø.
const DIAMETER_SYMBOL: u16 = 0x00D8;

/// `PrsDim_DiameterDimension`; `PrsDimDiameterDimension::from_circle(&circle)?`.
pub struct PrsDimDiameterDimension {
    transient: StandardTransient,
    pub(super) base: PrsDimDimensionBase,
    /// The protected fields.
    pub fields: SharedState<PrsDimCircularFields>,
}
rustycad_tkernel::impl_transient!(PrsDimDiameterDimension, "PrsDim_DiameterDimension", dyn PrsDimDimensionSource, transient,
    interfaces: [dyn PrsDimDimensionSource, dyn crate::ais::AisInteractiveObject, dyn crate::select_mgr::SelectMgrSelectableObject, dyn crate::prs_mgr::PrsMgrPresentableObject]);
prs_dim_dimension_impls!(PrsDimDiameterDimension);

impl PrsDimDiameterDimension {
    fn create(
        plane: Option<&Plane3>,
        setup: impl FnOnce(&Self) -> Result<(), Failure>,
    ) -> Result<Handle<Self>, Failure> {
        let dimension = Handle::new(Self {
            transient: StandardTransient::default(),
            base: PrsDimDimensionBase::new(PrsDimKindOfDimension::Diameter),
            fields: SharedState::new(PrsDimCircularFields::default()),
        });
        prs_mgr_presentable_bind(&dimension.cast());
        if let Some(plane) = plane {
            dimension.set_custom_plane(plane);
        }
        setup(&dimension)?;
        dimension.set_special_symbol(DIAMETER_SYMBOL);
        dimension.set_display_special_symbol(PrsDimDisplaySpecialSymbol::Before);
        dimension.set_flyout(0.0);
        Ok(dimension)
    }
    /// `PrsDim_DiameterDimension(circle)`; `PrsDimDiameterDimension::from_circle(&circle)?`.
    pub fn from_circle(circle: &Circle3) -> Result<Handle<Self>, Failure> {
        Self::create(None, |d| d.set_measured_circle(circle))
    }
    /// `PrsDim_DiameterDimension(circle, plane)`; `PrsDimDiameterDimension::from_circle_plane(&c, &p)?`.
    pub fn from_circle_plane(circle: &Circle3, plane: &Plane3) -> Result<Handle<Self>, Failure> {
        Self::create(Some(plane), |d| d.set_measured_circle(circle))
    }
    /// `PrsDim_DiameterDimension(shape)`; `PrsDimDiameterDimension::from_shape(&shape)?`.
    pub fn from_shape(shape: &Shape) -> Result<Handle<Self>, Failure> {
        Self::create(None, |d| d.set_measured_shape(shape))
    }
    /// `PrsDim_DiameterDimension(shape, plane)`; `PrsDimDiameterDimension::from_shape_plane(&s, &p)?`.
    pub fn from_shape_plane(shape: &Shape, plane: &Plane3) -> Result<Handle<Self>, Failure> {
        Self::create(Some(plane), |d| d.set_measured_shape(shape))
    }
    /// `Circle`; `dimension.circle()`.
    pub fn circle(&self) -> Circle3 {
        self.fields.read().circle
    }
    /// `AnchorPoint`: the anchor of a valid dimension, else the origin.
    /// Example: `dimension.anchor_point()`.
    pub fn anchor_point(&self) -> Point3 {
        if !self.is_valid() {
            return gp::origin();
        }
        self.fields.read().anchor_point
    }
    /// `Shape`; `dimension.shape()`.
    pub fn shape(&self) -> Shape {
        self.fields.read().shape.clone()
    }
    fn finish_measure(&self) -> Result<(), Failure> {
        let (valid, custom) = {
            let f = self.base.fields.read();
            (f.is_geometry_valid, f.is_plane_custom)
        };
        if valid && custom {
            self.compute_anchor_point()?;
        } else if !custom {
            self.compute_plane();
            let circle = self.circle();
            self.fields.write().anchor_point = elclib::value(0.0, &circle);
        }
        self.set_to_update_all();
        Ok(())
    }
    /// `SetMeasuredGeometry(circle)`; `dimension.set_measured_circle(&circle)?`.
    pub fn set_measured_circle(&self, circle: &Circle3) -> Result<(), Failure> {
        let shape = BRepLibMakeEdge::from_circ(*circle)?.edge()?;
        {
            let mut f = self.fields.write();
            f.circle = *circle;
            f.shape = shape.into();
            f.anchor_point = gp::origin();
        }
        let valid = self.is_valid_circle(circle);
        let mut f = self.base.fields.write();
        f.geometry_type = PrsDimGeometryType::Edge;
        f.is_geometry_valid = valid;
        drop(f);
        self.finish_measure()
    }
    /// `SetMeasuredGeometry(shape)`: the circle of a closed circular shape.
    /// Example: `dimension.set_measured_shape(&shape)?`.
    pub fn set_measured_shape(&self, shape: &Shape) -> Result<(), Failure> {
        let previous = self.circle();
        {
            let mut f = self.fields.write();
            f.shape = shape.clone();
            f.anchor_point = gp::origin();
        }
        self.base.fields.write().geometry_type = PrsDimGeometryType::UndefShapes;
        let mut valid = false;
        if let Some((circle, _, closed)) = init_circular_dimension(shape, previous)? {
            self.fields.write().circle = circle;
            valid = self.is_valid_circle(&circle) && closed;
        }
        self.base.fields.write().is_geometry_valid = valid;
        self.finish_measure()
    }
    /// `ComputePlane`: the plane of the circle; `dimension.compute_plane()`.
    pub fn compute_plane(&self) {
        if !self.base.fields.read().is_geometry_valid {
            return;
        }
        let circle = self.circle();
        self.base.fields.write().plane =
            Plane3::from_position(Axis3::from_axis2(circle.position()));
    }
    /// `ComputeAnchorPoint`: the intersection of the circle with the custom plane whose
    /// diameter turns positively, or the point at parameter 0 without two intersections.
    /// Example: `dimension.compute_anchor_point()?`.
    pub fn compute_anchor_point(&self) -> Result<(), Failure> {
        let circle = self.circle();
        let plane = self.get_plane();
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(circle)).cast();
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(plane)).cast();
        let intersector = IntCS::with_curve(&curve, &surface)?;
        if !intersector.is_done() {
            self.base.fields.write().is_geometry_valid = false;
            return Ok(());
        }
        if intersector.nb_points()? != 2 {
            self.fields.write().anchor_point = elclib::value(0.0, &circle);
            self.base.fields.write().is_geometry_valid = true;
            return Ok(());
        }
        let (first, second) = (intersector.point(1)?, intersector.point(2)?);
        let first_dir =
            Direction3::from_xyz(circle.location().xyz() - first.xyz()).map_err(|_| {
                Failure::with_message("StdFail_NotDone", "gce_MakeDir::Value() - no result")
            })?;
        let direction = plane.axis().direction().crossed(first_dir)?;
        let positive =
            Vector3::from(direction).dot(Vector3::from(circle.position().direction())) > 0.0;
        self.fields.write().anchor_point = if positive { first } else { second };
        Ok(())
    }
    /// `ComputeSidePoints`: the anchor and its opposite point.
    /// Example: `dimension.compute_side_points(&circle)`.
    pub fn compute_side_points(&self, circle: &Circle3) -> (Point3, Point3) {
        let first = self.anchor_point();
        let radius = Vector3::between(circle.location(), first);
        (first, circle.location().translated(radius.reversed()))
    }
    /// `IsValidCircle`: a diameter above the confusion; `dimension.is_valid_circle(&circle)`.
    pub fn is_valid_circle(&self, circle: &Circle3) -> bool {
        circle.radius() * 2.0 > precision::confusion()
    }
    /// `IsValidAnchor`: an anchor off the circle in its plane (with the radius of the
    /// measured circle, as natively); `dimension.is_valid_anchor(&circle, &point)`.
    pub fn is_valid_anchor(&self, circle: &Circle3, anchor: &Point3) -> bool {
        let Ok(plane) = Plane3::new(circle.location(), circle.axis().direction()) else {
            return false;
        };
        let distance = anchor.distance(&circle.location());
        let radius = self.circle().radius();
        (distance - radius).abs() > precision::confusion()
            && plane.contains(*anchor, precision::confusion())
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
        let (first, second) = self.compute_side_points(&self.circle());
        self.draw_linear_dimension(prs, mode, &first, &second, false)
    }
}

impl PrsDimDimensionSource for PrsDimDiameterDimension {
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
    /// `GetTextPosition`: the fixed position, else the linear label position from the anchor
    /// to the center (as natively).
    fn get_text_position(&self) -> Point3 {
        if self.is_text_position_custom() {
            return self.base.fields.read().fixed_text_position;
        }
        let (circle, anchor) = (self.circle(), self.fields.read().anchor_point);
        rustycad_tkernel::standard::raise_native_failure(self.get_text_position_for_linear(
            &anchor,
            &circle.location(),
            false,
        ))
    }
    fn get_display_units(&self) -> AsciiString {
        self.attributes().dim_length_display_units()
    }
    fn get_model_units(&self) -> AsciiString {
        self.attributes().dim_length_model_units()
    }
    fn set_display_units(&self, units: &AsciiString) {
        self.attributes().set_dim_length_display_units(units);
    }
    fn set_model_units(&self, units: &AsciiString) {
        self.attributes().set_dim_length_model_units(units);
    }
    /// `CheckPlane`: a plane through the center.
    fn check_plane(&self, plane: &Plane3) -> bool {
        plane.contains(self.circle().location(), precision::confusion())
    }
    /// `ComputeValue`: the diameter of a valid dimension.
    fn compute_value(&self) -> f64 {
        if !self.is_valid() {
            return 0.0;
        }
        self.circle().radius() * 2.0
    }
    /// `ComputeFlyoutSelection`: the linear flyouts of the side points.
    fn compute_flyout_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> Result<(), Failure> {
        if !self.is_valid() {
            return Ok(());
        }
        let (first, second) = self.compute_side_points(&self.circle());
        self.compute_linear_flyouts(selection, owner, &first, &second)
    }
}
