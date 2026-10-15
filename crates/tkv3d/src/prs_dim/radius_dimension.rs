//! `PrsDim_RadiusDimension`: the radius of a circle or circular shape, drawn from an anchor
//! point on the circle to its center.
use super::dimension_impls::prs_dim_dimension_impls;
use super::*;
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, prs_mgr_presentable_bind};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis3, Circle3, Direction3, Plane3, Point3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// `THE_RADIUS_SYMBOL`.
const RADIUS_SYMBOL: u16 = b'R' as u16;

/// The protected fields of a circular dimension: the circle, anchor point and shape.
#[derive(Clone, Default)]
pub struct PrsDimCircularFields {
    /// `myCircle`.
    pub circle: Circle3,
    /// `myAnchorPoint`.
    pub anchor_point: Point3,
    /// `myShape`.
    pub shape: Shape,
}

/// `PrsDim_RadiusDimension`; `PrsDimRadiusDimension::from_circle(&circle)?`.
pub struct PrsDimRadiusDimension {
    transient: StandardTransient,
    pub(super) base: PrsDimDimensionBase,
    /// The protected fields.
    pub fields: SharedState<PrsDimCircularFields>,
}
rustycad_tkernel::impl_transient!(PrsDimRadiusDimension, "PrsDim_RadiusDimension", dyn PrsDimDimensionSource, transient,
    interfaces: [dyn PrsDimDimensionSource, dyn crate::ais::AisInteractiveObject, dyn crate::select_mgr::SelectMgrSelectableObject, dyn crate::prs_mgr::PrsMgrPresentableObject]);
prs_dim_dimension_impls!(PrsDimRadiusDimension);

