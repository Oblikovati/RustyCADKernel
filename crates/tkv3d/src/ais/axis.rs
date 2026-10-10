//! `AIS_Axis` at OCCT 3d097a0: retained geometry and constructor-time unit conversions.
use super::{AisInteractiveObject, AisInteractiveObjectBase, AisKindOfInteractive};
use crate::{prs_mgr::*, prs3d::Prs3dLineAspect, select_mgr::*};
use rustycad_tkernel::{
    standard::{Failure, Handle, SharedState, StandardTransient},
    units_api::UnitsApi,
};
use rustycad_tkg3d::geom::{GeomAxis1Placement, GeomAxis2Placement, GeomLine};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use std::sync::Arc;
/// `AIS_TypeOfAxis`; `AisTypeOfAxis::XAxis` selects the frame's X direction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum AisTypeOfAxis {
    /// Unclassified line datum.
    #[default]
    Unknown = 0,
    /// Local X axis.
    XAxis = 1,
    /// Local Y axis.
    YAxis = 2,
    /// Local Z axis.
    ZAxis = 3,
}
#[derive(Clone, Default)]
pub(super) struct AxisGeometry {
    pub component: Handle<GeomLine>,
    pub placement: Handle<GeomAxis2Placement>,
    pub kind: AisTypeOfAxis,
    pub xyz: bool,
    pub first: Point3,
    pub last: Point3,
    pub direction: Direction3,
    pub length: f64,
    pub label: &'static str,
    pub line: Handle<Prs3dLineAspect>,
}
/// Interactive line, coordinate axis or ray; `AisAxis::new(&line, units)?`.
pub struct AisAxis {
    transient: StandardTransient,
    pub(super) base: AisInteractiveObjectBase,
    pub(super) geometry: SharedState<AxisGeometry>,
    pub(super) units: Arc<SharedState<UnitsApi>>,
}
rustycad_tkernel::impl_transient!(AisAxis,"AIS_Axis",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisAxis {
    /// Construct an infinite datum and snapshot its selection ends; `AisAxis::new(&line, units)?`.
    pub fn new(
        line: &Handle<GeomLine>,
        units: Arc<SharedState<UnitsApi>>,
    ) -> Result<Handle<Self>, Failure> {
        let object = Self::allocate(units);
        object.initialize_axis_line()?;
        object.set_component(line)?;
        Ok(object)
    }
    /// Construct from an axis-one placement; `AisAxis::from_axis1(&placement, units)?`.
    pub fn from_axis1(
        placement: &Handle<GeomAxis1Placement>,
        units: Arc<SharedState<UnitsApi>>,
    ) -> Result<Handle<Self>, Failure> {
        Self::new(&Handle::new(GeomLine::new(placement.ax1())), units)
    }
    /// Construct a labeled coordinate axis; `AisAxis::from_axis2(&placement, AisTypeOfAxis::XAxis, units)?`.
    pub fn from_axis2(
        placement: &Handle<GeomAxis2Placement>,
        kind: AisTypeOfAxis,
        units: Arc<SharedState<UnitsApi>>,
    ) -> Result<Handle<Self>, Failure> {
        let object = Self::allocate(units);
        object.initialize_axis_datum()?;
        object.set_axis2_placement(placement, kind)?;
        Ok(object)
    }
    /// Construct an unlabeled ray; -1 selects the native unit-derived length; `AisAxis::from_ray(&axis, -1., units)?`.
    pub fn from_ray(
        axis: &Axis1,
        length: f64,
        units: Arc<SharedState<UnitsApi>>,
    ) -> Result<Handle<Self>, Failure> {
        let object = Self::allocate(units);
        object.initialize_axis_ray(axis, length)?;
        Ok(object)
    }
    fn allocate(units: Arc<SharedState<UnitsApi>>) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            geometry: SharedState::default(),
            units,
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
    /// Retain the geometric line handle; `axis.component()`.
    pub fn component(&self) -> Handle<GeomLine> {
        self.geometry.read().component.clone()
    }
    /// Retain the axis-two frame, even after switching to a line; `axis.axis2_placement()`.
    pub fn axis2_placement(&self) -> Handle<GeomAxis2Placement> {
        self.geometry.read().placement.clone()
    }
    /// Return the classification; `axis.type_of_axis()`.
    pub fn type_of_axis(&self) -> AisTypeOfAxis {
        self.geometry.read().kind
    }
    /// Change only classification, without rebuilding cached fields; `axis.set_type_of_axis(AisTypeOfAxis::YAxis)`.
    pub fn set_type_of_axis(&self, kind: AisTypeOfAxis) {
        self.geometry.write().kind = kind;
    }
    /// Whether presentation uses the axis builder; `axis.is_xyz_axis()`.
    pub fn is_xyz_axis(&self) -> bool {
        self.geometry.read().xyz
    }
    /// Replace a line and refresh unit-derived selection ends; `axis.set_component(&line)?`.
    pub fn set_component(&self, line: &Handle<GeomLine>) -> Result<(), Failure> {
        {
            let mut g = self.geometry.write();
            g.component = line.clone();
            g.kind = AisTypeOfAxis::Unknown;
            g.xyz = false;
        }
        self.set_infinite_state(true);
        self.refresh_line_endpoints()
    }
    /// Replace a frame and rebuild cached labeled endpoints; `axis.set_axis2_placement(&frame, AisTypeOfAxis::ZAxis)?`.
    pub fn set_axis2_placement(
        &self,
        placement: &Handle<GeomAxis2Placement>,
        kind: AisTypeOfAxis,
    ) -> Result<(), Failure> {
        {
            let mut g = self.geometry.write();
            g.placement = placement.clone();
            g.kind = kind;
            g.xyz = true;
        }
        self.compute_axis_fields()
    }
    /// Replace the component with a newly allocated line; `axis.set_axis1_placement(&frame)?`.
    pub fn set_axis1_placement(
        &self,
        placement: &Handle<GeomAxis1Placement>,
    ) -> Result<(), Failure> {
        self.set_component(&Handle::new(GeomLine::new(placement.ax1())))
    }
}
impl AisInteractiveObject for AisAxis {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        AisKindOfInteractive::Datum
    }
    fn signature(&self) -> i32 {
        2
    }
}