impl PrsDimRadiusDimension {
    fn create(setup: impl FnOnce(&Self) -> Result<(), Failure>) -> Result<Handle<Self>, Failure> {
        let dimension = Handle::new(Self {
            transient: StandardTransient::default(),
            base: PrsDimDimensionBase::new(PrsDimKindOfDimension::Radius),
            fields: SharedState::new(PrsDimCircularFields::default()),
        });
        prs_mgr_presentable_bind(&dimension.cast());
        setup(&dimension)?;
        dimension.set_special_symbol(RADIUS_SYMBOL);
        dimension.set_display_special_symbol(PrsDimDisplaySpecialSymbol::Before);
        dimension.set_flyout(0.0);
        Ok(dimension)
    }
    /// `PrsDim_RadiusDimension(circle)`: anchored at parameter 0.
    /// Example: `PrsDimRadiusDimension::from_circle(&circle)?`.
    pub fn from_circle(circle: &Circle3) -> Result<Handle<Self>, Failure> {
        Self::create(|d| d.set_measured_circle(circle, None))
    }
    /// `PrsDim_RadiusDimension(circle, anchor)`; `PrsDimRadiusDimension::from_circle_anchor(&c, &p)?`.
    pub fn from_circle_anchor(circle: &Circle3, anchor: &Point3) -> Result<Handle<Self>, Failure> {
        Self::create(|d| d.set_measured_circle(circle, Some(*anchor)))
    }
    /// `PrsDim_RadiusDimension(shape)`; `PrsDimRadiusDimension::from_shape(&shape)?`.
    pub fn from_shape(shape: &Shape) -> Result<Handle<Self>, Failure> {
        Self::create(|d| d.set_measured_shape(shape, None))
    }
    /// `Circle`; `dimension.circle()`.
    pub fn circle(&self) -> Circle3 {
        self.fields.read().circle
    }
    /// `AnchorPoint`; `dimension.anchor_point()`.
    pub fn anchor_point(&self) -> Point3 {
        self.fields.read().anchor_point
    }
    /// `Shape`; `dimension.shape()`.
    pub fn shape(&self) -> Shape {
        self.fields.read().shape.clone()
    }
    /// `SetMeasuredGeometry(circle, anchor, hasAnchor)`: the circle (its point at parameter 0
    /// without an anchor), with the plane through the anchor.
    /// Example: `dimension.set_measured_circle(&circle, Some(anchor))?`.
    pub fn set_measured_circle(
        &self,
        circle: &Circle3,
        anchor: Option<Point3>,
    ) -> Result<(), Failure> {
        let shape = BRepLibMakeEdge::from_circ(*circle)?.edge()?;
        let anchor = anchor.unwrap_or_else(|| elclib::value(0.0, circle));
        {
            let mut f = self.fields.write();
            f.circle = *circle;
            f.shape = shape.into();
            f.anchor_point = anchor;
        }
        self.base.fields.write().geometry_type = PrsDimGeometryType::Edge;
        let valid = self.is_valid_circle(circle) && self.is_valid_anchor(circle, &anchor);
        self.base.fields.write().is_geometry_valid = valid;
        if valid {
            self.compute_plane()?;
        }
        self.set_to_update_all();
        Ok(())
    }
    /// `SetMeasuredGeometry(shape, anchor, hasAnchor)`: the circle of a shape, anchored at
    /// the middle of its arc unless an anchor is given.
    /// Example: `dimension.set_measured_shape(&shape, None)?`.
    pub fn set_measured_shape(&self, shape: &Shape, anchor: Option<Point3>) -> Result<(), Failure> {
        let previous = self.circle();
        self.fields.write().shape = shape.clone();
        self.base.fields.write().geometry_type = PrsDimGeometryType::UndefShapes;
        let found = init_circular_dimension(shape, previous)?;
        let mut valid = false;
        if let Some((circle, middle, _)) = found {
            let mut f = self.fields.write();
            f.circle = circle;
            f.anchor_point = middle;
            drop(f);
            valid = self.is_valid_circle(&circle);
        }
        if let Some(anchor) = anchor {
            self.fields.write().anchor_point = anchor;
            valid = valid && self.is_valid_anchor(&self.circle(), &anchor);
        }
        self.base.fields.write().is_geometry_valid = valid;
        if valid {
            self.compute_plane()?;
        }
        self.set_to_update_all();
        Ok(())
    }
    /// `ComputePlane`: the circle plane with X toward the center from the anchor.
    /// Example: `dimension.compute_plane()?`.
    pub fn compute_plane(&self) -> Result<(), Failure> {
        if !self.base.fields.read().is_geometry_valid {
            return Ok(());
        }
        let (circle, anchor) = (self.circle(), self.anchor_point());
        let x = Direction3::from_xyz(circle.location().xyz() - anchor.xyz()).map_err(|_| {
            Failure::with_message("StdFail_NotDone", "gce_MakeDir::Value() - no result")
        })?;
        let plane =
            Plane3::from_position(Axis3::new(circle.location(), circle.axis().direction(), x)?);
        self.base.fields.write().plane = plane;
        Ok(())
    }
    /// `IsValidCircle`: a radius above the confusion; `dimension.is_valid_circle(&circle)`.
    pub fn is_valid_circle(&self, circle: &Circle3) -> bool {
        circle.radius() > precision::confusion()
    }
    /// `IsValidAnchor`: an anchor off the center in the circle plane.
    /// Example: `dimension.is_valid_anchor(&circle, &point)`.
    pub fn is_valid_anchor(&self, circle: &Circle3, anchor: &Point3) -> bool {
        let Ok(plane) = Plane3::new(circle.location(), circle.axis().direction()) else {
            return false;
        };
        anchor.distance(&circle.location()) > precision::confusion()
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
        let (circle, anchor) = (self.circle(), self.anchor_point());
        self.draw_linear_dimension(prs, mode, &anchor, &circle.location(), true)
    }
}

impl PrsDimDimensionSource for PrsDimRadiusDimension {
    fn dimension_base(&self) -> &PrsDimDimensionBase {
        &self.base
    }
    /// `SetTextPosition`: a fixed label position of a valid geometry.
    fn set_text_position(&self, position: &Point3) {
        if !self.base.fields.read().is_geometry_valid {
            return;
        }
        {
            let mut f = self.base.fields.write();
            f.is_text_position_fixed = true;
            f.fixed_text_position = *position;
        }
        self.set_to_update_all();
    }
    /// `GetTextPosition`: the fixed position, else the one-sided linear label position.
    fn get_text_position(&self) -> Point3 {
        if self.is_text_position_custom() {
            return self.base.fields.read().fixed_text_position;
        }
        let (circle, anchor) = (self.circle(), self.anchor_point());
        rustycad_tkernel::standard::raise_native_failure(self.get_text_position_for_linear(
            &anchor,
            &circle.location(),
            true,
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
    /// `CheckPlane`: a plane through the anchor or the center.
    fn check_plane(&self, plane: &Plane3) -> bool {
        let (circle, anchor) = (self.circle(), self.anchor_point());
        plane.contains(anchor, precision::confusion())
            || plane.contains(circle.location(), precision::confusion())
    }
    /// `ComputeValue`: the radius of a valid dimension.
    fn compute_value(&self) -> f64 {
        if !self.is_valid() {
            return 0.0;
        }
        self.circle().radius()
    }
}
